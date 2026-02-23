//! Configuration system for HyperSearchX (PRD §11).
//!
//! Layered: defaults → config file → env vars → CLI args.
//! Config file: `~/.hypersearchx/config.toml`

use crate::types::{OutputFormat, PdsTier, ResourceTier};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Top-level configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

// Manual default removed, used #[derive(Default)] on the struct instead.

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
            user_agent: format!(
                "HyperSearchX/{} (https://github.com/hypersearchx/hypersearchx)",
                env!("CARGO_PKG_VERSION")
            ),
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
        self.general.data_dir.clone().unwrap_or_else(|| {
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

        let mut config = if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(contents) => match toml::from_str(&contents) {
                    Ok(config) => config,
                    Err(e) => {
                        tracing::warn!("Failed to parse config at {}: {e}", config_path.display());
                        Self::default()
                    }
                },
                Err(e) => {
                    tracing::warn!("Failed to read config at {}: {e}", config_path.display());
                    Self::default()
                }
            }
        } else {
            Self::default()
        };

        // Layer 3: environment variable overrides
        config.apply_env_overrides();

        config
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

    /// Apply environment variable overrides.
    /// Convention: `HSX_SECTION_KEY` (uppercase, underscore-separated).
    /// Examples: HSX_SEARCH_DEFAULT_BUDGET=8000, HSX_CACHE_ENABLED=false
    pub fn apply_env_overrides(&mut self) {
        if let Ok(val) = std::env::var("HSX_SEARCH_DEFAULT_BUDGET") {
            if let Ok(budget) = val.parse::<u32>() {
                self.search.default_budget = budget;
            }
        }
        if let Ok(val) = std::env::var("HSX_SEARCH_MAX_CONCURRENT") {
            if let Ok(n) = val.parse::<u32>() {
                self.search.max_concurrent = n;
            }
        }
        if let Ok(val) = std::env::var("HSX_SEARCH_TIMEOUT_SECS") {
            if let Ok(n) = val.parse::<u64>() {
                self.search.timeout_secs = n;
            }
        }
        if let Ok(val) = std::env::var("HSX_CACHE_ENABLED") {
            if let Ok(b) = val.parse::<bool>() {
                self.cache.enabled = b;
            }
        }
        if let Ok(val) = std::env::var("HSX_CACHE_TTL_SECS") {
            if let Ok(n) = val.parse::<u64>() {
                self.cache.ttl_secs = n;
            }
        }
        if let Ok(val) = std::env::var("HSX_AI_OLLAMA_HOST") {
            self.ai.ollama_host = val;
        }
        if let Ok(val) = std::env::var("HSX_AI_DEFAULT_MODEL") {
            self.ai.default_model = val;
        }
        if let Ok(val) = std::env::var("HSX_GENERAL_VERBOSE") {
            if let Ok(b) = val.parse::<bool>() {
                self.general.verbose = b;
            }
        }
        if let Ok(val) = std::env::var("HSX_FETCH_USER_AGENT") {
            self.fetch.user_agent = val;
        }
        if let Ok(val) = std::env::var("HSX_FETCH_RESPECT_ROBOTS") {
            if let Ok(b) = val.parse::<bool>() {
                self.fetch.respect_robots = b;
            }
        }
    }

    /// Ensure the data directory exists, creating it if necessary.
    pub fn ensure_data_dir(&self) -> std::io::Result<PathBuf> {
        let dir = self.data_dir();
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }
        Ok(dir)
    }

    /// Get the path to the config file.
    pub fn config_file_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".hypersearchx")
            .join("config.toml")
    }

    /// Write the current config to the config file (for `hsx config set`).
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_file_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let toml_str = toml::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(&path, toml_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = HsxConfig::default();
        assert_eq!(config.search.default_budget, 4000);
        assert!(config.fetch.respect_robots);
        assert!(config.cache.enabled);
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

    #[test]
    fn env_override_budget() {
        std::env::set_var("HSX_SEARCH_DEFAULT_BUDGET", "8000");
        let mut config = HsxConfig::default();
        config.apply_env_overrides();
        assert_eq!(config.search.default_budget, 8000);
        std::env::remove_var("HSX_SEARCH_DEFAULT_BUDGET");
    }

    #[test]
    fn env_override_cache_disabled() {
        std::env::set_var("HSX_CACHE_ENABLED", "false");
        let mut config = HsxConfig::default();
        config.apply_env_overrides();
        assert!(!config.cache.enabled);
        std::env::remove_var("HSX_CACHE_ENABLED");
    }

    #[test]
    fn save_and_reload() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");

        let mut config = HsxConfig::default();
        config.search.default_budget = 9999;

        // Save
        let toml_str = toml::to_string_pretty(&config).unwrap();
        std::fs::write(&config_path, &toml_str).unwrap();

        // Reload
        let loaded = HsxConfig::load_from(Some(&config_path));
        assert_eq!(loaded.search.default_budget, 9999);
    }

    #[test]
    fn config_file_path_contains_hypersearchx() {
        let path = HsxConfig::config_file_path();
        assert!(path.to_string_lossy().contains(".hypersearchx"));
        assert!(path.to_string_lossy().ends_with("config.toml"));
    }
}
