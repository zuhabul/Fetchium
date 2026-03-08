//! AI-powered URL/text summarization pipeline.
//!
//! Pipeline: `input → detect_type → fetch (if URL) → CEP extract → QATBE → AI summarize`
//!
//! Used by `fetchium summarize` CLI command and the REST API.

use crate::ai::prompt::summarize_prompt;
use crate::ai::provider_client::chat_with_fallback;
use crate::ai::providers::ProviderKind;
use crate::ai::types::{AiConfig, ChatMessage};
use crate::config::FetchiumConfig;
use crate::error::{ErrorKind, FetchiumError};
use crate::http::client::HttpClient;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Desired summary length.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SummaryLength {
    /// ~100 words
    Short,
    /// ~300 words
    #[default]
    Medium,
    /// ~700 words
    Long,
}

impl SummaryLength {
    fn description(self) -> &'static str {
        match self {
            Self::Short => "short (~100 words)",
            Self::Medium => "medium (~300 words)",
            Self::Long => "long (~700 words)",
        }
    }

    /// Number of sentences for heuristic fallback.
    fn fallback_sentences(self) -> usize {
        match self {
            Self::Short => 3,
            Self::Medium => 7,
            Self::Long => 15,
        }
    }

    /// Max characters sent to the LLM for synthesis (latency/cost guardrail).
    fn max_input_chars(self) -> usize {
        match self {
            Self::Short => 8_000,
            Self::Medium => 16_000,
            Self::Long => 28_000,
        }
    }
}

/// Configuration for the summarize pipeline.
#[derive(Debug, Clone)]
pub struct SummarizeConfig {
    /// Desired summary length.
    pub length: SummaryLength,
    /// Optional model override for AI synthesis.
    pub model: Option<String>,
}

impl Default for SummarizeConfig {
    fn default() -> Self {
        Self {
            length: SummaryLength::Medium,
            model: None,
        }
    }
}

/// Result of summarization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryResult {
    /// The generated summary text.
    pub summary: String,
    /// Source URL if input was a URL.
    pub source_url: Option<String>,
    /// Title extracted from the source (if URL).
    pub source_title: Option<String>,
    /// Whether AI was used (false = heuristic fallback).
    pub ai_used: bool,
    /// Duration in milliseconds.
    pub duration_ms: u64,
}

/// Summarize a URL or raw text.
///
/// If `input` starts with `http://` or `https://`, it is fetched and extracted first.
/// Otherwise it is treated as raw text.
pub async fn summarize(
    input: &str,
    config: &SummarizeConfig,
    fetchium_config: &FetchiumConfig,
) -> Result<SummaryResult, FetchiumError> {
    let start = Instant::now();

    let is_url = input.starts_with("http://") || input.starts_with("https://");

    let (content, source_url, source_title) = if is_url {
        let http = HttpClient::new(fetchium_config)?;
        // YouTube URLs: extract transcript instead of HTML for much better summaries
        let is_youtube = input.contains("youtube.com/watch")
            || input.contains("youtu.be/")
            || input.contains("youtube.com/shorts/");

        if is_youtube {
            let video_id = crate::multimodal::video::extract_video_id(input).ok();
            let metadata_fut = async {
                if let Some(ref vid) = video_id {
                    crate::youtube::metadata::fetch_metadata(vid, &http, fetchium_config)
                        .await
                        .ok()
                } else {
                    None
                }
            };
            let (transcript_result, metadata_opt) = tokio::join!(
                crate::youtube::universal::fetch_universal_transcript(
                    input,
                    &http,
                    fetchium_config
                ),
                metadata_fut
            );

            match transcript_result {
                Ok(transcript) if !transcript.full_text.trim().is_empty() => {
                    let video_id = transcript.video_id.clone();
                    let text = transcript.full_text.clone();
                    let title = metadata_opt
                        .as_ref()
                        .map(|m| m.title.clone())
                        .unwrap_or_else(|| format!("YouTube Video ({})", video_id));
                    tracing::info!(
                        "YouTube transcript extracted: {} chars for video {}",
                        text.len(),
                        video_id
                    );
                    (text, Some(input.to_string()), Some(title))
                }
                Ok(transcript) => {
                    tracing::warn!(
                        "YouTube transcript was empty for video {}, falling back to metadata",
                        transcript.video_id
                    );
                    let title = metadata_opt
                        .as_ref()
                        .map(|m| m.title.clone())
                        .unwrap_or_else(|| format!("YouTube Video ({})", transcript.video_id));
                    let context = metadata_opt
                        .as_ref()
                        .map(|m| {
                            format!(
                                "Title: {}\nChannel: {}\nDescription: {}",
                                m.title,
                                m.channel.name,
                                trim_to_chars(&m.description, 900),
                            )
                        })
                        .unwrap_or_else(|| "No metadata available.".to_string());
                    (
                        format!(
                            "This YouTube video's transcript could not be extracted.\n\n{context}"
                        ),
                        Some(input.to_string()),
                        Some(title),
                    )
                }
                Err(e) => {
                    tracing::warn!(
                        "YouTube transcript fetch failed: {}. Falling back to HTML.",
                        e
                    );
                    // Fall back to regular HTML fetch if transcript unavailable
                    let html = fetch_html_with_block_fallback(&http, input).await?;
                    let extracted = crate::extract::pipeline::extract(&html, input);
                    let title = if extracted.title.is_empty() {
                        None
                    } else {
                        Some(extracted.title)
                    };
                    (extracted.text, Some(input.to_string()), title)
                }
            }
        } else {
            let html = fetch_html_with_block_fallback(&http, input).await?;
            let extracted = crate::extract::pipeline::extract(&html, input);
            let title = if extracted.title.is_empty() {
                None
            } else {
                Some(extracted.title)
            };
            (extracted.text, Some(input.to_string()), title)
        }
    } else {
        (input.to_string(), None, None)
    };

    if content.trim().is_empty() {
        return Ok(SummaryResult {
            summary: String::new(),
            source_url,
            source_title,
            ai_used: false,
            duration_ms: start.elapsed().as_millis() as u64,
        });
    }

    // Try AI synthesis
    let ai_config = AiConfig::from_fetchium_config(fetchium_config);
    let has_ai = has_reachable_ai_provider(&ai_config, fetchium_config).await;

    if has_ai {
        let system = summarize_prompt(config.length.description());
        let content_for_ai = prepare_summary_input(&content, config.length);
        let messages = vec![
            ChatMessage {
                role: "system".into(),
                content: system,
            },
            ChatMessage {
                role: "user".into(),
                content: content_for_ai,
            },
        ];

        let providers = ai_config.providers.clone();
        let model_override = config.model.clone();

        // Use thread isolation to keep future Send-safe (required for axum handlers).
        let (tx, rx) = tokio::sync::oneshot::channel::<Option<String>>();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            let result = rt.block_on(async move {
                let mut noop = |_: &str| {};
                match tokio::time::timeout(
                    std::time::Duration::from_secs(12),
                    chat_with_fallback(
                        &messages,
                        model_override.as_deref(),
                        &ai_config,
                        &providers,
                        false,
                        &mut noop,
                    ),
                )
                .await
                {
                    Ok(Ok(result)) => Some(result.content),
                    _ => None,
                }
            });
            let _ = tx.send(result);
        });

        if let Ok(Some(summary)) = rx.await {
            if !summary.is_empty() && grounding_score(&summary, &content) >= 0.12 {
                return Ok(SummaryResult {
                    summary,
                    source_url,
                    source_title,
                    ai_used: true,
                    duration_ms: start.elapsed().as_millis() as u64,
                });
            }
        }
    }

    // Heuristic fallback: take first N sentences
    let sentences: Vec<&str> = content.split(". ").collect();
    let n = config.length.fallback_sentences().min(sentences.len());
    let summary = sentences[..n]
        .iter()
        .map(|s| format!("{}.", s.trim()))
        .collect::<Vec<_>>()
        .join(" ");

    Ok(SummaryResult {
        summary,
        source_url,
        source_title,
        ai_used: false,
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

async fn has_reachable_ai_provider(ai_config: &AiConfig, fetchium_config: &FetchiumConfig) -> bool {
    let configured = ai_config.providers.configured_providers();
    if configured.is_empty() {
        return false;
    }

    // Non-local network providers are considered reachable if configured.
    if configured
        .iter()
        .any(|k| *k != ProviderKind::Ollama && *k != ProviderKind::GeminiCli)
    {
        return true;
    }

    // Local-only providers require local availability checks.
    let has_ollama = configured.contains(&ProviderKind::Ollama);
    if !has_ollama {
        return false;
    }

    // Ollama is configured: check if local server is actually reachable.
    if let Ok(http) = HttpClient::new(fetchium_config) {
        let url = format!(
            "{}:{}/api/tags",
            ai_config.ollama_host, ai_config.ollama_port
        );
        return tokio::time::timeout(std::time::Duration::from_millis(1200), async {
            http.fetch_text_once(&url).await.is_ok()
        })
        .await
        .unwrap_or(false);
    }
    false
}

async fn fetch_html_with_block_fallback(http: &HttpClient, url: &str) -> Result<String, FetchiumError> {
    match http.fetch_text(url).await {
        Ok(html) => Ok(html),
        Err(err) => {
            if !is_block_like_fetch_error(&err) {
                return Err(err);
            }

            let normalized = url
                .trim_start_matches("https://")
                .trim_start_matches("http://");
            let mirror_url = format!("https://r.jina.ai/http://{normalized}");

            match tokio::time::timeout(
                std::time::Duration::from_secs(8),
                http.fetch_text_once(&mirror_url),
            )
            .await
            {
                Ok(Ok(body)) if !body.trim().is_empty() => {
                    tracing::warn!("Using reader fallback for blocked URL: {}", url);
                    Ok(body)
                }
                _ => Err(err),
            }
        }
    }
}

fn is_block_like_fetch_error(err: &FetchiumError) -> bool {
    match err {
        FetchiumError::Structured(se) => {
            if matches!(
                se.kind,
                ErrorKind::Http403
                    | ErrorKind::Http429
                    | ErrorKind::Http5xx
                    | ErrorKind::AntiBot
                    | ErrorKind::Paywall
            ) {
                return true;
            }

            // Some JS-heavy sites return synthetic 404/403 variants through edge layers.
            se.message.contains("HTTP 404")
                || se.message.contains("HTTP 403")
                || se.message.contains("HTTP 429")
        }
        _ => false,
    }
}

fn prepare_summary_input(content: &str, length: SummaryLength) -> String {
    let max_chars = length.max_input_chars();
    let content_chars = content.chars().count();
    if content_chars <= max_chars {
        return content.to_string();
    }
    let head_chars = (max_chars * 7) / 10;
    let tail_chars = max_chars.saturating_sub(head_chars);
    let head = trim_to_chars(content, head_chars);
    let tail = trim_from_end(content, tail_chars);
    format!("{head}\n\n[... middle omitted for brevity ...]\n\n{tail}")
}

fn trim_to_chars(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}

fn trim_from_end(s: &str, max_chars: usize) -> String {
    let count = s.chars().count();
    if count <= max_chars {
        return s.to_string();
    }
    s.chars().skip(count - max_chars).collect()
}

fn grounding_score(answer: &str, source: &str) -> f64 {
    let src = source.to_lowercase();
    let mut total = 0usize;
    let mut supported = 0usize;

    for sentence in answer.split(['.', '!', '?']) {
        let terms = sentence
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| t.len() >= 4)
            .map(|t| t.to_lowercase())
            .collect::<Vec<_>>();
        if terms.is_empty() {
            continue;
        }
        total += 1;
        let hits = terms.iter().filter(|t| src.contains(t.as_str())).count();
        if hits >= 2 {
            supported += 1;
        }
    }

    if total == 0 {
        0.0
    } else {
        supported as f64 / total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_length_descriptions() {
        assert!(SummaryLength::Short.description().contains("100"));
        assert!(SummaryLength::Medium.description().contains("300"));
        assert!(SummaryLength::Long.description().contains("700"));
    }

    #[test]
    fn default_config() {
        let config = SummarizeConfig::default();
        assert_eq!(config.length, SummaryLength::Medium);
        assert!(config.model.is_none());
    }

    #[tokio::test]
    async fn summarize_raw_text() {
        let text = "Rust is a systems language. It ensures memory safety. It has zero-cost abstractions. It supports async/await. It has great tooling.";
        let config = SummarizeConfig {
            length: SummaryLength::Short,
            model: None,
        };
        let fetchium_config = FetchiumConfig::default();
        let result = summarize(text, &config, &fetchium_config).await.unwrap();
        assert!(!result.summary.is_empty());
        assert!(result.source_url.is_none());
    }

    #[tokio::test]
    async fn summarize_empty_text() {
        let config = SummarizeConfig::default();
        let fetchium_config = FetchiumConfig::default();
        let result = summarize("", &config, &fetchium_config).await.unwrap();
        assert!(result.summary.is_empty());
        assert!(!result.ai_used);
    }

    #[test]
    fn prepare_summary_input_truncates_long_content() {
        let long = "a".repeat(40_000);
        let out = prepare_summary_input(&long, SummaryLength::Medium);
        assert!(out.len() < long.len());
        assert!(out.contains("[... middle omitted for brevity ...]"));
    }

    #[test]
    fn grounding_score_prefers_supported_text() {
        let src = "Rust has ownership and borrowing for memory safety.";
        let good = "Rust uses ownership and borrowing for memory safety.";
        let bad = "Rust runs on quantum crystals.";
        assert!(grounding_score(good, src) > grounding_score(bad, src));
    }

    #[test]
    fn is_block_like_fetch_error_detects_403() {
        let err = FetchiumError::Structured(crate::error::StructuredError {
            kind: ErrorKind::Http403,
            retryable: false,
            message: "blocked".into(),
            source_url: Some("https://example.com".into()),
            suggested_action: "fallback".into(),
            alternatives: vec![],
        });
        assert!(is_block_like_fetch_error(&err));
    }

    #[test]
    fn is_block_like_fetch_error_detects_unknown_404() {
        let err = FetchiumError::Structured(crate::error::StructuredError {
            kind: ErrorKind::Unknown,
            retryable: false,
            message: "HTTP 404 Not Found from https://example.com".into(),
            source_url: Some("https://example.com".into()),
            suggested_action: "fallback".into(),
            alternatives: vec![],
        });
        assert!(is_block_like_fetch_error(&err));
    }

    #[tokio::test]
    async fn has_reachable_ai_provider_false_when_unconfigured() {
        let fetchium_config = FetchiumConfig::default();
        let mut ai = AiConfig::from_fetchium_config(&fetchium_config);
        ai.providers.fallback_chain.clear();
        ai.providers.ollama.enabled = false;
        ai.providers.openai.enabled = false;
        ai.providers.anthropic.enabled = false;
        ai.providers.gemini.enabled = false;
        ai.providers.gemini_cli.enabled = false;
        ai.providers.openrouter.enabled = false;
        ai.providers.antigravity.enabled = false;
        assert!(!has_reachable_ai_provider(&ai, &fetchium_config).await);
    }
}
