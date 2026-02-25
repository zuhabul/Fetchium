//! Provider-agnostic AI chat client with automatic fallback chain.
//!
//! Supports both **API key** and **subscription OAuth** authentication:
//! - Gemini: reads `~/.gemini/oauth_creds.json` (Gemini CLI / "antigravity" auth)
//! - Anthropic: reads macOS Keychain entry `"Claude Code-credentials"` (Claude Code Max/Pro)
//! - OpenRouter, OpenAI: API key only
//! - Ollama, GeminiCli: local, no key needed

use crate::ai::credentials::{
    antigravity_auth_available, get_antigravity_token, get_claude_code_token,
    get_codex_token_if_valid, get_gemini_access_token_if_valid, invalidate_gemini_creds,
    ANTIGRAVITY_ENDPOINTS, ANTIGRAVITY_VERSION, GEMINI_OAUTH_CLIENT_ID, GEMINI_OAUTH_CLIENT_SECRET,
    GOOGLE_TOKEN_ENDPOINT,
};
use crate::ai::ollama::OllamaClient;
use crate::ai::providers::{ProviderKind, ProvidersConfig};
use crate::ai::types::AiConfig;
use crate::error::HsxError;
use futures::StreamExt;
use serde::Deserialize;

// ─── Public result type ───────────────────────────────────────────────────────

/// The answer returned by whichever provider succeeded.
#[derive(Debug)]
pub struct ChatResult {
    /// Generated text content.
    pub content: String,
    /// Identifier of the model that produced the answer (e.g. `"gemini-2.0-flash"`).
    pub model_used: String,
    /// Which provider was ultimately used.
    pub provider: ProviderKind,
}

// ─── Fallback dispatch ────────────────────────────────────────────────────────

/// Try each provider in the configured fallback chain and return the first success.
///
/// `on_token` is called for each streamed token (may be called 0 times for
/// non-streaming or batch responses).
pub async fn chat_with_fallback(
    messages: &[crate::ai::types::ChatMessage],
    model_override: Option<&str>,
    ai_config: &AiConfig,
    providers: &ProvidersConfig,
    streaming: bool,
    on_token: &mut dyn FnMut(&str),
) -> Result<ChatResult, HsxError> {
    let chain = providers.resolved_chain();

    if chain.is_empty() {
        return Err(HsxError::AiUnavailable(
            "No AI providers configured. Run `hsx provider setup` to get started.".into(),
        ));
    }

    let mut last_error: Option<HsxError> = None;

    for kind in &chain {
        match call_provider(
            *kind,
            messages,
            model_override,
            ai_config,
            providers,
            streaming,
            on_token,
        )
        .await
        {
            Ok(result) => return Ok(result),
            Err(e) => {
                tracing::warn!("Provider {} failed: {}", kind.slug(), e);
                last_error = Some(e);
            }
        }
    }

    Err(last_error
        .unwrap_or_else(|| HsxError::AiUnavailable("All configured AI providers failed.".into())))
}

/// Attempt a single provider and return its `ChatResult` on success.
async fn call_provider(
    kind: ProviderKind,
    messages: &[crate::ai::types::ChatMessage],
    model_override: Option<&str>,
    ai_config: &AiConfig,
    providers: &ProvidersConfig,
    streaming: bool,
    on_token: &mut dyn FnMut(&str),
) -> Result<ChatResult, HsxError> {
    let entry = providers.entry(kind);
    let model = model_override
        .map(|s| s.to_string())
        .unwrap_or_else(|| entry.resolve_model(kind));

    match kind {
        ProviderKind::Ollama => call_ollama(messages, &model, ai_config, streaming, on_token).await,

        ProviderKind::OpenAi => {
            // Priority: 1) config/env API key  2) auth store API key  3) OpenAI Codex CLI OAuth session
            let auth_store_key_openai = crate::ai::credentials::hsx_auth_get("openai")
                .and_then(|a| a.api_key().map(|s| s.to_string()));
            let key = if let Some(k) = entry
                .resolve_api_key("OPENAI_API_KEY")
                .or(auth_store_key_openai)
            {
                k
            } else if let Some(tok) = get_codex_token_if_valid() {
                tracing::info!("Using OpenAI Codex CLI OAuth session (ChatGPT subscription)");
                tok
            } else {
                return Err(HsxError::AiUnavailable(
                    "OpenAI: no API key or Codex CLI session found.\n  \
                     Options:\n  \
                     • Set OPENAI_API_KEY env var\n  \
                     • Log in via Codex CLI: codex auth login\n  \
                     • Run `hsx provider setup openai`"
                        .into(),
                ));
            };
            let base = entry
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.openai.com".into());
            call_openai_compat(
                kind, &key, &base, &model, messages, ai_config, streaming, on_token,
            )
            .await
        }

        ProviderKind::Anthropic => {
            // Priority: 1) config/env API key  2) auth store API key  3) Claude Code OAuth subscription
            let auth_store_key_anthropic = crate::ai::credentials::hsx_auth_get("anthropic")
                .and_then(|a| a.api_key().map(|s| s.to_string()));
            let (token, use_oauth) = if let Some(k) = entry
                .resolve_api_key("ANTHROPIC_API_KEY")
                .or(auth_store_key_anthropic)
            {
                (k, false)
            } else if let Some(creds) = get_claude_code_token() {
                tracing::info!(
                    "Using Claude Code {} subscription (OAuth)",
                    creds.subscription_type
                );
                (creds.access_token, true)
            } else {
                return Err(HsxError::AiUnavailable(
                    "Anthropic: no API key or Claude Code session found.\n  \
                     Options:\n  \
                     • Set ANTHROPIC_API_KEY env var\n  \
                     • Run `claude` once to log in (uses your Claude Max/Pro subscription)\n  \
                     • Run `hsx provider setup anthropic`"
                        .into(),
                ));
            };
            call_anthropic(
                &token, use_oauth, &model, messages, ai_config, streaming, on_token,
            )
            .await
        }

        ProviderKind::Gemini => {
            // Priority: 1) config/env API key  2) auth store API key  3) Gemini CLI OAuth  4) refresh
            let auth_store_key = crate::ai::credentials::hsx_auth_get("gemini")
                .and_then(|a| a.api_key().map(|s| s.to_string()));
            if let Some(api_key) = entry.resolve_api_key("GEMINI_API_KEY").or(auth_store_key) {
                call_gemini_api_key(&api_key, &model, messages, ai_config, streaming, on_token)
                    .await
            } else if let Some(access_token) = get_gemini_access_token_if_valid() {
                tracing::info!("Using Gemini OAuth (antigravity / Gemini CLI subscription)");
                call_gemini_oauth(
                    &access_token,
                    &model,
                    messages,
                    ai_config,
                    streaming,
                    on_token,
                )
                .await
            } else {
                // Try to refresh the expired token
                let entry_clone = entry.clone();
                match refresh_gemini_token_async(ai_config).await {
                    Some(fresh_token) => {
                        call_gemini_oauth(&fresh_token, &model, messages, ai_config, streaming, on_token).await
                    }
                    None if entry_clone.resolve_api_key("GEMINI_API_KEY").is_none() => {
                        Err(HsxError::AiUnavailable(
                            "Gemini: OAuth session expired and refresh token is no longer valid.\n  \
                             Fix (choose one):\n  \
                             • gemini auth login                          (re-authenticate, free)\n  \
                             • hsx provider set gemini --key AIza...      (persist API key in config)\n  \
                             • export GEMINI_API_KEY=AIza...              (env var, not stored)\n  \
                             Get a free Gemini API key: https://aistudio.google.com/app/apikey".into(),
                        ))
                    }
                    None => Err(HsxError::AiUnavailable("Gemini token refresh failed.".into())),
                }
            }
        }

        ProviderKind::GeminiCli => call_gemini_cli(&model, messages, on_token).await,

        ProviderKind::OpenRouter => {
            // Priority: 1) config/env  2) ~/.openrouter/config.json
            let key = entry.resolve_api_key("OPENROUTER_API_KEY")
                .or_else(crate::ai::credentials::read_openrouter_key)
                .ok_or_else(|| HsxError::AiUnavailable(
                    "OpenRouter API key not set. Set OPENROUTER_API_KEY or run `hsx provider setup openrouter`.".into(),
                ))?;
            let base = entry
                .base_url
                .clone()
                .unwrap_or_else(|| "https://openrouter.ai/api".into());
            call_openai_compat(
                kind, &key, &base, &model, messages, ai_config, streaming, on_token,
            )
            .await
        }

        ProviderKind::Antigravity => {
            // Refresh the antigravity Google OAuth access token on every call
            // (short-lived tokens, no local caching needed)
            let (access_token, project_id) = get_antigravity_token().ok_or_else(|| {
                HsxError::AiUnavailable(
                    "Antigravity: no OpenCode account found.\n  \
                     Install OpenCode and add an account: https://opencode.ai\n  \
                     Then install the plugin: npm i -g opencode-antigravity-auth\n  \
                     Run `hsx provider setup antigravity` to verify."
                        .into(),
                )
            })?;
            call_antigravity(
                &access_token,
                &project_id,
                &model,
                messages,
                ai_config,
                streaming,
                on_token,
            )
            .await
        }
    }
}

// ─── Ollama ───────────────────────────────────────────────────────────────────

async fn call_ollama(
    messages: &[crate::ai::types::ChatMessage],
    model: &str,
    ai_config: &AiConfig,
    streaming: bool,
    on_token: &mut dyn FnMut(&str),
) -> Result<ChatResult, HsxError> {
    let ollama = OllamaClient::new(ai_config);

    if !ollama.is_available().await {
        return Err(HsxError::AiUnavailable(
            "Ollama is not running. Start it with: ollama serve".into(),
        ));
    }

    let available = ollama.list_models().await.unwrap_or_default();
    let model_name = if available
        .iter()
        .any(|m| m.name == model || m.name.starts_with(model))
    {
        model.to_string()
    } else if let Some(first) = available.first() {
        tracing::warn!(
            "Requested model '{model}' not found; using '{}'",
            first.name
        );
        first.name.clone()
    } else {
        return Err(HsxError::AiUnavailable(format!(
            "No models installed in Ollama. Run: ollama pull {model}"
        )));
    };

    let content = if streaming {
        ollama
            .chat_stream(&model_name, messages, ai_config.temperature, on_token)
            .await?
    } else {
        ollama
            .chat(&model_name, messages, ai_config.temperature)
            .await?
    };

    Ok(ChatResult {
        content,
        model_used: model_name,
        provider: ProviderKind::Ollama,
    })
}

// ─── OpenAI-compatible (OpenAI + OpenRouter) ──────────────────────────────────

#[derive(Deserialize)]
struct OaiResponse {
    choices: Vec<OaiChoice>,
    model: Option<String>,
}
#[derive(Deserialize)]
struct OaiChoice {
    message: OaiMessage,
}
#[derive(Deserialize)]
struct OaiMessage {
    content: String,
}
#[derive(Deserialize)]
struct OaiStreamChunk {
    choices: Vec<OaiStreamChoice>,
}
#[derive(Deserialize)]
struct OaiStreamChoice {
    delta: OaiDelta,
}
#[derive(Deserialize, Default)]
struct OaiDelta {
    #[serde(default)]
    content: Option<String>,
}

#[allow(clippy::too_many_arguments)]
async fn call_openai_compat(
    kind: ProviderKind,
    api_key: &str,
    base_url: &str,
    model: &str,
    messages: &[crate::ai::types::ChatMessage],
    ai_config: &AiConfig,
    streaming: bool,
    on_token: &mut dyn FnMut(&str),
) -> Result<ChatResult, HsxError> {
    let url = format!("{}/v1/chat/completions", base_url.trim_end_matches('/'));
    let oai_msgs: Vec<serde_json::Value> = messages
        .iter()
        .map(|m| serde_json::json!({"role": m.role, "content": m.content}))
        .collect();

    let body = serde_json::json!({
        "model": model,
        "messages": oai_msgs,
        "temperature": ai_config.temperature,
        "stream": streaming,
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(ai_config.timeout_secs))
        .build()
        .map_err(|e| HsxError::AiUnavailable(format!("HTTP client build error: {e}")))?;

    let mut req = client
        .post(&url)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json");

    // OpenRouter requires these headers for proper attribution
    if kind == ProviderKind::OpenRouter {
        req = req
            .header(
                "HTTP-Referer",
                "https://github.com/hypersearchx/hypersearchx",
            )
            .header("X-Title", "HyperSearchX");
    }

    let resp = req.json(&body).send().await.map_err(|e| {
        HsxError::AiUnavailable(format!("Request to {} failed: {e}", kind.display_name()))
    })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        return Err(HsxError::AiUnavailable(format!(
            "{} API error {status}: {body_text}",
            kind.display_name()
        )));
    }

    if streaming {
        let mut stream = resp.bytes_stream();
        let mut full = String::new();
        let mut buf = Vec::<u8>::new();

        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(|e| HsxError::AiUnavailable(format!("Stream error: {e}")))?;
            buf.extend_from_slice(&bytes);

            while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = buf.drain(..=pos).collect();
                let s = std::str::from_utf8(&line).unwrap_or("").trim();
                if let Some(data) = s.strip_prefix("data: ") {
                    if data == "[DONE]" {
                        return Ok(ChatResult {
                            content: full,
                            model_used: model.to_string(),
                            provider: kind,
                        });
                    }
                    if let Ok(chunk) = serde_json::from_str::<OaiStreamChunk>(data) {
                        for choice in &chunk.choices {
                            if let Some(ref tok) = choice.delta.content {
                                on_token(tok);
                                full.push_str(tok);
                            }
                        }
                    }
                }
            }
        }
        Ok(ChatResult {
            content: full,
            model_used: model.to_string(),
            provider: kind,
        })
    } else {
        let parsed: OaiResponse = resp
            .json()
            .await
            .map_err(|e| HsxError::AiUnavailable(format!("Invalid response: {e}")))?;
        let content = parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();
        let model_used = parsed.model.unwrap_or_else(|| model.to_string());
        Ok(ChatResult {
            content,
            model_used,
            provider: kind,
        })
    }
}

// ─── Anthropic ────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicBlock>,
    model: String,
}
#[derive(Deserialize)]
struct AnthropicBlock {
    #[serde(rename = "type")]
    kind: String,
    text: Option<String>,
}

async fn call_anthropic(
    token: &str,
    use_oauth: bool, // true = Claude Code subscription (Bearer), false = API key (x-api-key)
    model: &str,
    messages: &[crate::ai::types::ChatMessage],
    ai_config: &AiConfig,
    streaming: bool,
    on_token: &mut dyn FnMut(&str),
) -> Result<ChatResult, HsxError> {
    let mut system_text = String::new();
    let mut anth_msgs: Vec<serde_json::Value> = Vec::new();

    for m in messages {
        if m.role == "system" {
            system_text = m.content.clone();
        } else {
            anth_msgs.push(serde_json::json!({"role": m.role, "content": m.content}));
        }
    }
    if anth_msgs.is_empty() {
        anth_msgs.push(serde_json::json!({"role": "user", "content": "Hello"}));
    }

    let mut body = serde_json::json!({
        "model": model,
        "max_tokens": ai_config.max_context_tokens,
        "temperature": ai_config.temperature,
        "messages": anth_msgs,
        "stream": streaming,
    });
    if !system_text.is_empty() {
        body["system"] = serde_json::Value::String(system_text);
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(ai_config.timeout_secs))
        .build()
        .map_err(|e| HsxError::AiUnavailable(format!("HTTP client build error: {e}")))?;

    let mut req = client
        .post("https://api.anthropic.com/v1/messages")
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json");

    if use_oauth {
        // Claude Code subscription: OAuth Bearer token (sk-ant-oat01-…)
        req = req.header("Authorization", format!("Bearer {token}"));
    } else {
        // Direct API key
        req = req.header("x-api-key", token);
    }

    let resp = req
        .json(&body)
        .send()
        .await
        .map_err(|e| HsxError::AiUnavailable(format!("Anthropic request failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        return Err(HsxError::AiUnavailable(format!(
            "Anthropic API error {status}: {body_text}"
        )));
    }

    if streaming {
        let mut stream = resp.bytes_stream();
        let mut full = String::new();
        let mut buf = Vec::<u8>::new();

        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(|e| HsxError::AiUnavailable(format!("Stream error: {e}")))?;
            buf.extend_from_slice(&bytes);

            while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = buf.drain(..=pos).collect();
                let s = std::str::from_utf8(&line).unwrap_or("").trim();
                if let Some(data) = s.strip_prefix("data: ") {
                    if let Ok(ev) = serde_json::from_str::<serde_json::Value>(data) {
                        if ev["type"] == "content_block_delta" {
                            if let Some(text) = ev["delta"]["text"].as_str() {
                                on_token(text);
                                full.push_str(text);
                            }
                        }
                    }
                }
            }
        }
        Ok(ChatResult {
            content: full,
            model_used: model.to_string(),
            provider: ProviderKind::Anthropic,
        })
    } else {
        let parsed: AnthropicResponse = resp
            .json()
            .await
            .map_err(|e| HsxError::AiUnavailable(format!("Invalid Anthropic response: {e}")))?;
        let content = parsed
            .content
            .into_iter()
            .filter(|b| b.kind == "text")
            .filter_map(|b| b.text)
            .collect::<Vec<_>>()
            .join("");
        Ok(ChatResult {
            content,
            model_used: parsed.model,
            provider: ProviderKind::Anthropic,
        })
    }
}

// ─── Google Gemini REST API ───────────────────────────────────────────────────

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}
#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}
#[derive(Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}
#[derive(Deserialize)]
struct GeminiPart {
    text: Option<String>,
}

/// Build Gemini request contents from ChatMessage list (system → prepended to first user turn).
fn gemini_build_contents(messages: &[crate::ai::types::ChatMessage]) -> Vec<serde_json::Value> {
    let mut system_text = String::new();
    let mut contents: Vec<serde_json::Value> = Vec::new();

    for m in messages {
        if m.role == "system" {
            system_text = m.content.clone();
        } else {
            let role = if m.role == "assistant" {
                "model"
            } else {
                "user"
            };
            contents.push(serde_json::json!({
                "role": role,
                "parts": [{"text": m.content}],
            }));
        }
    }

    if !system_text.is_empty() {
        if let Some(first) = contents.first_mut() {
            if first["role"] == "user" {
                let original = first["parts"][0]["text"].as_str().unwrap_or("").to_string();
                first["parts"][0]["text"] =
                    serde_json::Value::String(format!("{system_text}\n\n{original}"));
            }
        }
    }
    if contents.is_empty() {
        contents.push(serde_json::json!({"role": "user", "parts": [{"text": "Hello"}]}));
    }
    contents
}

/// Parse a Gemini SSE stream into a complete response string, calling `on_token` for each chunk.
async fn gemini_read_stream(
    resp: reqwest::Response,
    model: &str,
    on_token: &mut dyn FnMut(&str),
) -> Result<ChatResult, HsxError> {
    gemini_read_stream_with_provider(resp, model, ProviderKind::Gemini, on_token).await
}

/// Parse a Gemini SSE stream, tagging the result with the given provider.
async fn gemini_read_stream_with_provider(
    resp: reqwest::Response,
    model: &str,
    provider: ProviderKind,
    on_token: &mut dyn FnMut(&str),
) -> Result<ChatResult, HsxError> {
    let mut stream = resp.bytes_stream();
    let mut full = String::new();
    let mut buf = Vec::<u8>::new();

    while let Some(chunk) = stream.next().await {
        let bytes = chunk.map_err(|e| HsxError::AiUnavailable(format!("Stream error: {e}")))?;
        buf.extend_from_slice(&bytes);

        while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = buf.drain(..=pos).collect();
            let s = std::str::from_utf8(&line).unwrap_or("").trim();
            if let Some(data) = s.strip_prefix("data: ") {
                if let Ok(parsed) = serde_json::from_str::<GeminiResponse>(data) {
                    for cand in parsed.candidates {
                        for part in cand.content.parts {
                            if let Some(text) = part.text {
                                on_token(&text);
                                full.push_str(&text);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(ChatResult {
        content: full,
        model_used: model.to_string(),
        provider,
    })
}

/// Call Gemini REST API with an **API key** (`?key=` query parameter).
async fn call_gemini_api_key(
    api_key: &str,
    model: &str,
    messages: &[crate::ai::types::ChatMessage],
    ai_config: &AiConfig,
    streaming: bool,
    on_token: &mut dyn FnMut(&str),
) -> Result<ChatResult, HsxError> {
    let contents = gemini_build_contents(messages);
    let body = serde_json::json!({
        "contents": contents,
        "generationConfig": {
            "maxOutputTokens": ai_config.max_context_tokens,
            "temperature": ai_config.temperature,
        },
    });

    let endpoint = if streaming {
        "streamGenerateContent?alt=sse"
    } else {
        "generateContent"
    };
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{model}:{endpoint}&key={api_key}"
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(ai_config.timeout_secs))
        .build()
        .map_err(|e| HsxError::AiUnavailable(format!("HTTP client build error: {e}")))?;

    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| HsxError::AiUnavailable(format!("Gemini request failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        return Err(HsxError::AiUnavailable(format!(
            "Gemini API error {status}: {body_text}"
        )));
    }

    if streaming {
        gemini_read_stream(resp, model, on_token).await
    } else {
        let parsed: GeminiResponse = resp
            .json()
            .await
            .map_err(|e| HsxError::AiUnavailable(format!("Invalid Gemini response: {e}")))?;
        let content = parsed
            .candidates
            .into_iter()
            .next()
            .map(|c| {
                c.content
                    .parts
                    .into_iter()
                    .filter_map(|p| p.text)
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_default();
        Ok(ChatResult {
            content,
            model_used: model.to_string(),
            provider: ProviderKind::Gemini,
        })
    }
}

/// Call Gemini REST API with a **Google OAuth Bearer token** (antigravity / Gemini CLI subscription).
///
/// No `?key=` parameter — authentication is via `Authorization: Bearer {access_token}`.
async fn call_gemini_oauth(
    access_token: &str,
    model: &str,
    messages: &[crate::ai::types::ChatMessage],
    ai_config: &AiConfig,
    streaming: bool,
    on_token: &mut dyn FnMut(&str),
) -> Result<ChatResult, HsxError> {
    let contents = gemini_build_contents(messages);
    let body = serde_json::json!({
        "contents": contents,
        "generationConfig": {
            "maxOutputTokens": ai_config.max_context_tokens,
            "temperature": ai_config.temperature,
        },
    });

    // OAuth endpoint: no `?key=` parameter
    let endpoint = if streaming {
        "streamGenerateContent?alt=sse"
    } else {
        "generateContent"
    };
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{model}:{endpoint}");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(ai_config.timeout_secs))
        .build()
        .map_err(|e| HsxError::AiUnavailable(format!("HTTP client build error: {e}")))?;

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {access_token}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| HsxError::AiUnavailable(format!("Gemini OAuth request failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        // 403 SCOPE_INSUFFICIENT means the Gemini CLI OAuth token was obtained with
        // limited scopes — it cannot be used for the Generative Language REST API.
        // Do NOT clear credentials (they're still valid for the `gemini` CLI subprocess).
        if status == reqwest::StatusCode::FORBIDDEN
            && body_text.contains("ACCESS_TOKEN_SCOPE_INSUFFICIENT")
        {
            return Err(HsxError::AiUnavailable(
                "Gemini OAuth token has insufficient scopes for the REST API.
                   Fix (choose one):
                   • hsx provider auth gemini       (interactive setup — API key or OAuth)
                   • export GEMINI_API_KEY=AIza...   (free key: aistudio.google.com/app/apikey)
                   • hsx provider chain gemini_cli   (use the local `gemini` CLI instead)"
                    .into(),
            ));
        }
        return Err(HsxError::AiUnavailable(format!(
            "Gemini OAuth API error {status}: {body_text}"
        )));
    }

    if streaming {
        gemini_read_stream(resp, model, on_token).await
    } else {
        let parsed: GeminiResponse = resp
            .json()
            .await
            .map_err(|e| HsxError::AiUnavailable(format!("Invalid Gemini OAuth response: {e}")))?;
        let content = parsed
            .candidates
            .into_iter()
            .next()
            .map(|c| {
                c.content
                    .parts
                    .into_iter()
                    .filter_map(|p| p.text)
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_default();
        Ok(ChatResult {
            content,
            model_used: model.to_string(),
            provider: ProviderKind::Gemini,
        })
    }
}

/// Refresh an expired Gemini OAuth token using the stored refresh_token.
///
/// Uses the same OAuth client ID/secret as the Gemini CLI (installed-app flow).
/// Returns the new access token string on success, or `None` if refresh fails.
async fn refresh_gemini_token_async(_ai_config: &AiConfig) -> Option<String> {
    use crate::ai::credentials::read_gemini_creds;

    let creds = read_gemini_creds()?;
    let refresh_token = &creds.refresh_token;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .ok()?;

    let params = [
        ("client_id", GEMINI_OAUTH_CLIENT_ID),
        ("client_secret", GEMINI_OAUTH_CLIENT_SECRET),
        ("refresh_token", refresh_token.as_str()),
        ("grant_type", "refresh_token"),
    ];

    let resp = client
        .post(GOOGLE_TOKEN_ENDPOINT)
        .form(&params)
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::BAD_REQUEST
        {
            // Refresh token is revoked or invalid — clear stale creds so future
            // check_provider calls correctly report Gemini as unavailable.
            invalidate_gemini_creds();
            tracing::warn!(
                "Gemini refresh token revoked (HTTP {status}) — credentials cleared.\n  \
                 Fix: run `gemini auth login` to get a new session, then retry."
            );
        } else {
            tracing::warn!("Gemini token refresh failed: HTTP {status}");
        }
        return None;
    }

    let json: serde_json::Value = resp.json().await.ok()?;
    let new_token = json["access_token"].as_str()?.to_string();

    // Persist the refreshed token back to ~/.gemini/oauth_creds.json so subsequent
    // calls within this session and future sessions can use it directly.
    if let Some(path) = dirs::home_dir().map(|h| h.join(".gemini").join("oauth_creds.json")) {
        let expires_in: u64 = json["expires_in"].as_u64().unwrap_or(3600);
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let updated = serde_json::json!({
            "access_token":  new_token,
            "refresh_token": refresh_token,
            "expiry_date":   now_ms + expires_in * 1000,
            "token_type":    "Bearer",
        });
        let _ = std::fs::write(
            &path,
            serde_json::to_string_pretty(&updated).unwrap_or_default(),
        );
        tracing::debug!("Gemini OAuth token refreshed and saved to {:?}", path);
    }

    Some(new_token)
}

// ─── OpenCode Antigravity (Google Cloud Code Assist) ─────────────────────────
//
// Antigravity routes Gemini-format requests to Google's internal Cloud Code Assist
// API, which proxies them to Gemini 3 (Pro/Flash) and Claude (Sonnet/Opus) models
// without requiring any API key — only a Google OAuth access token from the
// `opencode-antigravity-auth` plugin.
//
// Request format: same as Gemini REST API (contents[] body)
// Auth:           Authorization: Bearer {access_token}
// Extra headers:  User-Agent, X-Goog-Api-Client, Client-Metadata (required for routing)
// Endpoint:       {cloudcode-pa}/v1internal:generateContent[?alt=sse]

/// Call the Google Cloud Code Assist API via Antigravity OAuth credentials.
///
/// The body format is identical to the Gemini REST API (`contents[]`).
/// The endpoint routes automatically based on the model name prefix:
/// - `antigravity-gemini-*`  → Gemini 3 models
/// - `antigravity-claude-*`  → Claude models via Vertex AI
#[allow(clippy::too_many_arguments)]
async fn call_antigravity(
    access_token: &str,
    _project_id: &str,
    model: &str,
    messages: &[crate::ai::types::ChatMessage],
    ai_config: &AiConfig,
    streaming: bool,
    on_token: &mut dyn FnMut(&str),
) -> Result<ChatResult, HsxError> {
    let contents = gemini_build_contents(messages);
    let body = serde_json::json!({
        "contents": contents,
        "generationConfig": {
            "maxOutputTokens": ai_config.max_context_tokens,
            "temperature": ai_config.temperature,
        },
        "model": model,
    });

    let ua = format!(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
         (KHTML, like Gecko) Antigravity/{ANTIGRAVITY_VERSION} \
         Chrome/138.0.7204.235 Electron/37.3.1 Safari/537.36"
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(ai_config.timeout_secs))
        .user_agent(&ua)
        .build()
        .map_err(|e| HsxError::AiUnavailable(format!("HTTP client build error: {e}")))?;

    let action = if streaming {
        "generateContent?alt=sse"
    } else {
        "generateContent"
    };

    // Try each endpoint in order; first success wins
    let mut last_err = String::new();
    for &endpoint in ANTIGRAVITY_ENDPOINTS {
        let url = format!("{endpoint}/v1internal:{action}");

        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {access_token}"))
            .header("Content-Type", "application/json")
            .header("X-Goog-Api-Client", "google-cloud-sdk vscode_cloudshelleditor/0.1")
            .header("Client-Metadata", r#"{"ideType":"IDE_UNSPECIFIED","platform":"PLATFORM_UNSPECIFIED","pluginType":"GEMINI"}"#)
            .json(&body)
            .send()
            .await;

        let resp = match resp {
            Ok(r) => r,
            Err(e) => {
                last_err = format!("Request to {endpoint} failed: {e}");
                tracing::debug!("Antigravity endpoint {endpoint} failed: {e}");
                continue;
            }
        };

        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(HsxError::AiUnavailable(
                "Antigravity: authentication failed — re-run OpenCode to refresh your session.\n  \
                 Run `opencode` once, then retry."
                    .into(),
            ));
        }
        if !status.is_success() {
            let body_text = resp.text().await.unwrap_or_default();
            last_err = format!("Antigravity API error {status} at {endpoint}: {body_text}");
            tracing::debug!("{last_err}");
            continue;
        }

        return if streaming {
            // Gemini SSE format
            gemini_read_stream_with_provider(resp, model, ProviderKind::Antigravity, on_token).await
        } else {
            let parsed: GeminiResponse = resp.json().await.map_err(|e| {
                HsxError::AiUnavailable(format!("Invalid Antigravity response: {e}"))
            })?;
            let content = parsed
                .candidates
                .into_iter()
                .next()
                .map(|c| {
                    c.content
                        .parts
                        .into_iter()
                        .filter_map(|p| p.text)
                        .collect::<Vec<_>>()
                        .join("")
                })
                .unwrap_or_default();
            Ok(ChatResult {
                content,
                model_used: model.to_string(),
                provider: ProviderKind::Antigravity,
            })
        };
    }

    Err(HsxError::AiUnavailable(format!(
        "Antigravity: all endpoints failed. Last error: {last_err}\n  \
         Ensure your OpenCode session is active and the antigravity plugin is installed."
    )))
}

// ─── Gemini CLI subprocess ────────────────────────────────────────────────────

async fn call_gemini_cli(
    model: &str,
    messages: &[crate::ai::types::ChatMessage],
    on_token: &mut dyn FnMut(&str),
) -> Result<ChatResult, HsxError> {
    let prompt = messages
        .iter()
        .map(|m| {
            if m.role == "system" {
                format!("[System Instructions]: {}", m.content)
            } else {
                m.content.clone()
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    // Gemini CLI v0.30+ sends `thinking_config.include_thoughts` on every request.
    // Only models that support thinking (gemini-2.5-flash, gemini-3-flash-preview, etc.) work.
    // Always pass `--model` so the CLI uses the explicitly selected model.
    //
    // Capacity fallback: if `model` is capacity-exhausted (429 MODEL_CAPACITY_EXHAUSTED)
    // we automatically retry once with `gemini-2.5-flash` before giving up.
    const CAPACITY_FALLBACK: &str = "gemini-2.5-flash";
    let models_to_try: Vec<String> = if model == CAPACITY_FALLBACK {
        vec![model.to_string()]
    } else {
        vec![model.to_string(), CAPACITY_FALLBACK.to_string()]
    };

    let mut last_err: Option<HsxError> = None;

    for try_model in &models_to_try {
        let args = vec![
            "--model".to_string(),
            try_model.clone(),
            "--prompt".to_string(),
            prompt.clone(),
        ];

        let output = tokio::process::Command::new("gemini")
            .args(&args)
            .output()
            .await
            .map_err(|e| {
                HsxError::ExternalTool(format!(
                    "Gemini CLI not found: {e}\nInstall: npm install -g @google/gemini-cli"
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            let is_not_last = try_model != models_to_try.last().unwrap();
            let is_capacity = stderr.contains("MODEL_CAPACITY_EXHAUSTED")
                || stderr.contains("No capacity available")
                || stderr.contains("RESOURCE_EXHAUSTED");
            if is_capacity && is_not_last {
                tracing::warn!(
                    "GeminiCli: '{try_model}' capacity exhausted, retrying with '{CAPACITY_FALLBACK}'"
                );
                last_err = Some(HsxError::ExternalTool(format!("Gemini CLI error: {stderr}")));
                continue;
            }
            return Err(HsxError::ExternalTool(format!("Gemini CLI error: {stderr}")));
        }

        let content = String::from_utf8_lossy(&output.stdout).trim().to_string();
        // Gemini CLI doesn't stream — emit the full response as one token.
        on_token(&content);
        return Ok(ChatResult {
            content,
            model_used: format!("gemini-cli/{try_model}"),
            provider: ProviderKind::GeminiCli,
        });
    }

    Err(last_err
        .unwrap_or_else(|| HsxError::ExternalTool("GeminiCli: all models capacity-exhausted".into())))
}

// ─── Provider availability check ─────────────────────────────────────────────

/// Status of an AI provider for display in `hsx provider list` / `hsx doctor`.
#[derive(Debug)]
pub enum ProviderStatus {
    /// Provider is reachable and configured.
    Available {
        /// Number of installed models (Ollama only).
        model_count: Option<usize>,
    },
    /// Provider is not reachable or not configured.
    Unavailable {
        /// Human-readable reason.
        reason: String,
    },
}

/// Check whether a single provider is available without making a real LLM call.
pub async fn check_provider(
    kind: ProviderKind,
    providers: &ProvidersConfig,
    ai_config: &AiConfig,
) -> ProviderStatus {
    let entry = providers.entry(kind);

    match kind {
        ProviderKind::Ollama => {
            let ollama = OllamaClient::new(ai_config);
            if ollama.is_available().await {
                let models = ollama.list_models().await.unwrap_or_default();
                ProviderStatus::Available {
                    model_count: Some(models.len()),
                }
            } else {
                ProviderStatus::Unavailable {
                    reason: "Ollama not running (start: ollama serve)".into(),
                }
            }
        }

        ProviderKind::GeminiCli => {
            let found = std::process::Command::new("gemini")
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if found {
                ProviderStatus::Available { model_count: None }
            } else {
                ProviderStatus::Unavailable {
                    reason: "`gemini` not in PATH (npm install -g @google/generative-ai-cli)"
                        .into(),
                }
            }
        }

        ProviderKind::OpenAi => {
            if entry.resolve_api_key("OPENAI_API_KEY").is_some()
                || crate::ai::credentials::codex_auth_available()
            {
                ProviderStatus::Available { model_count: None }
            } else {
                ProviderStatus::Unavailable {
                    reason: "No API key or Codex CLI session (set OPENAI_API_KEY or run `codex auth login`)".into(),
                }
            }
        }

        ProviderKind::Anthropic => {
            if entry.resolve_api_key("ANTHROPIC_API_KEY").is_some()
                || crate::ai::credentials::get_claude_code_token().is_some()
            {
                ProviderStatus::Available { model_count: None }
            } else {
                ProviderStatus::Unavailable {
                    reason: "No API key or Claude Code session (run `claude` once to log in)"
                        .into(),
                }
            }
        }

        ProviderKind::Gemini => {
            // Creds file "existing" is not enough — the refresh token must be non-empty.
            // invalidate_gemini_creds() sets refresh_token="" when a 401 is received,
            // so a dead session is correctly reported as unavailable on the next check.
            if entry.resolve_api_key("GEMINI_API_KEY").is_some()
                || crate::ai::credentials::get_gemini_access_token_if_valid().is_some()
                || crate::ai::credentials::read_gemini_creds()
                    .map(|c| c.is_refreshable())
                    .unwrap_or(false)
            {
                ProviderStatus::Available { model_count: None }
            } else {
                ProviderStatus::Unavailable {
                    reason: "No API key or valid OAuth session.\n    \
                             Fix: run `gemini auth login`  OR  \
                             `hsx provider set gemini --key AIza...`"
                        .into(),
                }
            }
        }

        ProviderKind::OpenRouter => {
            if entry.resolve_api_key("OPENROUTER_API_KEY").is_some()
                || crate::ai::credentials::read_openrouter_key().is_some()
            {
                ProviderStatus::Available { model_count: None }
            } else {
                ProviderStatus::Unavailable {
                    reason:
                        "No API key (set OPENROUTER_API_KEY or run `hsx provider setup openrouter`)"
                            .into(),
                }
            }
        }

        ProviderKind::Antigravity => {
            if antigravity_auth_available() {
                ProviderStatus::Available { model_count: None }
            } else {
                ProviderStatus::Unavailable {
                    reason: "No OpenCode Antigravity account found.\n    \
                             Install: npm i -g opencode-antigravity-auth\n    \
                             Then run `opencode` to authenticate."
                        .into(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::providers::ProvidersConfig;

    #[test]
    fn chat_result_has_provider() {
        let r = ChatResult {
            content: "hello".into(),
            model_used: "test-model".into(),
            provider: ProviderKind::Gemini,
        };
        assert_eq!(r.provider, ProviderKind::Gemini);
    }

    #[tokio::test]
    async fn empty_chain_returns_error() {
        let ai_config = AiConfig::default();
        let mut providers = ProvidersConfig::default();
        providers.fallback_chain = vec!["nonexistent_provider_xyz".into()];
        let msgs = vec![];
        let mut noop = |_: &str| {};
        let result =
            chat_with_fallback(&msgs, None, &ai_config, &providers, false, &mut noop).await;
        // Resolved chain will be empty (unknown slug filtered out) → error
        assert!(result.is_err());
    }
}
