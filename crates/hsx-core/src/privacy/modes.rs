//! Privacy mode definitions and enforcement (PRD §36).

use crate::error::HsxError;

/// The four privacy operating modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyMode {
    /// Normal operation — cache, history, PIE all enabled.
    Standard,
    /// No cache, no history, no PIE updates; nothing written to disk.
    Private,
    /// Route all HTTP through Tor SOCKS5 proxy (requires Tor daemon).
    Tor,
    /// Zero network — only local index queries (requires Phase 5 index).
    AirGap,
}

impl std::fmt::Display for PrivacyMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrivacyMode::Standard => write!(f, "standard"),
            PrivacyMode::Private => write!(f, "private"),
            PrivacyMode::Tor => write!(f, "tor"),
            PrivacyMode::AirGap => write!(f, "air-gap"),
        }
    }
}

impl PrivacyMode {
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().trim() {
            "standard" | "normal" | "default" => Some(PrivacyMode::Standard),
            "private" | "incognito" | "no-trace" => Some(PrivacyMode::Private),
            "tor" | "onion" => Some(PrivacyMode::Tor),
            "air-gap" | "airgap" | "offline" => Some(PrivacyMode::AirGap),
            _ => None,
        }
    }
}

/// Runtime configuration modified by a privacy mode.
///
/// Callers should create this with `RuntimeConfig::default()`, then call
/// `apply_mode()` to override fields based on the selected mode.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub cache_enabled: bool,
    pub pie_enabled: bool,
    pub history_enabled: bool,
    pub embedding_cache_enabled: bool,
    pub network_enabled: bool,
    pub local_index_only: bool,
    /// SOCKS5 proxy URL, e.g. `socks5://127.0.0.1:9050`
    pub proxy: Option<String>,
    pub user_agent: Option<String>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            cache_enabled: true,
            pie_enabled: true,
            history_enabled: true,
            embedding_cache_enabled: true,
            network_enabled: true,
            local_index_only: false,
            proxy: None,
            user_agent: None,
        }
    }
}

/// Apply a privacy mode to a `RuntimeConfig`.
pub fn apply_mode(mode: PrivacyMode, config: &mut RuntimeConfig) -> Result<(), HsxError> {
    match mode {
        PrivacyMode::Standard => {}
        PrivacyMode::Private => {
            config.cache_enabled = false;
            config.pie_enabled = false;
            config.history_enabled = false;
            config.embedding_cache_enabled = false;
            tracing::info!("Privacy: private mode — cache/history/PIE disabled");
        }
        PrivacyMode::Tor => {
            // Verify Tor is reachable before committing
            check_tor_available()?;
            config.proxy = Some("socks5://127.0.0.1:9050".to_string());
            config.user_agent = Some(
                "Mozilla/5.0 (Windows NT 10.0; rv:128.0) Gecko/20100101 Firefox/128.0"
                    .to_string(),
            );
            tracing::info!("Privacy: Tor mode — routing via SOCKS5 proxy");
        }
        PrivacyMode::AirGap => {
            config.network_enabled = false;
            config.local_index_only = true;
            tracing::info!("Privacy: air-gap mode — local index only");
        }
    }
    Ok(())
}

/// Check if Tor SOCKS5 proxy is reachable on 127.0.0.1:9050.
fn check_tor_available() -> Result<(), HsxError> {
    use std::net::TcpStream;
    use std::time::Duration;
    TcpStream::connect_timeout(
        &"127.0.0.1:9050".parse().unwrap(),
        Duration::from_secs(2),
    )
    .map(|_| ())
    .map_err(|_| {
        HsxError::Internal(
            "Tor SOCKS5 proxy not reachable at 127.0.0.1:9050. \
             Install Tor and ensure it is running (`tor` or `brew install tor && brew services start tor`)."
                .into(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_mode_disables_cache() {
        let mut cfg = RuntimeConfig::default();
        apply_mode(PrivacyMode::Private, &mut cfg).unwrap();
        assert!(!cfg.cache_enabled);
        assert!(!cfg.pie_enabled);
    }

    #[test]
    fn air_gap_disables_network() {
        let mut cfg = RuntimeConfig::default();
        apply_mode(PrivacyMode::AirGap, &mut cfg).unwrap();
        assert!(!cfg.network_enabled);
        assert!(cfg.local_index_only);
    }

    #[test]
    fn mode_from_str_loose() {
        assert_eq!(
            PrivacyMode::from_str_loose("private"),
            Some(PrivacyMode::Private)
        );
        assert_eq!(
            PrivacyMode::from_str_loose("TOR"),
            Some(PrivacyMode::Tor)
        );
        assert_eq!(PrivacyMode::from_str_loose("unknown"), None);
    }
}
