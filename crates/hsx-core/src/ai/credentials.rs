//! Subscription-based credential detection for AI providers.
//!
//! Reads OAuth tokens that have already been set up by first-party CLIs:
//! - **Gemini**: reads `~/.gemini/oauth_creds.json` (populated by `gemini auth login`)
//! - **Claude Code**: reads from macOS Keychain service `"Claude Code-credentials"`
//! - **OpenAI Codex**: reads `~/.codex/auth.json` (populated by `codex auth login`)
//!
//! No browser flows are initiated here — this module only reads existing credentials.

use serde::{Deserialize, Serialize};

// ─── Gemini OAuth credentials ─────────────────────────────────────────────────

/// Google OAuth credentials stored by the Gemini CLI at `~/.gemini/oauth_creds.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiOAuthCreds {
    /// Current access token (short-lived, ~1 hour).
    pub access_token: String,
    /// Long-lived refresh token.
    pub refresh_token: String,
    /// Expiry as Unix timestamp in **milliseconds**.
    pub expiry_date: u64,
    /// Always `"Bearer"`.
    pub token_type: String,
}

impl GeminiOAuthCreds {
    /// True if the access token has not yet expired.
    pub fn is_valid(&self) -> bool {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        // Allow 60-second grace period
        self.expiry_date > now_ms + 60_000
    }
}

/// Read Gemini CLI OAuth credentials from `~/.gemini/oauth_creds.json`.
///
/// Returns `None` if the file does not exist or cannot be parsed.
pub fn read_gemini_creds() -> Option<GeminiOAuthCreds> {
    let path = dirs::home_dir()?.join(".gemini").join("oauth_creds.json");
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Return a valid access token if one is cached locally, or `None` if expired/absent.
///
/// Call [`refresh_gemini_token`] (async) when this returns `None` due to expiry.
pub fn get_gemini_access_token_if_valid() -> Option<String> {
    let creds = read_gemini_creds()?;
    if creds.is_valid() {
        Some(creds.access_token.clone())
    } else {
        tracing::debug!("Gemini OAuth token expired — refresh needed");
        None
    }
}

/// Google's public OAuth client ID baked into the Gemini CLI package.
/// This is a "native app" / "installed app" OAuth client — the client_secret
/// is **not secret** for this client type.
pub const GEMINI_OAUTH_CLIENT_ID: &str =
    "681255809395-oo8ft2oprdrnp9e3aqf6av3hmdib135j.apps.googleusercontent.com";
/// Public client secret for the Gemini CLI's installed-app OAuth client.
pub const GEMINI_OAUTH_CLIENT_SECRET: &str = "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";
/// Google OAuth token endpoint.
pub const GOOGLE_TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";

// ─── Claude Code (Anthropic) OAuth credentials ───────────────────────────────

/// Claude Code OAuth credentials stored in the macOS Keychain.
#[derive(Debug, Clone)]
pub struct ClaudeOAuthCreds {
    /// OAuth access token for the Anthropic API (`sk-ant-oat01-...`).
    pub access_token: String,
    /// Refresh token (`sk-ant-ort01-...`).
    pub refresh_token: String,
    /// Token expiry as Unix timestamp in milliseconds.
    pub expires_at: u64,
    /// Subscription tier (e.g. `"max"`, `"pro"`).
    pub subscription_type: String,
}

impl ClaudeOAuthCreds {
    /// True if the access token has not yet expired.
    pub fn is_valid(&self) -> bool {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        self.expires_at > now_ms + 60_000
    }
}

/// Read Claude Code OAuth credentials from the macOS Keychain.
///
/// Claude Code stores its session under service `"Claude Code-credentials"`.
/// Returns `None` on non-macOS systems or if credentials are not present.
pub fn read_claude_code_creds() -> Option<ClaudeOAuthCreds> {
    #[cfg(target_os = "macos")]
    {
        // Use the macOS `security` CLI to read from Keychain without extra deps.
        let output = std::process::Command::new("security")
            .args(["find-generic-password", "-s", "Claude Code-credentials", "-w"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let raw = String::from_utf8(output.stdout).ok()?;
        let json: serde_json::Value = serde_json::from_str(raw.trim()).ok()?;
        let oauth = json.get("claudeAiOauth")?;

        let access_token  = oauth.get("accessToken")?.as_str()?.to_string();
        let refresh_token = oauth.get("refreshToken")?.as_str()?.to_string();
        let expires_at    = oauth.get("expiresAt")?.as_u64().unwrap_or(0);
        let subscription  = oauth.get("subscriptionType")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
            .to_string();

        Some(ClaudeOAuthCreds {
            access_token,
            refresh_token,
            expires_at,
            subscription_type: subscription,
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        // On Linux/Windows, check for a credentials file that users can create manually.
        let path = dirs::home_dir()?.join(".claude").join("credentials.json");
        let content = std::fs::read_to_string(path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        let oauth = json.get("claudeAiOauth")?;

        let access_token  = oauth.get("accessToken")?.as_str()?.to_string();
        let refresh_token = oauth.get("refreshToken")?.as_str()?.to_string();
        let expires_at    = oauth.get("expiresAt")?.as_u64().unwrap_or(0);
        let subscription  = oauth.get("subscriptionType")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
            .to_string();

        Some(ClaudeOAuthCreds {
            access_token,
            refresh_token,
            expires_at,
            subscription_type: subscription,
        })
    }
}

/// Get a usable Anthropic access token from Claude Code's session.
///
/// Returns `None` if Claude Code is not installed or the session has expired.
pub fn get_claude_code_token() -> Option<ClaudeOAuthCreds> {
    let creds = read_claude_code_creds()?;
    if creds.is_valid() {
        return Some(creds);
    }
    // Token expired — refreshing Anthropic OAuth requires hitting their
    // token endpoint which may change. For now, instruct user to re-run Claude Code.
    tracing::warn!(
        "Claude Code OAuth token has expired. Re-run `claude` once to refresh it, then retry."
    );
    None
}

// ─── Detection helpers ────────────────────────────────────────────────────────

/// Returns `true` if Gemini CLI is installed and has OAuth credentials.
pub fn gemini_cli_auth_available() -> bool {
    // Check gemini binary exists
    let has_binary = std::process::Command::new("gemini")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    has_binary && read_gemini_creds().is_some()
}

/// Returns `true` if Claude Code OAuth credentials are available.
pub fn claude_code_auth_available() -> bool {
    read_claude_code_creds().is_some()
}

// ─── OpenAI Codex CLI credentials ────────────────────────────────────────────

/// OpenAI Codex CLI OAuth credentials stored at `~/.codex/auth.json`.
///
/// Populated by `codex auth login` (ChatGPT sign-in flow).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexOAuthCreds {
    /// Short-lived access token (JWT, used as `Authorization: Bearer`).
    pub access_token: String,
    /// Refresh token for obtaining a new access token.
    pub refresh_token: Option<String>,
    /// Expiry as ISO 8601 / RFC 3339 string (e.g. `"2025-04-01T10:30:00Z"`).
    pub expires_at: Option<String>,
}

impl CodexOAuthCreds {
    /// True if the access token has not yet expired (or expiry is unknown).
    pub fn is_valid(&self) -> bool {
        let Some(ref exp_str) = self.expires_at else {
            return true; // no expiry field → assume valid
        };
        // Parse as RFC 3339; if parsing fails also assume valid
        let Ok(exp) = chrono::DateTime::parse_from_rfc3339(exp_str) else {
            return true;
        };
        let now = chrono::Utc::now();
        exp.signed_duration_since(now).num_seconds() > 60
    }
}

/// Read OpenAI Codex CLI credentials from `~/.codex/auth.json`.
///
/// Returns `None` if the file does not exist, cannot be read, or cannot be parsed.
pub fn read_codex_creds() -> Option<CodexOAuthCreds> {
    let path = dirs::home_dir()?.join(".codex").join("auth.json");
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Return a valid OpenAI Codex access token, or `None` if absent/expired.
pub fn get_codex_token_if_valid() -> Option<String> {
    let creds = read_codex_creds()?;
    if creds.is_valid() {
        Some(creds.access_token.clone())
    } else {
        tracing::debug!("OpenAI Codex token expired — re-run `codex auth login` to refresh");
        None
    }
}

/// Returns `true` if OpenAI Codex CLI credentials are present and valid.
pub fn codex_auth_available() -> bool {
    get_codex_token_if_valid().is_some()
}

// ─── OpenRouter ───────────────────────────────────────────────────────────────

/// Read OpenRouter API key from its config file at `~/.openrouter/config.json`.
///
/// OpenRouter stores credentials in a simple JSON file after `openrouter auth`.
pub fn read_openrouter_key() -> Option<String> {
    let path = dirs::home_dir()?.join(".openrouter").join("config.json");
    let content = std::fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    json.get("api_key")
        .or_else(|| json.get("apiKey"))
        .or_else(|| json.get("key"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

// ─── Combined detection ───────────────────────────────────────────────────────

/// Describe which subscription auth methods are available on this machine.
pub fn detect_subscription_auth() -> Vec<SubscriptionAuth> {
    let mut found = Vec::new();

    if let Some(creds) = read_claude_code_creds() {
        found.push(SubscriptionAuth::ClaudeCode {
            subscription_type: creds.subscription_type.clone(),
            valid: creds.is_valid(),
        });
    }

    if let Some(creds) = read_gemini_creds() {
        // Try to get the active account email
        let account = std::fs::read_to_string(
            dirs::home_dir()
                .unwrap_or_default()
                .join(".gemini")
                .join("google_accounts.json"),
        )
        .ok()
        .and_then(|s| {
            let json: serde_json::Value = serde_json::from_str(&s).ok()?;
            json["active"].as_str().map(|s| s.to_string())
        });

        found.push(SubscriptionAuth::GeminiOAuth {
            account,
            valid: creds.is_valid(),
        });
    }

    if let Some(creds) = read_codex_creds() {
        found.push(SubscriptionAuth::CodexOAuth { valid: creds.is_valid() });
    }

    if let Some(key) = read_openrouter_key() {
        let masked = if key.len() > 4 {
            format!("…{}", &key[key.len() - 4..])
        } else {
            "****".into()
        };
        found.push(SubscriptionAuth::OpenRouterKey { masked });
    }

    found
}

/// A detected subscription-based authentication method.
#[derive(Debug, Clone)]
pub enum SubscriptionAuth {
    /// Claude Code OAuth session (Anthropic).
    ClaudeCode {
        subscription_type: String,
        valid: bool,
    },
    /// Google OAuth session (Gemini CLI).
    GeminiOAuth {
        account: Option<String>,
        valid: bool,
    },
    /// OpenAI Codex CLI OAuth session (ChatGPT subscription).
    CodexOAuth {
        valid: bool,
    },
    /// OpenRouter API key stored in `~/.openrouter/config.json`.
    OpenRouterKey {
        /// Masked key (last 4 chars visible).
        masked: String,
    },
}

impl SubscriptionAuth {
    /// Human-readable description.
    pub fn description(&self) -> String {
        match self {
            Self::ClaudeCode { subscription_type, valid } => format!(
                "Claude Code session ({subscription_type}) — {}",
                if *valid { "valid" } else { "expired" }
            ),
            Self::GeminiOAuth { account, valid } => format!(
                "Gemini OAuth ({}) — {}",
                account.as_deref().unwrap_or("unknown account"),
                if *valid { "valid" } else { "expired (needs refresh)" }
            ),
            Self::CodexOAuth { valid } => format!(
                "OpenAI Codex CLI session — {}",
                if *valid { "valid" } else { "expired (run `codex auth login`)" }
            ),
            Self::OpenRouterKey { masked } => format!(
                "OpenRouter API key ({masked}) — stored in ~/.openrouter/config.json"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gemini_creds_validity_future() {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let creds = GeminiOAuthCreds {
            access_token: "test".into(),
            refresh_token: "rt".into(),
            expiry_date: now_ms + 3_600_000, // 1 hour from now
            token_type: "Bearer".into(),
        };
        assert!(creds.is_valid());
    }

    #[test]
    fn gemini_creds_validity_expired() {
        let creds = GeminiOAuthCreds {
            access_token: "test".into(),
            refresh_token: "rt".into(),
            expiry_date: 1_000_000, // way in the past
            token_type: "Bearer".into(),
        };
        assert!(!creds.is_valid());
    }

    #[test]
    fn claude_oauth_validity_future() {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let creds = ClaudeOAuthCreds {
            access_token: "sk-ant-oat01-test".into(),
            refresh_token: "rt".into(),
            expires_at: now_ms + 7_200_000, // 2 hours
            subscription_type: "max".into(),
        };
        assert!(creds.is_valid());
    }
}
