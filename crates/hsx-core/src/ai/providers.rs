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
            Self::Ollama       => "Ollama (local)",
            Self::OpenAi       => "OpenAI",
            Self::Anthropic    => "Anthropic / Claude",
            Self::Gemini       => "Google Gemini",
            Self::GeminiCli    => "Gemini CLI (local)",
            Self::OpenRouter   => "OpenRouter",
            Self::Antigravity  => "Antigravity (OpenCode)",
        }
    }

    /// Default model ID for this provider.
    pub fn default_model(&self) -> &'static str {
        match self {
            Self::Ollama       => "qwen3:8b",
            Self::OpenAi       => "gpt-4o-mini",
            Self::Anthropic    => "claude-haiku-4-5-20251001",
            Self::Gemini       => "gemini-2.0-flash",
            Self::GeminiCli    => "gemini-2.0-flash",
            Self::OpenRouter   => "google/gemini-2.0-flash-001",
            Self::Antigravity  => "antigravity-gemini-3-flash",
        }
    }

    /// Parse from a string slug (case-insensitive, accepts common aliases).
    pub fn from_slug(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ollama"                                        => Some(Self::Ollama),
            "openai" | "open-ai" | "open_ai"               => Some(Self::OpenAi),
            "anthropic" | "claude"                          => Some(Self::Anthropic),
            "gemini" | "google" | "google-gemini"          => Some(Self::Gemini),
            "gemini-cli" | "gemini_cli" | "geminicli"      => Some(Self::GeminiCli),
            "openrouter" | "open-router" | "open_router"   => Some(Self::OpenRouter),
            "antigravity" | "anti-gravity" | "anti_gravity"
            | "opencode" | "open-code" | "open_code"       => Some(Self::Antigravity),
            _ => None,
        }
    }

    /// Canonical config key slug used in TOML and the CLI.
    pub fn slug(&self) -> &'static str {
        match self {
            Self::Ollama       => "ollama",
            Self::OpenAi       => "openai",
            Self::Anthropic    => "anthropic",
            Self::Gemini       => "gemini",
            Self::GeminiCli    => "gemini_cli",
            Self::OpenRouter   => "openrouter",
            Self::Antigravity  => "antigravity",
        }
    }

    /// Environment variable name for the API key (`None` for no-key providers).
    ///
    /// Note: Antigravity uses OAuth rather than an API key, so this returns `None`.
    pub fn api_key_env(&self) -> Option<&'static str> {
        match self {
            Self::Ollama | Self::GeminiCli | Self::Antigravity => None,
            Self::OpenAi     => Some("OPENAI_API_KEY"),
            Self::Anthropic  => Some("ANTHROPIC_API_KEY"),
            Self::Gemini     => Some("GEMINI_API_KEY"),
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
        &["ollama", "openai", "anthropic", "gemini", "gemini_cli", "openrouter", "antigravity"]
    }

    /// Link to the API key management page for this provider (`None` if no key needed).
    pub fn api_docs_url(&self) -> Option<&'static str> {
        match self {
            Self::OpenAi     => Some("https://platform.openai.com/api-keys"),
            Self::Anthropic  => Some("https://console.anthropic.com/settings/keys"),
            Self::Gemini     => Some("https://aistudio.google.com/app/apikey"),
            Self::OpenRouter => Some("https://openrouter.ai/keys"),
            _ => None,
        }
    }

    /// Short note about how this provider authenticates (for help text).
    pub fn auth_note(&self) -> &'static str {
        match self {
            Self::Ollama      => "local binary, no key",
            Self::OpenAi      => "API key or Codex CLI OAuth (ChatGPT subscription)",
            Self::Anthropic   => "API key or Claude Code OAuth (Max/Pro subscription)",
            Self::Gemini      => "API key or Gemini CLI OAuth (Gemini subscription)",
            Self::GeminiCli   => "local binary + Gemini subscription",
            Self::OpenRouter  => "API key (openrouter.ai)",
            Self::Antigravity => "OAuth via opencode-antigravity-auth — no key needed",
        }
    }
}

impl std::fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.display_name())
    }
}

/// Per-provider configuration stored in `~/.hypersearchx/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProviderEntry {
    /// API key stored in plain text.
    ///
    /// Prefer setting the corresponding environment variable instead
    /// (e.g. `OPENAI_API_KEY`) to avoid storing secrets in config files.
    pub api_key: Option<String>,
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
    pub fn resolve_model(&self, kind: ProviderKind) -> String {
        self.model
            .clone()
            .filter(|m| !m.is_empty())
            .unwrap_or_else(|| kind.default_model().to_string())
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
            ProviderKind::Ollama       => &self.ollama,
            ProviderKind::OpenAi       => &self.openai,
            ProviderKind::Anthropic    => &self.anthropic,
            ProviderKind::Gemini       => &self.gemini,
            ProviderKind::GeminiCli    => &self.gemini_cli,
            ProviderKind::OpenRouter   => &self.openrouter,
            ProviderKind::Antigravity  => &self.antigravity,
        }
    }

    /// Mutable reference to the entry for a given provider kind.
    pub fn entry_mut(&mut self, kind: ProviderKind) -> &mut ProviderEntry {
        match kind {
            ProviderKind::Ollama       => &mut self.ollama,
            ProviderKind::OpenAi       => &mut self.openai,
            ProviderKind::Anthropic    => &mut self.anthropic,
            ProviderKind::Gemini       => &mut self.gemini,
            ProviderKind::GeminiCli    => &mut self.gemini_cli,
            ProviderKind::OpenRouter   => &mut self.openrouter,
            ProviderKind::Antigravity  => &mut self.antigravity,
        }
    }

    /// Resolve the ordered fallback chain as `ProviderKind` values.
    ///
    /// Filters out unknown slugs and disabled entries.
    /// Returns `[Ollama]` when no chain is configured (backward compatible).
    pub fn resolved_chain(&self) -> Vec<ProviderKind> {
        if self.fallback_chain.is_empty() {
            return vec![ProviderKind::Ollama];
        }
        self.fallback_chain
            .iter()
            .filter_map(|s| ProviderKind::from_slug(s))
            .filter(|k| self.entry(*k).enabled)
            .collect()
    }

    /// Return all providers that appear to be usable (key set or local/OAuth).
    pub fn configured_providers(&self) -> Vec<ProviderKind> {
        use crate::ai::credentials::{
            antigravity_auth_available, claude_code_auth_available,
            codex_auth_available, get_gemini_access_token_if_valid, read_gemini_creds,
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
                            || get_gemini_access_token_if_valid().is_some()
                            || read_gemini_creds().map(|c| c.is_refreshable()).unwrap_or(false)
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
        assert_eq!(ProviderKind::from_slug("openai"), Some(ProviderKind::OpenAi));
        assert_eq!(ProviderKind::from_slug("claude"), Some(ProviderKind::Anthropic));
        assert_eq!(ProviderKind::from_slug("gemini"), Some(ProviderKind::Gemini));
        assert_eq!(ProviderKind::from_slug("gemini_cli"), Some(ProviderKind::GeminiCli));
        assert_eq!(ProviderKind::from_slug("openrouter"), Some(ProviderKind::OpenRouter));
        assert_eq!(ProviderKind::from_slug("ollama"), Some(ProviderKind::Ollama));
        assert_eq!(ProviderKind::from_slug("antigravity"), Some(ProviderKind::Antigravity));
        assert_eq!(ProviderKind::from_slug("opencode"), Some(ProviderKind::Antigravity));
        assert_eq!(ProviderKind::from_slug("bogus"), None);
    }

    #[test]
    fn resolved_chain_defaults_to_ollama() {
        let cfg = ProvidersConfig::default();
        assert_eq!(cfg.resolved_chain(), vec![ProviderKind::Ollama]);
    }

    #[test]
    fn resolved_chain_respects_order() {
        let mut cfg = ProvidersConfig::default();
        cfg.fallback_chain = vec!["gemini".into(), "openai".into(), "ollama".into()];
        let chain = cfg.resolved_chain();
        assert_eq!(chain[0], ProviderKind::Gemini);
        assert_eq!(chain[1], ProviderKind::OpenAi);
        assert_eq!(chain[2], ProviderKind::Ollama);
    }

    #[test]
    fn resolved_chain_skips_disabled() {
        let mut cfg = ProvidersConfig::default();
        cfg.fallback_chain = vec!["openai".into(), "ollama".into()];
        cfg.openai.enabled = false;
        let chain = cfg.resolved_chain();
        assert_eq!(chain, vec![ProviderKind::Ollama]);
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
        let mut entry = ProviderEntry::default();
        entry.api_key = Some("cfg-key".into());
        // Even if env var is set, config key wins
        let resolved = entry.resolve_api_key("NONEXISTENT_TEST_VAR_XYZ");
        assert_eq!(resolved, Some("cfg-key".into()));
    }

    #[test]
    fn resolve_model_uses_default_when_unset() {
        let entry = ProviderEntry::default();
        assert_eq!(entry.resolve_model(ProviderKind::Gemini), "gemini-2.0-flash");
    }
}
