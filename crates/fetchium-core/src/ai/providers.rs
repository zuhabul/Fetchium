//! Multi-provider AI configuration and types.
//!
//! Supported providers:
//! - `ollama`        — local models via Ollama HTTP API (no API key)
//! - `openai`        — OpenAI chat completions API (API key or Codex CLI OAuth)
//! - `anthropic`     — Anthropic Messages API / Claude (API key or Claude Code OAuth)
//! - `gemini`        — Google Gemini REST API (API key or Gemini CLI OAuth)
//! - `gemini_cli`    — Local `gemini` CLI subprocess (Gemini subscription)
//! - `openrouter`    — OpenRouter aggregator — 100+ models, one API key
//! - `antigravity`   — OpenCode Antigravity (Google Cloud Code Assist proxy:
//!   free Gemini 3 + Claude Sonnet/Opus via `opencode-antigravity-auth`)

use serde::{Deserialize, Serialize};

// ─── Model Registry ──────────────────────────────────────────────────────────
//
// **Single source of truth for all model names in Fetchium.**
// To add or change a model: update `ModelRegistry::models_for()` and, if it
// becomes the new default, update `ModelRegistry::default_model()`.
// No other file should ever hardcode a model string.

/// Model capability tier for routing and display purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelCapability {
    /// Sub-500 ms, minimal cost — HyDE, intent classification, autocomplete.
    Fast,
    /// Default for user-facing queries — good quality/speed balance.
    Standard,
    /// Highest quality, slower and costlier — deep research synthesis.
    Powerful,
}

impl ModelCapability {
    /// Short label for display (`"fast"`, `"standard"`, `"powerful"`).
    pub fn label(&self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Standard => "standard",
            Self::Powerful => "powerful",
        }
    }
}

/// Metadata for a single model available to a provider.
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Canonical model ID sent to the API or CLI.
    pub id: &'static str,
    /// Capability tier — controls default routing behavior.
    pub capability: ModelCapability,
    /// Short aliases accepted by `hsx provider set <provider> --model <alias>`.
    pub aliases: &'static [&'static str],
    /// One-line description shown in `hsx provider models`.
    pub note: &'static str,
}

/// **Central model registry** — the one place where all model names live.
///
/// # Usage
/// - `ModelRegistry::default_model(kind)` — provider's default model ID.
/// - `ModelRegistry::fast_model(kind)`    — low-latency model for HyDE / intent.
/// - `ModelRegistry::models_for(kind)`   — full list for tab-complete / display.
/// - `ModelRegistry::resolve_alias(kind, s)` — expand `"flash"` → full model ID.
pub struct ModelRegistry;

impl ModelRegistry {
    /// Canonical default model for a provider.
    ///
    /// **This is the only place default model strings are defined.**
    /// All other code (`ProviderKind::default_model`, `ProviderEntry::resolve_model`,
    /// `call_gemini_cli`, etc.) ultimately resolves through here.
    pub fn default_model(kind: ProviderKind) -> &'static str {
        match kind {
            ProviderKind::GeminiCli => "gemini-3-flash-preview", // Gemini 3 Flash — thinking-capable
            ProviderKind::Gemini => "gemini-3-flash-preview",    // Gemini 3 Flash (latest)
            ProviderKind::Anthropic => "claude-haiku-4-5-20251001", // Fast, affordable
            ProviderKind::OpenAi => "gpt-4o-mini",               // Fast, affordable
            ProviderKind::Ollama => "qwen3:8b",                  // Good local default
            ProviderKind::OpenRouter => "google/gemini-2.5-flash", // Via OpenRouter (2.5 Flash still most stable there)
            ProviderKind::Antigravity => "antigravity-gemini-3-flash", // Free via OpenCode
        }
    }

    /// Fast / low-latency model for HyDE queries and intent classification.
    pub fn fast_model(kind: ProviderKind) -> &'static str {
        match kind {
            ProviderKind::GeminiCli => "gemini-3-flash-preview", // Thinking-capable for fast mode override
            ProviderKind::Gemini => "gemini-3-flash-preview",
            ProviderKind::Anthropic => "claude-haiku-4-5-20251001",
            ProviderKind::OpenAi => "gpt-4o-mini",
            ProviderKind::Ollama => "qwen3:0.6b",
            ProviderKind::OpenRouter => "google/gemini-2.5-flash",
            ProviderKind::Antigravity => "antigravity-gemini-3-flash",
        }
    }

    /// Resolve a short alias or exact model ID to a canonical model ID.
    ///
    /// Returns `None` when the alias is unrecognized — callers should treat
    /// the input as a literal model ID (passed through unchanged).
    pub fn resolve_alias(kind: ProviderKind, alias: &str) -> Option<&'static str> {
        Self::models_for(kind)
            .iter()
            .find(|m| m.id == alias || m.aliases.contains(&alias))
            .map(|m| m.id)
    }

    /// All known models for a provider, ordered fast → standard → powerful.
    ///
    /// Used by `hsx provider models` and alias resolution.
    pub fn models_for(kind: ProviderKind) -> &'static [ModelInfo] {
        match kind {
            ProviderKind::GeminiCli => &[
                ModelInfo {
                    id: "gemini-3-flash-preview",
                    capability: ModelCapability::Standard,
                    aliases: &["flash3", "flash", "g3flash", "gemini3flash"],
                    note: "Gemini 3 Flash — newest, thinking-capable (default)",
                },
                ModelInfo {
                    id: "gemini-2.5-flash",
                    capability: ModelCapability::Fast,
                    aliases: &["flash25", "fast", "2.5flash", "flash2.5"],
                    note: "Gemini 2.5 Flash — ~5x faster, no thinking. Use with --fast flag.",
                },
                ModelInfo {
                    id: "gemini-3-pro-preview",
                    capability: ModelCapability::Powerful,
                    aliases: &["pro3", "pro", "g3pro", "gemini3pro"],
                    note: "Gemini 3 Pro — most capable (preview, may 429 under load)",
                },
            ],
            ProviderKind::Gemini => &[
                ModelInfo {
                    id: "gemini-3-flash-preview",
                    capability: ModelCapability::Standard,
                    aliases: &["flash3", "g3flash", "gemini3flash", "flash"],
                    note: "Gemini 3 Flash Preview — latest generation (default)",
                },
                ModelInfo {
                    id: "gemini-2.5-flash",
                    capability: ModelCapability::Standard,
                    aliases: &["flash25", "flash2.5"],
                    note: "Gemini 2.5 Flash — previous generation stable model",
                },
                ModelInfo {
                    id: "gemini-2.5-pro",
                    capability: ModelCapability::Powerful,
                    aliases: &["pro", "pro25"],
                    note: "Gemini 2.5 Pro — highest REST API quality",
                },
                ModelInfo {
                    id: "gemini-2.5-flash-thinking-exp",
                    capability: ModelCapability::Powerful,
                    aliases: &["thinking", "flash-thinking"],
                    note: "Gemini 2.5 Flash Thinking — extended reasoning",
                },
                ModelInfo {
                    id: "gemini-2.0-flash",
                    capability: ModelCapability::Fast,
                    aliases: &["flash20", "2.0flash"],
                    note: "Gemini 2.0 Flash — fast, lower cost",
                },
            ],
            ProviderKind::Anthropic => &[
                ModelInfo {
                    id: "claude-haiku-4-5-20251001",
                    capability: ModelCapability::Fast,
                    aliases: &["haiku", "claude-haiku"],
                    note: "Claude Haiku — fastest, lowest cost (default)",
                },
                ModelInfo {
                    id: "claude-sonnet-4-6",
                    capability: ModelCapability::Standard,
                    aliases: &["sonnet", "claude-sonnet"],
                    note: "Claude Sonnet — best quality/speed balance",
                },
                ModelInfo {
                    id: "claude-opus-4-6",
                    capability: ModelCapability::Powerful,
                    aliases: &["opus", "claude-opus"],
                    note: "Claude Opus — most capable, highest cost",
                },
            ],
            ProviderKind::OpenAi => &[
                ModelInfo {
                    id: "gpt-4o-mini",
                    capability: ModelCapability::Fast,
                    aliases: &["mini", "4o-mini"],
                    note: "GPT-4o Mini — fast, affordable (default)",
                },
                ModelInfo {
                    id: "gpt-4o",
                    capability: ModelCapability::Standard,
                    aliases: &["4o", "gpt4o"],
                    note: "GPT-4o — flagship multimodal model",
                },
                ModelInfo {
                    id: "o3-mini",
                    capability: ModelCapability::Powerful,
                    aliases: &["o3mini", "o3"],
                    note: "o3-mini — advanced reasoning model",
                },
            ],
            ProviderKind::Ollama => &[
                ModelInfo {
                    id: "qwen3:0.6b",
                    capability: ModelCapability::Fast,
                    aliases: &["tiny", "qwen-tiny"],
                    note: "Qwen3 0.6B — ultra-fast, ~0.5 GB RAM",
                },
                ModelInfo {
                    id: "qwen3:8b",
                    capability: ModelCapability::Standard,
                    aliases: &["qwen3"],
                    note: "Qwen3 8B — good local default (~5 GB RAM)",
                },
                ModelInfo {
                    id: "qwen3:30b-a3b",
                    capability: ModelCapability::Powerful,
                    aliases: &["qwen3-large"],
                    note: "Qwen3 30B MoE — high quality (~8 GB RAM, sparse)",
                },
                ModelInfo {
                    id: "gemma3:1b",
                    capability: ModelCapability::Fast,
                    aliases: &["gemma-tiny", "gemma1b"],
                    note: "Gemma3 1B — Google compact model",
                },
                ModelInfo {
                    id: "gemma3:27b",
                    capability: ModelCapability::Powerful,
                    aliases: &["gemma"],
                    note: "Gemma3 27B — Google powerful model (~20 GB RAM)",
                },
                ModelInfo {
                    id: "llama3.3:70b",
                    capability: ModelCapability::Powerful,
                    aliases: &["llama"],
                    note: "Llama 3.3 70B — Meta flagship (~40 GB RAM required)",
                },
            ],
            ProviderKind::OpenRouter => &[
                ModelInfo {
                    id: "google/gemini-2.5-flash",
                    capability: ModelCapability::Standard,
                    aliases: &["gemini-flash", "gflash"],
                    note: "Gemini 2.5 Flash via OpenRouter (default)",
                },
                ModelInfo {
                    id: "anthropic/claude-haiku-4-5",
                    capability: ModelCapability::Fast,
                    aliases: &["claude-haiku"],
                    note: "Claude Haiku via OpenRouter",
                },
                ModelInfo {
                    id: "anthropic/claude-sonnet-4-6",
                    capability: ModelCapability::Standard,
                    aliases: &["claude-sonnet"],
                    note: "Claude Sonnet via OpenRouter",
                },
                ModelInfo {
                    id: "openai/gpt-4o-mini",
                    capability: ModelCapability::Fast,
                    aliases: &["gpt-mini"],
                    note: "GPT-4o Mini via OpenRouter",
                },
                ModelInfo {
                    id: "mistralai/mistral-small",
                    capability: ModelCapability::Fast,
                    aliases: &["mistral"],
                    note: "Mistral Small via OpenRouter",
                },
            ],
            ProviderKind::Antigravity => &[
                ModelInfo {
                    id: "antigravity-gemini-3-flash",
                    capability: ModelCapability::Standard,
                    aliases: &["ag-flash", "ag-gemini"],
                    note: "Gemini 3 Flash via Antigravity/OpenCode (default)",
                },
                ModelInfo {
                    id: "antigravity-gemini-3-pro",
                    capability: ModelCapability::Powerful,
                    aliases: &["ag-pro", "ag-gemini-pro"],
                    note: "Gemini 3 Pro via Antigravity/OpenCode",
                },
                ModelInfo {
                    id: "antigravity-claude-sonnet",
                    capability: ModelCapability::Standard,
                    aliases: &["ag-sonnet", "ag-claude"],
                    note: "Claude Sonnet via Antigravity/OpenCode",
                },
                ModelInfo {
                    id: "antigravity-claude-opus",
                    capability: ModelCapability::Powerful,
                    aliases: &["ag-opus"],
                    note: "Claude Opus via Antigravity/OpenCode",
                },
            ],
        }
    }
}

/// Which AI backend to route requests to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    /// Ollama local models (no API key required).
    Ollama,
    /// OpenAI chat completions API (API key or Codex CLI OAuth).
    OpenAi,
    /// Anthropic Messages API — Claude models (API key or Claude Code OAuth).
    Anthropic,
    /// Google Gemini REST API (API key or Gemini CLI OAuth).
    Gemini,
    /// Local `gemini` CLI subprocess (requires `gemini` in PATH + Gemini subscription).
    GeminiCli,
    /// OpenRouter aggregator — access 100+ models with one API key.
    OpenRouter,
    /// OpenCode Antigravity — Google Cloud Code Assist proxy.
    ///
    /// Provides free access to Gemini 3 (Pro, Flash) and Claude Sonnet/Opus models
    /// using credentials from the `opencode-antigravity-auth` plugin.
    /// Credentials are stored in `~/.config/opencode/antigravity-accounts.json`.
    Antigravity,
}

impl ProviderKind {
    /// Human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Ollama => "Ollama (local)",
            Self::OpenAi => "OpenAI",
            Self::Anthropic => "Anthropic / Claude",
            Self::Gemini => "Google Gemini",
            Self::GeminiCli => "Gemini CLI (local)",
            Self::OpenRouter => "OpenRouter",
            Self::Antigravity => "Antigravity (OpenCode)",
        }
    }

    /// Default model ID for this provider.
    ///
    /// Delegates to [`ModelRegistry::default_model`] — the single source of truth.
    /// To change a default, update the registry rather than this method.
    pub fn default_model(&self) -> &'static str {
        ModelRegistry::default_model(*self)
    }

    /// Parse from a string slug (case-insensitive, accepts common aliases).
    pub fn from_slug(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ollama" => Some(Self::Ollama),
            "openai" | "open-ai" | "open_ai" => Some(Self::OpenAi),
            "anthropic" | "claude" => Some(Self::Anthropic),
            "gemini" | "google" | "google-gemini" => Some(Self::Gemini),
            "gemini-cli" | "gemini_cli" | "geminicli" => Some(Self::GeminiCli),
            "openrouter" | "open-router" | "open_router" => Some(Self::OpenRouter),
            "antigravity" | "anti-gravity" | "anti_gravity" | "opencode" | "open-code"
            | "open_code" => Some(Self::Antigravity),
            _ => None,
        }
    }

    /// Canonical config key slug used in TOML and the CLI.
    pub fn slug(&self) -> &'static str {
        match self {
            Self::Ollama => "ollama",
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
            Self::GeminiCli => "gemini_cli",
            Self::OpenRouter => "openrouter",
            Self::Antigravity => "antigravity",
        }
    }

    /// Environment variable name for the API key (`None` for no-key providers).
    ///
    /// Note: Antigravity uses OAuth rather than an API key, so this returns `None`.
    pub fn api_key_env(&self) -> Option<&'static str> {
        match self {
            Self::Ollama | Self::GeminiCli | Self::Antigravity => None,
            Self::OpenAi => Some("OPENAI_API_KEY"),
            Self::Anthropic => Some("ANTHROPIC_API_KEY"),
            Self::Gemini => Some("GEMINI_API_KEY"),
            Self::OpenRouter => Some("OPENROUTER_API_KEY"),
        }
    }

    /// Whether this provider requires an explicit API key to operate.
    ///
    /// Returns `false` for local providers and OAuth-only providers (Antigravity).
    pub fn requires_api_key(&self) -> bool {
        self.api_key_env().is_some()
    }

    /// All canonical provider slugs (for help text and tab completion).
    pub fn all_slugs() -> &'static [&'static str] {
        &[
            "ollama",
            "openai",
            "anthropic",
            "gemini",
            "gemini_cli",
            "openrouter",
            "antigravity",
        ]
    }

    /// Link to the API key management page for this provider (`None` if no key needed).
    pub fn api_docs_url(&self) -> Option<&'static str> {
        match self {
            Self::OpenAi => Some("https://platform.openai.com/api-keys"),
            Self::Anthropic => Some("https://console.anthropic.com/settings/keys"),
            Self::Gemini => Some("https://aistudio.google.com/app/apikey"),
            Self::OpenRouter => Some("https://openrouter.ai/keys"),
            _ => None,
        }
    }

    /// Short note about how this provider authenticates (for help text).
    pub fn auth_note(&self) -> &'static str {
        match self {
            Self::Ollama => "local binary, no key",
            Self::OpenAi => "API key or Codex CLI OAuth (ChatGPT subscription)",
            Self::Anthropic => "API key or Claude Code OAuth (Max/Pro subscription)",
            Self::Gemini => "API key or Gemini CLI OAuth (Gemini subscription)",
            Self::GeminiCli => "local binary + Gemini subscription",
            Self::OpenRouter => "API key (openrouter.ai)",
            Self::Antigravity => "OAuth via opencode-antigravity-auth — no key needed",
        }
    }
}

impl std::fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.display_name())
    }
}

/// Per-provider configuration stored in `~/.fetchium/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProviderEntry {
    /// API key stored in plain text.
    ///
    /// Prefer setting the corresponding environment variable instead
    /// (e.g. `OPENAI_API_KEY`) to avoid storing secrets in config files.
    pub api_key: Option<String>,
    /// Additional API keys for pool rotation.
    /// All keys in this array plus `api_key` are combined into a single pool.
    /// Keys are rotated with rate-limit awareness — if one key gets 429'd,
    /// the next key is tried automatically.
    #[serde(default)]
    pub api_keys: Vec<String>,
    /// Model name override. Uses the provider default when absent.
    pub model: Option<String>,
    /// Base URL override (useful for proxies or custom deployments).
    pub base_url: Option<String>,
    /// Whether this entry is active in fallback resolution.
    pub enabled: bool,
}

impl Default for ProviderEntry {
    fn default() -> Self {
        Self {
            api_key: None,
            api_keys: Vec::new(),
            model: None,
            base_url: None,
            enabled: true,
        }
    }
}

impl ProviderEntry {
    /// Resolve the API key: config value first, then environment variable.
    ///
    /// Returns `None` if neither source has a non-empty, non-placeholder value.
    pub fn resolve_api_key(&self, env_var: &str) -> Option<String> {
        if let Some(ref k) = self.api_key {
            if !k.is_empty() && !k.starts_with("your-") && k != "REPLACE_ME" {
                return Some(k.clone());
            }
        }
        std::env::var(env_var).ok().filter(|v| !v.is_empty())
    }

    /// Resolve the model, falling back to the provider's built-in default.
    ///
    /// Supports short aliases (e.g. `"flash"` → `"gemini-3-flash-preview"` for
    /// `gemini_cli`, `"haiku"` → `"claude-haiku-4-5-20251001"` for `anthropic`).
    /// Any unrecognized string is treated as a literal model ID and passed through.
    pub fn resolve_model(&self, kind: ProviderKind) -> String {
        if let Some(ref m) = self.model {
            if !m.is_empty() {
                return ModelRegistry::resolve_alias(kind, m)
                    .map(str::to_string)
                    .unwrap_or_else(|| m.clone());
            }
        }
        ModelRegistry::default_model(kind).to_string()
    }

    /// Return `true` if this entry appears to have a usable API key.
    pub fn has_key(&self, env_var: &str) -> bool {
        self.resolve_api_key(env_var).is_some()
    }
}

/// Full multi-provider configuration block (`[ai.providers]` in config.toml).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ProvidersConfig {
    /// Ordered list of provider slugs to try (first = highest priority).
    ///
    /// If a provider fails (bad key, network error, etc.), the next one in
    /// the list is tried automatically.
    ///
    /// Example: `["antigravity", "gemini", "anthropic", "openai", "ollama"]`
    ///
    /// Defaults to `["ollama"]` for backward compatibility when empty.
    pub fallback_chain: Vec<String>,

    /// Ollama local model configuration.
    pub ollama: ProviderEntry,
    /// OpenAI API configuration.
    pub openai: ProviderEntry,
    /// Anthropic (Claude) API configuration.
    pub anthropic: ProviderEntry,
    /// Google Gemini API configuration.
    pub gemini: ProviderEntry,
    /// Gemini CLI subprocess configuration.
    pub gemini_cli: ProviderEntry,
    /// OpenRouter API configuration.
    pub openrouter: ProviderEntry,
    /// OpenCode Antigravity OAuth configuration (model override only — no API key needed).
    pub antigravity: ProviderEntry,
}

impl ProvidersConfig {
    /// Immutable reference to the entry for a given provider kind.
    pub fn entry(&self, kind: ProviderKind) -> &ProviderEntry {
        match kind {
            ProviderKind::Ollama => &self.ollama,
            ProviderKind::OpenAi => &self.openai,
            ProviderKind::Anthropic => &self.anthropic,
            ProviderKind::Gemini => &self.gemini,
            ProviderKind::GeminiCli => &self.gemini_cli,
            ProviderKind::OpenRouter => &self.openrouter,
            ProviderKind::Antigravity => &self.antigravity,
        }
    }

    /// Mutable reference to the entry for a given provider kind.
    pub fn entry_mut(&mut self, kind: ProviderKind) -> &mut ProviderEntry {
        match kind {
            ProviderKind::Ollama => &mut self.ollama,
            ProviderKind::OpenAi => &mut self.openai,
            ProviderKind::Anthropic => &mut self.anthropic,
            ProviderKind::Gemini => &mut self.gemini,
            ProviderKind::GeminiCli => &mut self.gemini_cli,
            ProviderKind::OpenRouter => &mut self.openrouter,
            ProviderKind::Antigravity => &mut self.antigravity,
        }
    }

    /// Resolve the ordered fallback chain as `ProviderKind` values.
    ///
    /// Filters out unknown slugs and disabled entries, then **auto-appends** any
    /// subscription-auth providers that are locally available but not in the explicit chain.
    ///
    /// Auto-discovery order (appended only when not already present):
    /// 1. **Antigravity** — OpenCode account at `~/.config/opencode/antigravity-accounts.json`
    /// 2. **GeminiCli** — Gemini OAuth session (subprocess works even when REST API scope is
    ///    insufficient — covers `gemini auth login` tokens)
    /// 3. **OpenAI** — Codex CLI session at `~/.codex/auth.json` (ChatGPT subscription)
    /// 4. **Anthropic** — Claude Code session (Agent SDK / Claude Max/Pro subscription).
    ///    If the session is rejected by `api.anthropic.com` the error is logged as a warning
    ///    and the chain continues — this is a graceful fallback.
    ///
    /// Falls back to `[Ollama]` when no chain is configured and nothing is discovered.
    pub fn resolved_chain(&self) -> Vec<ProviderKind> {
        use crate::ai::credentials::{
            antigravity_auth_available, codex_auth_available, read_gemini_creds,
        };

        // Build the explicit user-configured chain first.
        let mut chain: Vec<ProviderKind> = self
            .fallback_chain
            .iter()
            .filter_map(|s| ProviderKind::from_slug(s))
            .filter(|k| self.entry(*k).enabled)
            .collect();

        // Auto-append subscription-auth providers not already in the chain.
        //
        // Gemini REST API is auto-prepended when any API key is available
        // (env var, config.toml, or auth store pool) — it's the fastest path.
        //
        // GeminiCli is also auto-discovered via `gemini auth login` sessions,
        // as a fallback when keys aren't configured.
        //
        // NOTE: Anthropic is only auto-added when a real API key is present.
        let gemini_session = read_gemini_creds()
            .map(|c| c.is_refreshable())
            .unwrap_or(false);
        let has_gemini_api_key = {
            let entry = self.entry(ProviderKind::Gemini);
            let from_config = entry
                .api_key
                .as_ref()
                .map(|k| !k.is_empty() && !k.starts_with("your-") && k != "REPLACE_ME")
                .unwrap_or(false);
            let from_env = std::env::var("GEMINI_API_KEY")
                .map(|k| !k.is_empty())
                .unwrap_or(false);
            let from_multi_env = std::env::var("GEMINI_API_KEYS")
                .map(|k| !k.is_empty())
                .unwrap_or(false);
            let from_auth = crate::ai::credentials::hsx_auth_get("gemini")
                .map(|a| a.key_count() > 0)
                .unwrap_or(false);
            from_config || from_env || from_multi_env || from_auth
        };
        let has_anthropic_key = self
            .entry(ProviderKind::Anthropic)
            .resolve_api_key("ANTHROPIC_API_KEY")
            .is_some();
        let auto_fallbacks: [(ProviderKind, bool); 5] = [
            // Gemini REST API — fastest when keys are present
            (ProviderKind::Gemini, has_gemini_api_key),
            (ProviderKind::Antigravity, antigravity_auth_available()),
            (ProviderKind::GeminiCli, gemini_session),
            (ProviderKind::OpenAi, codex_auth_available()),
            (ProviderKind::Anthropic, has_anthropic_key),
        ];
        for (kind, available) in auto_fallbacks {
            if available && self.entry(kind).enabled && !chain.contains(&kind) {
                chain.push(kind);
            }
        }

        if chain.is_empty() {
            vec![ProviderKind::Ollama]
        } else {
            chain
        }
    }

    /// Return all providers that appear to be usable (key set or local/OAuth).
    pub fn configured_providers(&self) -> Vec<ProviderKind> {
        use crate::ai::credentials::{
            antigravity_auth_available, claude_code_auth_available, codex_auth_available,
            get_gemini_access_token_if_valid, read_gemini_creds,
        };
        // Note: `read_gemini_creds().is_some()` is intentionally NOT used as a signal —
        // a creds file can exist with an empty refresh_token after `invalidate_gemini_creds()`.
        // Use `is_refreshable()` to distinguish a live session from a dead one.
        const ALL: &[ProviderKind] = &[
            ProviderKind::Ollama,
            ProviderKind::OpenAi,
            ProviderKind::Anthropic,
            ProviderKind::Gemini,
            ProviderKind::GeminiCli,
            ProviderKind::OpenRouter,
            ProviderKind::Antigravity,
        ];
        ALL.iter()
            .copied()
            .filter(|k| {
                let entry = self.entry(*k);
                if !entry.enabled {
                    return false;
                }
                match k {
                    // OAuth-capable: API key OR subscription session
                    ProviderKind::OpenAi => {
                        entry.has_key("OPENAI_API_KEY") || codex_auth_available()
                    }
                    ProviderKind::Anthropic => {
                        entry.has_key("ANTHROPIC_API_KEY") || claude_code_auth_available()
                    }
                    ProviderKind::Gemini => {
                        entry.has_key("GEMINI_API_KEY")
                            || std::env::var("GEMINI_API_KEYS")
                                .map(|k| !k.is_empty())
                                .unwrap_or(false)
                            || crate::ai::credentials::hsx_auth_get("gemini")
                                .map(|a| a.key_count() > 0)
                                .unwrap_or(false)
                            || get_gemini_access_token_if_valid().is_some()
                            || read_gemini_creds()
                                .map(|c| c.is_refreshable())
                                .unwrap_or(false)
                    }
                    // OAuth-only (no API key)
                    ProviderKind::Antigravity => antigravity_auth_available(),
                    // Local — always usable (availability checked at runtime)
                    ProviderKind::Ollama | ProviderKind::GeminiCli => true,
                    // API key only
                    ProviderKind::OpenRouter => entry.has_key("OPENROUTER_API_KEY"),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_slug_roundtrip() {
        assert_eq!(
            ProviderKind::from_slug("openai"),
            Some(ProviderKind::OpenAi)
        );
        assert_eq!(
            ProviderKind::from_slug("claude"),
            Some(ProviderKind::Anthropic)
        );
        assert_eq!(
            ProviderKind::from_slug("gemini"),
            Some(ProviderKind::Gemini)
        );
        assert_eq!(
            ProviderKind::from_slug("gemini_cli"),
            Some(ProviderKind::GeminiCli)
        );
        assert_eq!(
            ProviderKind::from_slug("openrouter"),
            Some(ProviderKind::OpenRouter)
        );
        assert_eq!(
            ProviderKind::from_slug("ollama"),
            Some(ProviderKind::Ollama)
        );
        assert_eq!(
            ProviderKind::from_slug("antigravity"),
            Some(ProviderKind::Antigravity)
        );
        assert_eq!(
            ProviderKind::from_slug("opencode"),
            Some(ProviderKind::Antigravity)
        );
        assert_eq!(ProviderKind::from_slug("bogus"), None);
    }

    #[test]
    fn resolved_chain_defaults_to_ollama() {
        let cfg = ProvidersConfig::default();
        let chain = cfg.resolved_chain();
        // Chain must always be non-empty.  When no explicit chain is configured and
        // no subscription sessions are present the result is [Ollama].  On machines
        // with a Gemini session / Antigravity / Codex CLI, those providers appear
        // instead.  Either way, all entries must be valid ProviderKind values.
        assert!(!chain.is_empty());
        for k in &chain {
            assert!(ProviderKind::all_slugs().contains(&k.slug()));
        }
    }

    #[test]
    fn resolved_chain_respects_order() {
        let cfg = ProvidersConfig {
            fallback_chain: vec!["gemini".into(), "openai".into(), "ollama".into()],
            ..ProvidersConfig::default()
        };
        let chain = cfg.resolved_chain();
        // The explicit entries must appear first and in order.
        assert_eq!(chain[0], ProviderKind::Gemini);
        assert_eq!(chain[1], ProviderKind::OpenAi);
        assert_eq!(chain[2], ProviderKind::Ollama);
        // Auto-discovered subscription providers may be appended after index 2.
    }

    #[test]
    fn resolved_chain_skips_disabled() {
        let mut cfg = ProvidersConfig {
            fallback_chain: vec!["openai".into(), "ollama".into()],
            ..ProvidersConfig::default()
        };
        cfg.openai.enabled = false;
        let chain = cfg.resolved_chain();
        // Disabled provider must not appear in the chain, even as an auto-fallback.
        assert!(!chain.contains(&ProviderKind::OpenAi));
        // Ollama (explicitly added and enabled) must be present.
        assert!(chain.contains(&ProviderKind::Ollama));
    }

    #[test]
    fn entry_ref_all_kinds() {
        let cfg = ProvidersConfig::default();
        for k in ProviderKind::all_slugs() {
            let kind = ProviderKind::from_slug(k).unwrap();
            let _ = cfg.entry(kind);
            // All slugs should round-trip correctly
            assert_eq!(kind.slug(), *k);
        }
    }

    #[test]
    fn resolve_api_key_prefers_config() {
        let entry = ProviderEntry {
            api_key: Some("cfg-key".into()),
            ..ProviderEntry::default()
        };
        // Even if env var is set, config key wins
        let resolved = entry.resolve_api_key("NONEXISTENT_TEST_VAR_XYZ");
        assert_eq!(resolved, Some("cfg-key".into()));
    }

    #[test]
    fn resolve_model_uses_default_when_unset() {
        let entry = ProviderEntry::default();
        assert_eq!(
            entry.resolve_model(ProviderKind::Gemini),
            "gemini-3-flash-preview"
        );
        // GeminiCli now defaults to gemini-3-flash-preview (Gemini 3 Flash)
        assert_eq!(
            entry.resolve_model(ProviderKind::GeminiCli),
            "gemini-3-flash-preview"
        );
    }

    #[test]
    fn resolve_model_alias_expansion() {
        // "flash" → gemini-3-flash-preview for GeminiCli
        let mut entry = ProviderEntry {
            model: Some("flash".into()),
            ..ProviderEntry::default()
        };
        assert_eq!(
            entry.resolve_model(ProviderKind::GeminiCli),
            "gemini-3-flash-preview"
        );
        // "flash" → gemini-3-flash-preview for Gemini REST (now added as alias)
        assert_eq!(
            entry.resolve_model(ProviderKind::Gemini),
            "gemini-3-flash-preview"
        );
        // "flash" is not a known Anthropic alias — passed through as literal
        assert_eq!(entry.resolve_model(ProviderKind::Anthropic), "flash");

        // "haiku" → full Anthropic model ID
        entry.model = Some("haiku".into());
        assert_eq!(
            entry.resolve_model(ProviderKind::Anthropic),
            "claude-haiku-4-5-20251001"
        );

        // "sonnet" → Claude Sonnet
        entry.model = Some("sonnet".into());
        assert_eq!(
            entry.resolve_model(ProviderKind::Anthropic),
            "claude-sonnet-4-6"
        );
    }

    #[test]
    fn resolve_model_unknown_alias_passes_through() {
        let entry = ProviderEntry {
            model: Some("my-custom-model:latest".into()),
            ..ProviderEntry::default()
        };
        assert_eq!(
            entry.resolve_model(ProviderKind::Ollama),
            "my-custom-model:latest"
        );
    }

    #[test]
    fn model_registry_all_providers_have_default() {
        for slug in ProviderKind::all_slugs() {
            let kind = ProviderKind::from_slug(slug).unwrap();
            let default = ModelRegistry::default_model(kind);
            assert!(
                !default.is_empty(),
                "Provider {slug} has empty default model"
            );
            // Default must appear in models_for() list
            let listed = ModelRegistry::models_for(kind)
                .iter()
                .any(|m| m.id == default);
            assert!(
                listed,
                "Provider {slug} default model '{default}' not in models_for() list"
            );
        }
    }

    #[test]
    fn model_registry_no_duplicate_aliases() {
        for slug in ProviderKind::all_slugs() {
            let kind = ProviderKind::from_slug(slug).unwrap();
            let mut seen = std::collections::HashSet::new();
            for model in ModelRegistry::models_for(kind) {
                for alias in model.aliases {
                    assert!(
                        seen.insert(*alias),
                        "Provider {slug}: duplicate alias '{alias}'"
                    );
                }
            }
        }
    }
}
