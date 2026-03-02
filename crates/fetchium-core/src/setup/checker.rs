//! Environment checks for `fetchium setup --check` and `fetchium doctor`.

use crate::config::HsxConfig;

/// A single environment check result.
#[derive(Debug)]
pub struct CheckItem {
    pub name: &'static str,
    pub status: CheckStatus,
    pub detail: String,
}

/// Status of an environment check.
#[derive(Debug, PartialEq, Eq)]
pub enum CheckStatus {
    /// Dependency found and working.
    Ok,
    /// Present but not ideal (optional or degraded).
    Warning,
    /// Missing — functionality will not work.
    Missing,
}

impl CheckStatus {
    /// Single-character icon for terminal output.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Ok => "✓",
            Self::Warning => "⚠",
            Self::Missing => "✗",
        }
    }
}

/// Run all setup-relevant environment checks and return results.
pub fn run_checks(config: &HsxConfig) -> Vec<CheckItem> {
    let mut items = Vec::new();

    // ── Chrome/Chromium ────────────────────────────────────────────────────
    let chrome = super::chromium::resolve_chrome_path(config);
    items.push(CheckItem {
        name: "Chrome/Chromium",
        status: if chrome.is_some() {
            CheckStatus::Ok
        } else {
            CheckStatus::Missing
        },
        detail: chrome
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "Not found — run: fetchium setup --headless".into()),
    });

    // Show whether Chrome is fetchium-managed or system
    if let Some(ref p) = chrome {
        let managed = config.data_dir().join("chromium");
        let source = if p.starts_with(&managed) {
            "(fetchium-managed)"
        } else if p.to_string_lossy().contains("snap") {
            "(snap)"
        } else {
            "(system)"
        };
        items.push(CheckItem {
            name: "  Chrome source",
            status: CheckStatus::Ok,
            detail: source.into(),
        });
    }

    // ── SearXNG ────────────────────────────────────────────────────────────
    match &config.search.searxng_url {
        Some(url) => {
            items.push(CheckItem {
                name: "SearXNG",
                status: CheckStatus::Ok,
                detail: format!("configured at {url}"),
            });
        }
        None => {
            items.push(CheckItem {
                name: "SearXNG",
                status: CheckStatus::Warning,
                detail: "not configured (set search.searxng_url in config)".into(),
            });
        }
    }

    // ── Data directory ─────────────────────────────────────────────────────
    let data_dir = config.data_dir();
    let dir_ok = data_dir.exists() || std::fs::create_dir_all(&data_dir).is_ok();
    items.push(CheckItem {
        name: "Data directory",
        status: if dir_ok {
            CheckStatus::Ok
        } else {
            CheckStatus::Missing
        },
        detail: data_dir.display().to_string(),
    });

    // ── Config file ────────────────────────────────────────────────────────
    let config_path = HsxConfig::config_file_path();
    items.push(CheckItem {
        name: "Config file",
        status: CheckStatus::Ok,
        detail: if config_path.exists() {
            config_path.display().to_string()
        } else {
            format!("{} (using defaults)", config_path.display())
        },
    });

    items
}
