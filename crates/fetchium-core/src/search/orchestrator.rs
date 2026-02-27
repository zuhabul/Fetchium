//! Search orchestrator — parallel dispatch, dedup, ranking (PRD §15).
//!
//! Phase 2: All HTTP backends + HyperFusion 8-signal ranking.
//! Dispatches to all enabled backends in parallel via tokio::spawn,
//! deduplicates via URL normalization + SimHash, then applies HyperFusion ranking.

use crate::error::HsxResult;
use crate::http::HttpClient;
use crate::rank;
use crate::rank::fusion::{detect_intent, hyperfusion_rank};
use crate::resilience::{Bulkhead, CircuitBreaker};
use crate::search::arxiv::ArxivBackend;
use crate::search::backend_selector::AdaptiveBackendSelector;
#[cfg(not(feature = "headless"))]
use crate::search::bing::BingBackend;
use crate::search::brave::BraveBackend;
use crate::search::dedup::deduplicate;
use crate::search::duckduckgo::DuckDuckGoBackend;
use crate::search::github::GithubBackend;
#[cfg(not(feature = "headless"))]
use crate::search::google::GoogleBackend;
use crate::search::hackernews::HackerNewsBackend;
use crate::search::reddit::RedditBackend;
use crate::search::searxng::SearxngBackend;
use crate::search::stackoverflow::StackOverflowBackend;
use crate::search::wikipedia::WikipediaBackend;
use crate::search::SearchBackend;
use crate::telemetry::PipelineMetrics;
use crate::types::{BackendId, ResultItem};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::sync::Mutex as AsyncMutex;
use tokio::time::timeout;
use tracing::{info, warn};

/// Waiters blocked on an in-flight identical query (singleflight pattern).
type InFlightWaiters = Vec<oneshot::Sender<Vec<ResultItem>>>;
/// Shared in-flight dedup map: query_key → list of waiting oneshot senders.
type InFlightMap = Arc<AsyncMutex<HashMap<String, InFlightWaiters>>>;

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
    BackendId::Searxng, // Self-hosted: unlimited free, aggregates 9 engines, zero CAPTCHA
    BackendId::Wikipedia,
    BackendId::HackerNews,
    BackendId::Reddit,
    BackendId::StackOverflow,
    BackendId::Arxiv,
];

/// All recommended default backends (scrapers + APIs).
///
/// SearXNG self-hosted (localhost:4040) is first — it aggregates Google, Bing,
/// Brave, DuckDuckGo, Wikipedia, SO, GitHub, ArXiv, Reddit in a single request.
const ALL_DEFAULT_BACKENDS: &[BackendId] = &[
    BackendId::Searxng, // Primary: self-hosted aggregator (free, unlimited, no CAPTCHA)
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
///
/// Now includes production-grade resilience:
/// - **Circuit breaker**: Automatically stops sending requests to failing backends
/// - **Bulkhead isolation**: Prevents one slow backend from starving others
/// - **Pipeline metrics**: Real-time observability for all operations
/// - **Singleflight dedup**: Identical concurrent queries share one backend search
pub struct SearchOrchestrator {
    backends: Vec<Arc<dyn SearchBackend>>,
    config: OrchestratorConfig,
    weight_overrides: HashMap<String, f64>,
    /// Circuit breaker — per-backend failure detection and fast-fail.
    circuit_breaker: CircuitBreaker,
    /// Bulkhead — per-backend concurrency isolation.
    bulkhead: Bulkhead,
    /// Pipeline metrics for observability.
    metrics: PipelineMetrics,
    /// In-flight request dedup — singleflight pattern.
    ///
    /// When two callers issue the same `(query, max)` concurrently, only one
    /// actually hits the backends. The second waits on a oneshot channel and
    /// receives the first's result when it completes.
    in_flight: InFlightMap,
    /// Adaptive Backend Selector — routes queries to intent-appropriate backends.
    ///
    /// Uses UCB1 multi-armed bandit + intent affinity to avoid dispatching
    /// GitHub to informational queries, arXiv to code queries, etc.
    backend_selector: AdaptiveBackendSelector,
}

impl SearchOrchestrator {
    /// Create an orchestrator with backends from the given config.
    pub fn new(http_client: HttpClient, config: OrchestratorConfig) -> Self {
        Self::with_overrides(http_client, config, HashMap::new())
    }

    /// Create an orchestrator with headless Chrome browser pool.
    ///
    /// Uses `BrowserPool` for Google and Bing (avoids CAPTCHA/scraper blocking),
    /// and HTTP backends for all other sources. Only available when compiled with
    /// the `headless` feature flag.
    ///
    /// # Example
    /// ```ignore
    /// let pool = Arc::new(BrowserPool::new(BrowserTier::Standard));
    /// pool.init().await?;
    /// let orchestrator = SearchOrchestrator::with_pool(http, config, pool);
    /// ```
    #[cfg(feature = "headless")]
    pub fn with_pool(
        http_client: HttpClient,
        config: OrchestratorConfig,
        pool: std::sync::Arc<crate::browser::pool::BrowserPool>,
    ) -> Self {
        Self::with_pool_and_overrides(http_client, config, pool, HashMap::new())
    }

    /// Create an orchestrator with headless pool and custom weight overrides.
    #[cfg(feature = "headless")]
    pub fn with_pool_and_overrides(
        http_client: HttpClient,
        config: OrchestratorConfig,
        pool: std::sync::Arc<crate::browser::pool::BrowserPool>,
        weight_overrides: HashMap<String, f64>,
    ) -> Self {
        let circuit_breaker = CircuitBreaker::new();
        let bulkhead = Bulkhead::new();
        let metrics = PipelineMetrics::new();
        let in_flight: InFlightMap = Arc::new(AsyncMutex::new(HashMap::new()));
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
                // Google and Bing use headless Chrome via the pool — CAPTCHA-resistant.
                BackendId::Google => {
                    backends.push(Arc::new(crate::search::google::GoogleBackend::new(
                        Arc::clone(&pool),
                    )));
                }
                BackendId::Bing => {
                    backends.push(Arc::new(crate::search::bing::BingBackend::new(Arc::clone(
                        &pool,
                    ))));
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
            circuit_breaker,
            bulkhead,
            metrics,
            in_flight,
            backend_selector: AdaptiveBackendSelector::default(),
        }
    }

    /// Create an orchestrator with custom ranking weight overrides.
    pub fn with_overrides(
        http_client: HttpClient,
        config: OrchestratorConfig,
        weight_overrides: HashMap<String, f64>,
    ) -> Self {
        let circuit_breaker = CircuitBreaker::new();
        let bulkhead = Bulkhead::new();
        let metrics = PipelineMetrics::new();
        let in_flight: InFlightMap = Arc::new(AsyncMutex::new(HashMap::new()));
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
                // Google: HTTP scraper in standard builds; headless needs with_pool().
                BackendId::Google => {
                    #[cfg(not(feature = "headless"))]
                    backends.push(Arc::new(GoogleBackend::new_http(http_client.clone())));
                    #[cfg(feature = "headless")]
                    warn!(
                        "Google headless backend needs BrowserPool — \
                         use SearchOrchestrator::with_pool(); skipping Google"
                    );
                }
                // Bing: HTTP scraper in standard builds; headless needs with_pool().
                BackendId::Bing => {
                    #[cfg(not(feature = "headless"))]
                    backends.push(Arc::new(BingBackend::new_http(http_client.clone())));
                    #[cfg(feature = "headless")]
                    warn!(
                        "Bing headless backend needs BrowserPool — \
                         use SearchOrchestrator::with_pool(); skipping Bing"
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
            circuit_breaker,
            bulkhead,
            metrics,
            in_flight,
            backend_selector: AdaptiveBackendSelector::default(),
        }
    }

    /// Get the pipeline metrics for observability.
    pub fn metrics(&self) -> &PipelineMetrics {
        &self.metrics
    }

    /// Get the circuit breaker for health monitoring.
    pub fn circuit_breaker(&self) -> &CircuitBreaker {
        &self.circuit_breaker
    }

    /// Get the bulkhead for utilization monitoring.
    pub fn bulkhead(&self) -> &Bulkhead {
        &self.bulkhead
    }

    /// Execute a search across all enabled backends, returning fused results.
    ///
    /// Applies the singleflight pattern: when two callers issue the same
    /// `(query, max)` concurrently, only one hits the backends. The second
    /// waits on a oneshot channel and receives the first's result.
    ///
    /// # Pipeline
    /// 1. Singleflight dedup — share in-flight result if same query is already running
    /// 2. Dispatch query to all backends concurrently
    /// 3. Collect results with per-backend timeout (failures = empty list)
    /// 4. Deduplicate by URL normalization + SimHash content similarity
    /// 5. Rank by HyperFusion 8-signal (or BM25 if disabled)
    /// 6. Return top N results
    pub async fn search(
        &self,
        query: &str,
        max_results: Option<u32>,
    ) -> HsxResult<Vec<ResultItem>> {
        let max = max_results.unwrap_or(self.config.max_total_results);
        let key = format!("{query}\x00{max}");

        // Singleflight check — if the same query is already in-flight, wait for it
        let rx = {
            let mut map = self.in_flight.lock().await;
            if map.contains_key(&key) {
                let (tx, rx) = oneshot::channel();
                map.get_mut(&key).expect("key just checked").push(tx);
                Some(rx)
            } else {
                map.insert(key.clone(), Vec::new());
                None
            }
        };

        if let Some(rx) = rx {
            info!("Orchestrator: singleflight — sharing in-flight result for {query:?}");
            self.metrics.inc_searches();
            return Ok(rx.await.unwrap_or_default());
        }

        // We are the primary — execute the actual search
        let result = self.execute_search(query, max).await;

        // Notify all waiters with the result (or empty vec on error)
        {
            let mut map = self.in_flight.lock().await;
            if let Some(waiters) = map.remove(&key) {
                let to_share = result.as_deref().unwrap_or(&[]).to_vec();
                for waiter in waiters {
                    let _ = waiter.send(to_share.clone());
                }
            }
        }

        result
    }

    /// Internal: run the full search pipeline without singleflight wrapping.
    async fn execute_search(&self, query: &str, max: u32) -> HsxResult<Vec<ResultItem>> {
        let per_backend = self.config.max_results_per_backend;
        let timeout_dur = self.config.backend_timeout;

        // Step 0: Detect query intent and select appropriate backends via ABS.
        // This prevents e.g. GitHub from being queried for "what is AI" definitions.
        let intent = detect_intent(query);
        let available_ids: Vec<BackendId> = self.backends.iter().map(|b| b.id()).collect();
        let unhealthy_ids: Vec<BackendId> = available_ids
            .iter()
            .filter(|id| !self.circuit_breaker.should_allow(&format!("{id:?}")))
            .cloned()
            .collect();
        let selection = self
            .backend_selector
            .select(&intent, &available_ids, &unhealthy_ids);
        let selected_set: std::collections::HashSet<String> = selection
            .backends
            .iter()
            .map(|b| format!("{b:?}"))
            .collect();

        info!(
            "Orchestrator: {:?} intent={:?}, {} of {} backend(s) selected, max={}",
            query,
            intent,
            selected_set.len(),
            self.backends.len(),
            max
        );

        self.metrics.inc_searches();
        let _search_timer = self.metrics.start_operation("search_total");

        // Step 1: Parallel dispatch with circuit breaker + bulkhead protection
        let mut handles = Vec::with_capacity(selected_set.len());
        let mut skipped_backends = Vec::new();

        for backend in &self.backends {
            let backend_id_str = format!("{:?}", backend.id());

            // Skip backends not selected by ABS for this query intent
            if !selected_set.contains(&backend_id_str) {
                skipped_backends.push(format!("ABS:{}", backend_id_str));
                continue;
            }

            // Circuit breaker check — skip backends that are known to be failing
            if !self.circuit_breaker.should_allow(&backend_id_str) {
                skipped_backends.push(backend_id_str);
                continue;
            }

            let backend = Arc::clone(backend);
            let q = query.to_string();
            let cb = self.circuit_breaker.clone();
            let bh = self.bulkhead.clone();
            let metrics = self.metrics.clone();

            handles.push(tokio::spawn(async move {
                let id = backend.id();
                let id_str = format!("{:?}", id);
                let _timer = metrics.start_operation(&format!("backend_{}", id));

                // Bulkhead — acquire per-backend concurrency slot
                let _permit = match bh.try_acquire(&id_str).await {
                    Some(p) => p,
                    None => {
                        warn!("Backend {:?}: bulkhead full, skipping", id);
                        return (id, Vec::new());
                    }
                };

                let results = match timeout(timeout_dur, backend.search(&q, per_backend)).await {
                    Ok(Ok(results)) => {
                        cb.record_success(&id_str);
                        info!("Backend {:?}: {} results", id, results.len());
                        results
                    }
                    Ok(Err(e)) => {
                        cb.record_failure(&id_str);
                        warn!("Backend {:?} error: {e}", id);
                        Vec::new()
                    }
                    Err(_) => {
                        cb.record_failure(&id_str);
                        warn!("Backend {:?} timed out after {timeout_dur:?}", id);
                        Vec::new()
                    }
                };
                (id, results)
            }));
        }

        if !skipped_backends.is_empty() {
            info!(
                "ABS/circuit-breaker: skipped {} backend(s): {:?}",
                skipped_backends.len(),
                skipped_backends
            );
        }

        // Step 2: Collect results (gracefully handle panics) + report ABS outcomes
        let mut all: Vec<ResultItem> = Vec::new();
        for handle in handles {
            match handle.await {
                Ok((backend_id, results)) => {
                    // Report outcome to ABS so UCB1 learns from this dispatch
                    let quality = (results.len() as f64 / 10.0).min(1.0);
                    self.backend_selector
                        .report_outcome(&backend_id, results.len(), quality);
                    all.extend(results);
                }
                Err(e) => {
                    self.metrics.record_error("backend_panic");
                    warn!("Backend task panicked: {e}");
                }
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
        // Reuse `intent` detected in Step 0 — no need to re-run detect_intent().
        let mut ranked = if self.config.use_hyperfusion {
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
