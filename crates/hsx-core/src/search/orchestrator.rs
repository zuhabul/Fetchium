//! Search orchestrator — parallel dispatch, dedup, ranking (PRD §15).
//!
//! Phase 2: All HTTP backends + HyperFusion 8-signal ranking.
//! Dispatches to all enabled backends in parallel via tokio::spawn,
//! deduplicates via URL normalization + SimHash, then applies HyperFusion ranking.

use crate::error::HsxResult;
use crate::http::HttpClient;
use crate::rank;
use crate::rank::fusion::{detect_intent, hyperfusion_rank};
use crate::search::arxiv::ArxivBackend;
use crate::search::bing::BingBackend;
use crate::search::brave::BraveBackend;
use crate::search::dedup::deduplicate;
use crate::search::duckduckgo::DuckDuckGoBackend;
use crate::search::github::GithubBackend;
use crate::search::google::GoogleBackend;
use crate::search::hackernews::HackerNewsBackend;
use crate::search::reddit::RedditBackend;
use crate::search::searxng::SearxngBackend;
use crate::search::stackoverflow::StackOverflowBackend;
use crate::search::wikipedia::WikipediaBackend;
use crate::search::SearchBackend;
use crate::types::{BackendId, ResultItem};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, warn};

/// Configuration for the search orchestrator.
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Max results to request from each backend (slightly more than total for dedup headroom).
    pub max_results_per_backend: u32,
    /// Total max results to return to the caller.
    pub max_total_results: u32,
    /// Per-backend search timeout.
    pub backend_timeout: Duration,
    /// Which backends to use (empty = all available HTTP backends).
    pub enabled_backends: Vec<BackendId>,
    /// SimHash threshold for near-duplicate detection (0–64, default: 6).
    pub simhash_threshold: u32,
    /// Freshness need for temporal signal (0.0–1.0, default: 0.5).
    pub freshness_need: f64,
    /// Use HyperFusion ranking (true) or legacy BM25 rerank (false).
    pub use_hyperfusion: bool,
}

/// Reliable API-based backends that never require scraping or CAPTCHAs.
/// These are always included as fallbacks when scraper backends fail.
const RELIABLE_API_BACKENDS: &[BackendId] = &[
    BackendId::Wikipedia,
    BackendId::HackerNews,
    BackendId::Reddit,
    BackendId::StackOverflow,
    BackendId::Arxiv,
];

/// All recommended default backends (scrapers + APIs).
const ALL_DEFAULT_BACKENDS: &[BackendId] = &[
    BackendId::Wikipedia,
    BackendId::HackerNews,
    BackendId::Reddit,
    BackendId::StackOverflow,
    BackendId::DuckDuckGo,
    BackendId::Google,
    BackendId::Bing,
    BackendId::Arxiv,
    BackendId::Github,
];

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_results_per_backend: 15,
            max_total_results: 10,
            backend_timeout: Duration::from_secs(15),
            enabled_backends: ALL_DEFAULT_BACKENDS.to_vec(),
            simhash_threshold: 6,
            freshness_need: 0.5,
            use_hyperfusion: true,
        }
    }
}

impl OrchestratorConfig {
    /// Create config from HsxConfig settings.
    ///
    /// Always ensures reliable API-based backends (Wikipedia, HN, Reddit, SO,
    /// Arxiv) are included even when the user's config only lists scrapers.
    /// This prevents total search failure when scrapers are CAPTCHA-blocked.
    pub fn from_hsx_config(hsx: &crate::config::HsxConfig, max_results: u32) -> Self {
        let mut enabled_backends = hsx
            .search
            .backends
            .iter()
            .filter_map(|s| parse_backend_id(s))
            .collect::<Vec<_>>();

        if enabled_backends.is_empty() {
            enabled_backends = ALL_DEFAULT_BACKENDS.to_vec();
        } else {
            // Always ensure reliable API backends are present as fallbacks
            for backend in RELIABLE_API_BACKENDS {
                if !enabled_backends.contains(backend) {
                    enabled_backends.push(backend.clone());
                }
            }
        }

        Self {
            max_results_per_backend: max_results + 5,
            max_total_results: max_results,
            backend_timeout: Duration::from_secs(hsx.search.timeout_secs),
            enabled_backends,
            simhash_threshold: hsx.ranking.simhash_threshold,
            freshness_need: hsx.ranking.freshness_need,
            use_hyperfusion: true,
        }
    }
}

/// Manages multiple search backends, dispatches in parallel, fuses results.
pub struct SearchOrchestrator {
    backends: Vec<Arc<dyn SearchBackend>>,
    config: OrchestratorConfig,
    weight_overrides: HashMap<String, f64>,
}

impl SearchOrchestrator {
    /// Create an orchestrator with backends from the given config.
    pub fn new(http_client: HttpClient, config: OrchestratorConfig) -> Self {
        Self::with_overrides(http_client, config, HashMap::new())
    }

    /// Create an orchestrator with custom ranking weight overrides.
    pub fn with_overrides(
        http_client: HttpClient,
        config: OrchestratorConfig,
        weight_overrides: HashMap<String, f64>,
    ) -> Self {
        let mut backends: Vec<Arc<dyn SearchBackend>> = Vec::new();

        for id in &config.enabled_backends {
            match id {
                BackendId::DuckDuckGo => {
                    backends.push(Arc::new(DuckDuckGoBackend::new(http_client.clone())));
                }
                BackendId::Searxng => {
                    backends.push(Arc::new(SearxngBackend::new(http_client.clone())));
                }
                BackendId::Wikipedia => {
                    backends.push(Arc::new(WikipediaBackend::new(http_client.clone())));
                }
                BackendId::Brave => {
                    backends.push(Arc::new(BraveBackend::new(http_client.clone())));
                }
                BackendId::HackerNews => {
                    backends.push(Arc::new(HackerNewsBackend::new(http_client.clone())));
                }
                BackendId::Arxiv => {
                    backends.push(Arc::new(ArxivBackend::new(http_client.clone())));
                }
                BackendId::Github => {
                    backends.push(Arc::new(GithubBackend::new(http_client.clone())));
                }
                BackendId::Reddit => {
                    backends.push(Arc::new(RedditBackend::new(http_client.clone())));
                }
                BackendId::StackOverflow => {
                    backends.push(Arc::new(StackOverflowBackend::new(http_client.clone())));
                }
                // Google: HTTP scraper by default; headless via BrowserPool when feature enabled.
                BackendId::Google => {
                    #[cfg(not(feature = "headless"))]
                    backends.push(Arc::new(GoogleBackend::new_http(http_client.clone())));
                    #[cfg(feature = "headless")]
                    warn!(
                        "Google headless backend requires BrowserPool — \
                         use SearchOrchestrator::with_pool(); falling back to HTTP"
                    );
                }
                // Bing: HTTP scraper by default; headless via BrowserPool when feature enabled.
                BackendId::Bing => {
                    #[cfg(not(feature = "headless"))]
                    backends.push(Arc::new(BingBackend::new_http(http_client.clone())));
                    #[cfg(feature = "headless")]
                    warn!(
                        "Bing headless backend requires BrowserPool — \
                         use SearchOrchestrator::with_pool(); falling back to HTTP"
                    );
                }
                BackendId::GoogleScholar => {
                    warn!("GoogleScholar backend not yet implemented — skipping");
                }
                other => {
                    warn!("Unknown backend {:?}, skipping", other);
                }
            }
        }

        if backends.is_empty() {
            warn!("No backends configured — falling back to DuckDuckGo");
            backends.push(Arc::new(DuckDuckGoBackend::new(http_client)));
        }

        Self {
            backends,
            config,
            weight_overrides,
        }
    }

    /// Execute a search across all enabled backends, returning fused results.
    ///
    /// # Pipeline
    /// 1. Dispatch query to all backends concurrently
    /// 2. Collect results with per-backend timeout (failures = empty list)
    /// 3. Deduplicate by URL normalization + SimHash content similarity
    /// 4. Rank by HyperFusion 8-signal (or BM25 if disabled)
    /// 5. Return top N results
    pub async fn search(
        &self,
        query: &str,
        max_results: Option<u32>,
    ) -> HsxResult<Vec<ResultItem>> {
        let max = max_results.unwrap_or(self.config.max_total_results);
        let per_backend = self.config.max_results_per_backend;
        let timeout_dur = self.config.backend_timeout;

        info!(
            "Orchestrator: {:?} across {} backend(s), max={}",
            query,
            self.backends.len(),
            max
        );

        // Step 1: Parallel dispatch
        let mut handles = Vec::with_capacity(self.backends.len());
        for backend in &self.backends {
            let backend = Arc::clone(backend);
            let q = query.to_string();
            handles.push(tokio::spawn(async move {
                let id = backend.id();
                match timeout(timeout_dur, backend.search(&q, per_backend)).await {
                    Ok(Ok(results)) => {
                        info!("Backend {:?}: {} results", id, results.len());
                        results
                    }
                    Ok(Err(e)) => {
                        warn!("Backend {:?} error: {e}", id);
                        Vec::new()
                    }
                    Err(_) => {
                        warn!("Backend {:?} timed out after {timeout_dur:?}", id);
                        Vec::new()
                    }
                }
            }));
        }

        // Step 2: Collect
        let mut all: Vec<ResultItem> = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(results) => all.extend(results),
                Err(e) => warn!("Backend task panicked: {e}"),
            }
        }

        if all.is_empty() {
            info!("Orchestrator: no results from any backend for {:?}", query);
            return Ok(Vec::new());
        }

        // Step 3: Deduplicate (URL normalization + SimHash)
        let deduped = deduplicate(all, self.config.simhash_threshold);

        // Step 4: Rank
        // HyperFusion computes its own BM25 via the signals module; no pre-scoring needed.
        let mut ranked = if self.config.use_hyperfusion {
            let intent = detect_intent(query);
            let mut results = deduped;
            hyperfusion_rank(
                &mut results,
                query,
                intent,
                self.config.freshness_need,
                &self.weight_overrides,
            );
            results
        } else {
            rank::rerank(deduped, query)
        };

        // Step 5: Take top N
        ranked.truncate(max as usize);

        info!(
            "Orchestrator: returning {} results for {:?}",
            ranked.len(),
            query
        );

        Ok(ranked)
    }

    /// Add a custom backend at runtime.
    pub fn add_backend(&mut self, backend: Arc<dyn SearchBackend>) {
        self.backends.push(backend);
    }
}

/// Parse a backend identifier string to a BackendId.
pub fn parse_backend_id(s: &str) -> Option<BackendId> {
    match s.to_lowercase().as_str() {
        "duckduckgo" | "ddg" | "duck" => Some(BackendId::DuckDuckGo),
        "google" => Some(BackendId::Google),
        "bing" => Some(BackendId::Bing),
        "scholar" | "google_scholar" | "googlescholar" => Some(BackendId::GoogleScholar),
        "searxng" | "searx" => Some(BackendId::Searxng),
        "wikipedia" | "wiki" => Some(BackendId::Wikipedia),
        "brave" => Some(BackendId::Brave),
        "hackernews" | "hn" => Some(BackendId::HackerNews),
        "arxiv" => Some(BackendId::Arxiv),
        "github" | "gh" => Some(BackendId::Github),
        "reddit" => Some(BackendId::Reddit),
        "stackoverflow" | "so" => Some(BackendId::StackOverflow),
        "youtube" | "yt" => Some(BackendId::YouTube),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_backend_id_variants() {
        assert_eq!(parse_backend_id("duckduckgo"), Some(BackendId::DuckDuckGo));
        assert_eq!(parse_backend_id("ddg"), Some(BackendId::DuckDuckGo));
        assert_eq!(parse_backend_id("DDG"), Some(BackendId::DuckDuckGo));
        assert_eq!(parse_backend_id("google"), Some(BackendId::Google));
        assert_eq!(parse_backend_id("scholar"), Some(BackendId::GoogleScholar));
        assert_eq!(parse_backend_id("hn"), Some(BackendId::HackerNews));
        assert_eq!(parse_backend_id("so"), Some(BackendId::StackOverflow));
        assert_eq!(parse_backend_id("arxiv"), Some(BackendId::Arxiv));
        assert_eq!(parse_backend_id("reddit"), Some(BackendId::Reddit));
        assert_eq!(parse_backend_id("wikipedia"), Some(BackendId::Wikipedia));
        assert_eq!(parse_backend_id("brave"), Some(BackendId::Brave));
        assert_eq!(parse_backend_id("github"), Some(BackendId::Github));
        assert_eq!(parse_backend_id("gh"), Some(BackendId::Github));
        assert_eq!(parse_backend_id("invalid_backend"), None);
    }

    #[test]
    fn orchestrator_config_defaults() {
        let cfg = OrchestratorConfig::default();
        assert_eq!(cfg.max_total_results, 10);
        // API-based backends are always included
        assert!(cfg.enabled_backends.contains(&BackendId::Wikipedia));
        assert!(cfg.enabled_backends.contains(&BackendId::HackerNews));
        assert!(cfg.enabled_backends.contains(&BackendId::Reddit));
        assert!(cfg.enabled_backends.contains(&BackendId::StackOverflow));
        assert!(cfg.enabled_backends.contains(&BackendId::DuckDuckGo));
        assert_eq!(cfg.simhash_threshold, 6);
        assert!(cfg.use_hyperfusion);
    }

    #[test]
    fn orchestrator_config_from_hsx() {
        let mut hsx = crate::config::HsxConfig::default();
        hsx.search.backends = vec!["duckduckgo".to_string(), "wikipedia".to_string()];
        hsx.ranking.simhash_threshold = 8;
        hsx.ranking.freshness_need = 0.8;
        let cfg = OrchestratorConfig::from_hsx_config(&hsx, 20);
        assert_eq!(cfg.max_total_results, 20);
        assert_eq!(cfg.simhash_threshold, 8);
        assert!((cfg.freshness_need - 0.8).abs() < 1e-9);
        // Explicit + always-included reliable backends
        assert!(cfg.enabled_backends.contains(&BackendId::DuckDuckGo));
        assert!(cfg.enabled_backends.contains(&BackendId::Wikipedia));
        // Reliable backends auto-injected even when not in config
        assert!(cfg.enabled_backends.contains(&BackendId::HackerNews));
        assert!(cfg.enabled_backends.contains(&BackendId::Reddit));
        assert!(cfg.enabled_backends.contains(&BackendId::StackOverflow));
        assert!(cfg.enabled_backends.contains(&BackendId::Arxiv));
    }
}
