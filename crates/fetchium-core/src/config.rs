//! Configuration system for Fetchium (PRD §11).
//!
//! Layered: defaults → config file → env vars → CLI args.
//! Config file: `~/.fetchium/config.toml`

use crate::ai::providers::ProvidersConfig;
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
    pub ranking: RankingConfig,
    pub youtube: YouTubeConfig,
    pub social: SocialConfig,
    pub headless: HeadlessConfig,
    pub proxy: ProxyConfig,
    pub dataimpulse: DataImpulseConfig,
}

/// DataImpulse residential proxy configuration.
///
/// Country targeting: `{username}__cr.{cc}:{password}@{host}:{port}`
/// Only activates for scraper backends blocked by datacenter IPs (Google, DDG, Bing, Brave).
/// API backends (Serper, Exa, Tavily) are never routed through DataImpulse.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DataImpulseConfig {
    /// Enable DataImpulse residential proxy routing.
    pub enabled: bool,
    /// DataImpulse account username.
    pub username: String,
    /// DataImpulse account password.
    pub password: String,
    /// Gateway host (default: gw.dataimpulse.com).
    pub host: String,
    /// Gateway port (default: 823).
    pub port: u16,
    /// Domains to route through DataImpulse (empty = use built-in blocked-domain list).
    #[serde(default)]
    pub proxy_domains: Vec<String>,
}

impl Default for DataImpulseConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            username: String::new(),
            password: String::new(),
            host: "gw.dataimpulse.com".into(),
            port: 823,
            proxy_domains: Vec::new(),
        }
    }
}

/// Proxy rotation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProxyConfig {
    /// Enable proxy rotation for search backends.
    pub enabled: bool,
    /// Path to proxy list file (default: ~/.fetchium/proxies.txt).
    pub proxy_file: Option<PathBuf>,
    /// Proxy protocol (http, https, socks5).
    pub protocol: String,
    /// Domains that should use proxies (empty = all search backends).
    #[serde(default)]
    pub proxy_domains: Vec<String>,
    /// Domains that should never use proxies.
    #[serde(default)]
    pub bypass_domains: Vec<String>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            proxy_file: None,
            protocol: "http".into(),
            proxy_domains: Vec::new(),
            bypass_domains: vec![
                "localhost".into(),
                "127.0.0.1".into(),
            ],
        }
    }
}

/// Headless browser configuration.
///
/// Stored as `[headless]` in config.toml. Controls Chrome binary resolution.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HeadlessConfig {
    /// Override path to Chrome/Chromium binary.
    ///
    /// Priority: `FETCHIUM_CHROME_PATH` env var > this value >
    /// fetchium-managed (`~/.fetchium/chromium/`) > system Chrome/Chromium.
    pub chrome_path: Option<PathBuf>,
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
    /// Data directory (default: ~/.fetchium).
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
    /// Tavily API key (set via TAVILY_API_KEY env var or config).
    pub tavily_api_key: Option<String>,
    /// Extra Tavily keys for pool rotation.
    #[serde(default)]
    pub tavily_api_keys: Vec<String>,
    /// Serper API key (set via SERPER_API_KEY env var or config).
    pub serper_api_key: Option<String>,
    /// Extra Serper keys for pool rotation.
    #[serde(default)]
    pub serper_api_keys: Vec<String>,
    /// Exa API key (set via EXA_API_KEY env var or config).
    pub exa_api_key: Option<String>,
    /// Extra Exa keys for pool rotation.
    #[serde(default)]
    pub exa_api_keys: Vec<String>,
    /// Firecrawl API key (set via FIRECRAWL_API_KEY env var or config).
    pub firecrawl_api_key: Option<String>,
    /// Extra Firecrawl keys for pool rotation.
    #[serde(default)]
    pub firecrawl_api_keys: Vec<String>,
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
    /// Default model (Ollama).
    pub default_model: String,
    /// Max tokens for AI responses.
    pub max_tokens: u32,
    /// Optional fast model for latency-sensitive tasks (HyDE, intent classification).
    /// Falls back to `default_model` when unset.
    pub fast_model: Option<String>,
    /// Multi-provider configuration block.
    ///
    /// Stored as `[ai.providers]` in config.toml. Defines the fallback chain
    /// and per-provider API keys / model overrides.
    pub providers: ProvidersConfig,
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

/// YouTube Intelligence System configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct YouTubeConfig {
    /// Preferred Invidious instance URLs (tried in order).
    pub invidious_instances: Vec<String>,
    /// Preferred Piped API instance URLs (tried in order).
    pub piped_instances: Vec<String>,
    /// Maximum videos to analyze in a pipeline run.
    pub max_videos: usize,
    /// Per-instance request timeout in seconds.
    pub timeout_secs: u64,
    /// Whether to fetch comments by default.
    pub fetch_comments: bool,
}

impl Default for YouTubeConfig {
    fn default() -> Self {
        Self {
            invidious_instances: vec![
                "https://invidious.nerdvpn.de".into(), // verified Feb 2026
                "https://yewtu.be".into(),             // verified Feb 2026
            ],
            // Limit to 2 most reliable Piped instances. Fewer instances =
            // fewer orphaned retry tasks per video when they're unreachable.
            // Innertube + oEmbed already guarantee metadata; Piped is bonus.
            piped_instances: vec![
                "https://pipedapi.kavin.rocks".into(), // official, most reliable
                "https://api.piped.yt".into(),         // Germany fallback
            ],
            max_videos: 5,
            timeout_secs: 4, // tight per-source timeout; sources are raced in parallel
            fetch_comments: false, // comments disabled by default (expensive)
        }
    }
}

/// Social Media Intelligence configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SocialConfig {
    /// Meta Graph API token for Facebook (`APP_ID|APP_SECRET`).
    ///
    /// Get one for free at <https://developers.facebook.com>:
    /// 1. Create an app (type: "Consumer" or "Business")
    /// 2. Go to Settings → Basic → copy App ID + App Secret
    /// 3. Set token as `APP_ID|APP_SECRET`
    ///
    /// Without a token, Facebook uses DDG + Open Graph (free, limited data).
    pub facebook_graph_token: Option<String>,
}

impl HsxConfig {
    /// Returns the Facebook Graph API token from config or env var.
    ///
    /// Checks `FETCHIUM_FACEBOOK_TOKEN`, then the config file value.
    pub fn facebook_graph_token(&self) -> Option<String> {
        self.social
            .facebook_graph_token
            .clone()
            .or_else(|| std::env::var("FETCHIUM_FACEBOOK_TOKEN").ok())
    }
}

/// Ranking configuration for HyperFusion signal weights and thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RankingConfig {
    /// Per-signal weight overrides (signal name → weight in [0.0, 1.0]).
    /// Example: `{"temporal": 0.3, "authority": 0.2}`
    #[serde(default)]
    pub weight_overrides: std::collections::HashMap<String, f64>,
    /// How much recency matters for temporal scoring (0.0 = ignore, 1.0 = max).
    #[serde(default = "RankingConfig::default_freshness")]
    pub freshness_need: f64,
    /// SimHash Hamming distance threshold for near-duplicate detection (0–64).
    #[serde(default = "RankingConfig::default_simhash_threshold")]
    pub simhash_threshold: u32,
}

impl RankingConfig {
    fn default_freshness() -> f64 {
        0.5
    }
    fn default_simhash_threshold() -> u32 {
        6
    }
}

impl Default for RankingConfig {
    fn default() -> Self {
        Self {
            weight_overrides: std::collections::HashMap::new(),
            freshness_need: Self::default_freshness(),
            simhash_threshold: Self::default_simhash_threshold(),
        }
    }
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
            // Default backends: stable API/meta-search sources only.
            // SearXNG covers Google, Bing, DDG internally — no per-server CAPTCHAs.
            // Wikipedia/HN/Reddit/SO/Arxiv/GitHub use public APIs (no rate limiting).
            // Direct Bing/DDG/Google scrapers excluded — they CAPTCHA-block server IPs.
            backends: vec![
                "searxng".into(),
                "tavily".into(),
                "serper".into(),
                "exa".into(),
                "firecrawl".into(),
                "wikipedia".into(),
                "hackernews".into(),
                "reddit".into(),
                "stackoverflow".into(),
                "arxiv".into(),
                "github".into(),
            ],
            default_budget: 4000,
            default_tier: PdsTier::Summary,
            max_concurrent: 10,
            timeout_secs: 30,
            searxng_url: Some("http://localhost:4040".into()),
            tavily_api_key: None,
            tavily_api_keys: Vec::new(),
            serper_api_key: None,
            serper_api_keys: Vec::new(),
            exa_api_key: None,
            exa_api_keys: Vec::new(),
            firecrawl_api_key: None,
            firecrawl_api_keys: Vec::new(),
        }
    }
}

impl Default for FetchConfig {
    fn default() -> Self {
        Self {
            user_agent: format!(
                "Fetchium/{} (https://github.com/fetchium/fetchium)",
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
            fast_model: None,
            providers: ProvidersConfig::default(),
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

// ─── Validation ──────────────────────────────────────────────────

impl HsxConfig {
    /// Validate all configuration values are within acceptable bounds.
    /// Returns a list of warnings for non-fatal issues or an error for fatal ones.
    pub fn validate(&self) -> Result<Vec<String>, String> {
        let mut warnings = Vec::new();

        // Search config bounds
        if self.search.default_budget == 0 {
            return Err("search.default_budget must be > 0".into());
        }
        if self.search.default_budget > 1_000_000 {
            warnings.push(format!(
                "search.default_budget={} is very high, may cause slow responses",
                self.search.default_budget
            ));
        }
        if self.search.max_concurrent == 0 {
            return Err("search.max_concurrent must be > 0".into());
        }
        if self.search.max_concurrent > 100 {
            warnings.push(format!(
                "search.max_concurrent={} exceeds recommended maximum of 100",
                self.search.max_concurrent
            ));
        }
        if self.search.timeout_secs == 0 {
            return Err("search.timeout_secs must be > 0".into());
        }
        if self.search.timeout_secs > 300 {
            warnings.push(format!(
                "search.timeout_secs={} is very high (>5min)",
                self.search.timeout_secs
            ));
        }

        // Fetch config bounds
        if self.fetch.timeout_secs == 0 {
            return Err("fetch.timeout_secs must be > 0".into());
        }
        if self.fetch.max_page_size == 0 {
            return Err("fetch.max_page_size must be > 0".into());
        }
        if self.fetch.max_page_size > 500 * 1024 * 1024 {
            warnings.push("fetch.max_page_size > 500MB is excessive".into());
        }
        if self.fetch.max_redirects > 50 {
            warnings.push("fetch.max_redirects > 50 may indicate a redirect loop issue".into());
        }

        // Cache config bounds
        if self.cache.enabled && self.cache.ttl_secs == 0 {
            warnings.push("cache.ttl_secs=0 means entries expire immediately".into());
        }

        // Ranking config bounds
        if !(0.0..=1.0).contains(&self.ranking.freshness_need) {
            return Err(format!(
                "ranking.freshness_need={} must be in [0.0, 1.0]",
                self.ranking.freshness_need
            ));
        }
        if self.ranking.simhash_threshold > 64 {
            return Err(format!(
                "ranking.simhash_threshold={} must be in [0, 64]",
                self.ranking.simhash_threshold
            ));
        }

        // AI config bounds
        if self.ai.max_tokens == 0 {
            return Err("ai.max_tokens must be > 0".into());
        }

        // General config bounds
        if self.general.max_results == 0 {
            return Err("general.max_results must be > 0".into());
        }
        if self.general.max_results > 10_000 {
            warnings.push(format!(
                "general.max_results={} is very high",
                self.general.max_results
            ));
        }

        Ok(warnings)
    }
}

// ─── Loading ─────────────────────────────────────────────────────

impl HsxConfig {
    /// Get the data directory, creating it if it doesn't exist.
    ///
    /// Resolution order:
    /// 1. Explicit override in config (`general.data_dir`)
    /// 2. `~/.fetchium/` (new canonical location)
    /// 2. `~/.fetchium/`
    pub fn data_dir(&self) -> PathBuf {
        if let Some(ref dir) = self.general.data_dir {
            return dir.clone();
        }

        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".fetchium")
    }

    /// Load config from the default location.
    pub fn load() -> Self {
        Self::load_from(None)
    }

    /// Load config with an optional path override.
    ///
    /// Config file resolution (when no explicit path is given):
    /// 1. `~/.fetchium/config.toml` (new canonical location)
    /// 2. Default values if file does not exist
    pub fn load_from(path: Option<&std::path::Path>) -> Self {
        let config_path = path.map(|p| p.to_path_buf()).unwrap_or_else(|| {
            let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            home.join(".fetchium").join("config.toml")
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

        // Layer 4: validation — warn on suspicious values, fail on invalid ones
        match config.validate() {
            Ok(warnings) => {
                for w in warnings {
                    tracing::warn!("Config warning: {w}");
                }
            }
            Err(e) => {
                tracing::error!("Config validation failed: {e} — using defaults");
                return Self::default();
            }
        }

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
    ///
    /// Reads `FETCHIUM_*` environment variables.
    ///
    /// Examples:
    ///   `FETCHIUM_SEARCH_DEFAULT_BUDGET=8000`
    ///   `FETCHIUM_CACHE_ENABLED=false`
    pub fn apply_env_overrides(&mut self) {
        // Helper: read env var string.
        macro_rules! env_str {
            ($name:expr) => {
                std::env::var($name).ok()
            };
        }

        if let Some(val) = env_str!("FETCHIUM_SEARCH_DEFAULT_BUDGET") {
            if let Ok(budget) = val.parse::<u32>() {
                self.search.default_budget = budget;
            }
        }
        if let Some(val) = env_str!("FETCHIUM_SEARCH_MAX_CONCURRENT") {
            if let Ok(n) = val.parse::<u32>() {
                self.search.max_concurrent = n;
            }
        }
        if let Some(val) = env_str!("FETCHIUM_SEARCH_TIMEOUT_SECS") {
            if let Ok(n) = val.parse::<u64>() {
                self.search.timeout_secs = n;
            }
        }
        if let Some(val) = env_str!("FETCHIUM_CACHE_ENABLED") {
            if let Ok(b) = val.parse::<bool>() {
                self.cache.enabled = b;
            }
        }
        if let Some(val) = env_str!("FETCHIUM_CACHE_TTL_SECS") {
            if let Ok(n) = val.parse::<u64>() {
                self.cache.ttl_secs = n;
            }
        }
        if let Some(val) = env_str!("FETCHIUM_AI_OLLAMA_HOST") {
            self.ai.ollama_host = val;
        }
        if let Some(val) = env_str!("FETCHIUM_AI_DEFAULT_MODEL") {
            self.ai.default_model = val;
        }
        if let Some(val) = env_str!("FETCHIUM_GENERAL_VERBOSE") {
            if let Ok(b) = val.parse::<bool>() {
                self.general.verbose = b;
            }
        }
        if let Some(val) = env_str!("FETCHIUM_FETCH_USER_AGENT") {
            self.fetch.user_agent = val;
        }
        if let Some(val) = env_str!("FETCHIUM_FETCH_RESPECT_ROBOTS") {
            if let Ok(b) = val.parse::<bool>() {
                self.fetch.respect_robots = b;
            }
        }
        // Search backend API keys (premium backends)
        if let Ok(val) = std::env::var("TAVILY_API_KEY") {
            if !val.is_empty() {
                self.search.tavily_api_key = Some(val);
            }
        }
        if let Ok(val) = std::env::var("SERPER_API_KEY") {
            if !val.is_empty() {
                self.search.serper_api_key = Some(val);
            }
        }
        if let Ok(val) = std::env::var("EXA_API_KEY") {
            if !val.is_empty() {
                self.search.exa_api_key = Some(val);
            }
        }
        if let Ok(val) = std::env::var("FIRECRAWL_API_KEY") {
            if !val.is_empty() {
                self.search.firecrawl_api_key = Some(val);
            }
        }
        // Provider API key overrides (also read directly by ProviderEntry::resolve_api_key)
        if let Ok(val) = std::env::var("OPENAI_API_KEY") {
            if !val.is_empty() {
                self.ai.providers.openai.api_key = Some(val);
            }
        }
        if let Ok(val) = std::env::var("ANTHROPIC_API_KEY") {
            if !val.is_empty() {
                self.ai.providers.anthropic.api_key = Some(val);
            }
        }
        if let Ok(val) = std::env::var("GEMINI_API_KEY") {
            if !val.is_empty() {
                self.ai.providers.gemini.api_key = Some(val);
            }
        }
        if let Ok(val) = std::env::var("OPENROUTER_API_KEY") {
            if !val.is_empty() {
                self.ai.providers.openrouter.api_key = Some(val);
            }
        }
        // Primary provider override (prepends to chain)
        if let Some(val) = env_str!("FETCHIUM_AI_PROVIDER") {
            if !val.is_empty() {
                self.ai.providers.fallback_chain.insert(0, val);
            }
        }
        // Chrome binary override
        if let Some(val) = env_str!("FETCHIUM_CHROME_PATH") {
            if !val.is_empty() {
                self.headless.chrome_path = Some(PathBuf::from(val));
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
    ///
    /// Returns the canonical Fetchium config path.
    pub fn config_file_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".fetchium").join("config.toml")
    }

    /// Write the current config to the config file (for `fetchium config set`).
    ///
    /// Always writes to `~/.fetchium/config.toml` — the new canonical location.
    pub fn save(&self) -> std::io::Result<()> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let path = home.join(".fetchium").join("config.toml");
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
        // The canonical default is ~/.fetchium.
        let s = dir.to_string_lossy();
        assert!(s.contains(".fetchium"));
    }

    #[test]
    fn env_override_budget_new_prefix() {
        std::env::set_var("FETCHIUM_SEARCH_DEFAULT_BUDGET", "8000");
        let mut config = HsxConfig::default();
        config.apply_env_overrides();
        assert_eq!(config.search.default_budget, 8000);
        std::env::remove_var("FETCHIUM_SEARCH_DEFAULT_BUDGET");
    }

    #[test]
    fn env_override_cache_disabled() {
        std::env::set_var("FETCHIUM_CACHE_ENABLED", "false");
        let mut config = HsxConfig::default();
        config.apply_env_overrides();
        assert!(!config.cache.enabled);
        std::env::remove_var("FETCHIUM_CACHE_ENABLED");
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
    fn config_file_path_is_fetchium() {
        let path = HsxConfig::config_file_path();
        let s = path.to_string_lossy();
        assert!(s.contains(".fetchium"));
        assert!(s.ends_with("config.toml"));
    }
}
