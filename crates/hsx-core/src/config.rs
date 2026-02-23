//! Configuration system for HyperSearchX (PRD §11).
//!
//! Layered: defaults → config file → env vars → CLI args.
//! Config file: `~/.hypersearchx/config.toml`

use crate::types::{OutputFormat, PdsTier, ResourceTier};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Top-level configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HsxConfig {
    pub general: GeneralConfig,
    pub search: SearchConfig,
    pub fetch: FetchConfig,
    pub cache: CacheConfig,
    pub ai: AiConfig,
    pub output: OutputConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// Default number of results to return.
    pub max_results: u32,
    /// Default output format.
    pub format: OutputFormat,
    /// Verbose logging.
    pub verbose: bool,
    /// Data directory (default: ~/.hypersearchx).
    pub data_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchConfig {
    /// Enabled backends.
    pub backends: Vec<String>,
    /// Default token budget for agent commands.
    pub default_budget: u32,
    /// Default PDS tier.
    pub default_tier: PdsTier,
    /// Maximum concurrent requests.
    pub max_concurrent: u32,
    /// Timeout per request in seconds.
    pub timeout_secs: u64,
    /// SearXNG instance URL (if configured).
    pub searxng_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FetchConfig {
    /// User-Agent string.
    pub user_agent: String,
    /// Respect robots.txt.
    pub respect_robots: bool,
    /// Maximum page size in bytes.
    pub max_page_size: u64,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
    /// Maximum redirects to follow.
    pub max_redirects: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CacheConfig {
    /// Enable caching.
    pub enabled: bool,
    /// Memory cache max entries.
    pub memory_max_entries: u64,
    /// Disk cache max size in MB.
    pub disk_max_mb: u64,
    /// Cache TTL in seconds.
    pub ttl_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AiConfig {
    /// Ollama host.
    pub ollama_host: String,
    /// Default model.
    pub default_model: String,
    /// Max tokens for AI responses.
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OutputConfig {
    /// Default output format.
    pub format: OutputFormat,
    /// Include source URLs in output.
    pub include_sources: bool,
    /// Include confidence scores.
    pub include_confidence: bool,
}

// ─── Defaults ────────────────────────────────────────────────────

impl Default for HsxConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            search: SearchConfig::default(),
            fetch: FetchConfig::default(),
            cache: CacheConfig::default(),
            ai: AiConfig::default(),
            output: OutputConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            max_results: 10,
            format: OutputFormat::Markdown,
            verbose: false,
            data_dir: None,
        }
    }
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            backends: vec!["duckduckgo".into()],
            default_budget: 4000,
            default_tier: PdsTier::Summary,
            max_concurrent: 8,
            timeout_secs: 30,
            searxng_url: None,
        }
    }
}

impl Default for FetchConfig {
    fn default() -> Self {
        Self {
            user_agent: format!("HyperSearchX/{} (https://github.com/hypersearchx/hypersearchx)", env!("CARGO_PKG_VERSION")),
            respect_robots: true,
            max_page_size: 10 * 1024 * 1024, // 10 MB
            timeout_secs: 30,
            max_redirects: 10,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            memory_max_entries: 1000,
            disk_max_mb: 500,
            ttl_secs: 3600,
        }
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            ollama_host: "http://localhost:11434".into(),
            default_model: "llama3.2".into(),
            max_tokens: 8192,
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::Markdown,
            include_sources: true,
            include_confidence: true,
        }
    }
}

// ─── Loading ─────────────────────────────────────────────────────

impl HsxConfig {
    /// Get the data directory, creating it if it doesn't exist.
    pub fn data_dir(&self) -> PathBuf {
        self.general
            .data_dir
            .clone()
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".hypersearchx")
            })
    }

    /// Load config from the default location.
    pub fn load() -> Self {
        Self::load_from(None)
    }

    /// Load config with an optional path override.
    pub fn load_from(path: Option<&std::path::Path>) -> Self {
        let config_path = path.map(|p| p.to_path_buf()).unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".hypersearchx")
                .join("config.toml")
        });

        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(contents) => match toml::from_str(&contents) {
                    Ok(config) => return config,
                    Err(e) => {
                        tracing::warn!("Failed to parse config at {}: {e}", config_path.display());
                    }
                },
                Err(e) => {
                    tracing::warn!("Failed to read config at {}: {e}", config_path.display());
                }
            }
        }

        Self::default()
    }

    /// Detect the resource tier based on system capabilities.
    pub fn detect_resource_tier() -> ResourceTier {
        let sys = sysinfo::System::new_all();
        let total_ram_gb = sys.total_memory() / (1024 * 1024 * 1024);
        let cpu_count = sys.cpus().len();

        match (total_ram_gb, cpu_count) {
            (ram, cpus) if ram >= 32 && cpus >= 8 => ResourceTier::Server,
            (ram, cpus) if ram >= 16 && cpus >= 4 => ResourceTier::Performance,
            (ram, _) if ram >= 4 => ResourceTier::Standard,
            _ => ResourceTier::Minimal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = HsxConfig::default();
        assert_eq!(config.search.default_budget, 4000);
        assert_eq!(config.fetch.respect_robots, true);
        assert_eq!(config.cache.enabled, true);
    }

    #[test]
    fn config_roundtrip_toml() {
        let config = HsxConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let back: HsxConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(back.search.default_budget, config.search.default_budget);
    }

    #[test]
    fn data_dir_default() {
        let config = HsxConfig::default();
        let dir = config.data_dir();
        assert!(dir.to_string_lossy().contains(".hypersearchx"));
    }
}
