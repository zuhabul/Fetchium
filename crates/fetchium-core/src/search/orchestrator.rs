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
use crate::search::exa::ExaBackend;
use crate::search::firecrawl::FirecrawlBackend;
use crate::search::github::GithubBackend;
#[cfg(not(feature = "headless"))]
use crate::search::google::GoogleBackend;
use crate::search::hackernews::HackerNewsBackend;
use crate::search::reddit::RedditBackend;
use crate::search::searxng::SearxngBackend;
use crate::search::serper::SerperBackend;
use crate::search::stackoverflow::StackOverflowBackend;
use crate::search::tavily::TavilyBackend;
use crate::search::wikipedia::WikipediaBackend;
use crate::search::{SearchBackend, SearchContext, TimeRange};
use crate::telemetry::PipelineMetrics;
use crate::types::{BackendId, ResultItem};
use std::collections::{HashMap, HashSet};
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
    /// Tavily API key.
    pub tavily_api_key: Option<String>,
    /// Serper API key.
    pub serper_api_key: Option<String>,
    /// Exa API key.
    pub exa_api_key: Option<String>,
    /// Firecrawl API key.
    pub firecrawl_api_key: Option<String>,
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
    BackendId::Tavily, // Premium: AI-optimized search with content (requires API key)
    BackendId::Serper, // Premium: fast Google + Scholar + News (requires API key)
    BackendId::Exa,    // Premium: neural semantic search (requires API key)
    BackendId::Firecrawl, // Premium: search + full markdown extraction (requires API key)
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
            tavily_api_key: None,
            serper_api_key: None,
            exa_api_key: None,
            firecrawl_api_key: None,
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

        // Auto-include premium backends when their API keys are configured,
        // even if the user's config file doesn't list them explicitly.
        let premium_backends: &[(BackendId, &Option<String>)] = &[
            (BackendId::Tavily, &hsx.search.tavily_api_key),
            (BackendId::Serper, &hsx.search.serper_api_key),
            (BackendId::Exa, &hsx.search.exa_api_key),
            (BackendId::Firecrawl, &hsx.search.firecrawl_api_key),
        ];
        for (backend_id, api_key) in premium_backends {
            if api_key.is_some() && !enabled_backends.contains(backend_id) {
                // Insert premium backends at the front for priority
                enabled_backends.insert(0, backend_id.clone());
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
            tavily_api_key: hsx.search.tavily_api_key.clone(),
            serper_api_key: hsx.search.serper_api_key.clone(),
            exa_api_key: hsx.search.exa_api_key.clone(),
            firecrawl_api_key: hsx.search.firecrawl_api_key.clone(),
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
                BackendId::Tavily => {
                    if let Some(ref key) = config.tavily_api_key {
                        backends.push(Arc::new(TavilyBackend::new(
                            http_client.clone(),
                            key.clone(),
                        )));
                    }
                }
                BackendId::Serper => {
                    if let Some(ref key) = config.serper_api_key {
                        backends.push(Arc::new(SerperBackend::new(
                            http_client.clone(),
                            key.clone(),
                        )));
                    }
                }
                BackendId::Exa => {
                    if let Some(ref key) = config.exa_api_key {
                        backends.push(Arc::new(ExaBackend::new(http_client.clone(), key.clone())));
                    }
                }
                BackendId::Firecrawl => {
                    if let Some(ref key) = config.firecrawl_api_key {
                        backends.push(Arc::new(FirecrawlBackend::new(
                            http_client.clone(),
                            key.clone(),
                        )));
                    }
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
                BackendId::Tavily => {
                    if let Some(ref key) = config.tavily_api_key {
                        backends.push(Arc::new(TavilyBackend::new(
                            http_client.clone(),
                            key.clone(),
                        )));
                    } else {
                        info!("Tavily backend skipped — no TAVILY_API_KEY configured");
                    }
                }
                BackendId::Serper => {
                    if let Some(ref key) = config.serper_api_key {
                        backends.push(Arc::new(SerperBackend::new(
                            http_client.clone(),
                            key.clone(),
                        )));
                    } else {
                        info!("Serper backend skipped — no SERPER_API_KEY configured");
                    }
                }
                BackendId::Exa => {
                    if let Some(ref key) = config.exa_api_key {
                        backends.push(Arc::new(ExaBackend::new(http_client.clone(), key.clone())));
                    } else {
                        info!("Exa backend skipped — no EXA_API_KEY configured");
                    }
                }
                BackendId::Firecrawl => {
                    if let Some(ref key) = config.firecrawl_api_key {
                        backends.push(Arc::new(FirecrawlBackend::new(
                            http_client.clone(),
                            key.clone(),
                        )));
                    } else {
                        info!("Firecrawl backend skipped — no FIRECRAWL_API_KEY configured");
                    }
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

        // Compute effective freshness based on intent:
        // CurrentEvents → boost to at least 0.9; Factual → cap at 0.3
        let effective_freshness = match intent {
            rank::fusion::QueryIntent::CurrentEvents => self.config.freshness_need.max(0.9),
            rank::fusion::QueryIntent::Factual => self.config.freshness_need.min(0.3),
            _ => self.config.freshness_need,
        };

        // Build search context for backends that support date filtering
        let time_range = match intent {
            rank::fusion::QueryIntent::CurrentEvents => Some(TimeRange::Year),
            _ => None,
        };
        let search_ctx = SearchContext { intent, time_range };

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
            let ctx = search_ctx.clone();

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

                let backend_timeout = backend_timeout_for(&id, timeout_dur);
                let results = match timeout(
                    backend_timeout,
                    backend.search_with_context(&q, per_backend, &ctx),
                )
                .await
                {
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
                        warn!("Backend {:?} timed out after {backend_timeout:?}", id);
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

        let rescue_threshold = ((max as usize) / 2).max(3);
        if all.len() <= rescue_threshold {
            info!(
                "Orchestrator: low recall from selected backends for {:?} ({} results) — running fallback sweep",
                query
                ,
                all.len()
            );
            let mut rescue_handles = Vec::new();
            for backend in &self.backends {
                let id_str = format!("{:?}", backend.id());
                if selected_set.contains(&id_str) {
                    continue;
                }
                // Respect circuit breaker on fallback too.
                if !self.circuit_breaker.should_allow(&id_str) {
                    continue;
                }
                let backend = Arc::clone(backend);
                let q = query.to_string();
                let ctx = search_ctx.clone();
                let timeout_dur = backend_timeout_for(&backend.id(), timeout_dur);
                rescue_handles.push(tokio::spawn(async move {
                    match timeout(
                        timeout_dur,
                        backend.search_with_context(&q, per_backend, &ctx),
                    )
                    .await
                    {
                        Ok(Ok(results)) => results,
                        _ => Vec::new(),
                    }
                }));
            }
            for h in rescue_handles {
                if let Ok(results) = h.await {
                    all.extend(results);
                }
            }
            if all.is_empty() {
                info!("Orchestrator: no results from any backend for {:?}", query);
                return Ok(Vec::new());
            }
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
                effective_freshness,
                &self.weight_overrides,
            );
            results
        } else {
            rank::rerank(deduped, query)
        };

        // Step 5: Filter low-relevance garbage results
        // (a) Score threshold: discard results with near-zero HyperFusion score.
        // (b) Title-term check: require at least one non-trivial query word in
        //     title+snippet. Prevents high-authority domains gaming BM25 on stopwords
        //     (e.g. arxiv.org "best practices" paper ranking for "best Rust async runtimes").
        let query_content_words = extract_content_words(query);
        let ranked_before_filter = ranked.clone();
        let pre_filter_count = ranked.len();
        let multilingual_query = has_non_ascii_letters(query);
        let strict_min_term_matches = if multilingual_query {
            0
        } else if query_content_words.len() >= 5 {
            2
        } else {
            1
        };
        let relaxed_min_term_matches = strict_min_term_matches.min(1);
        ranked.retain(|r| {
            // (a) score threshold
            if r.score.unwrap_or(0.0) < 0.10 {
                return false;
            }
            // (b) title-term check — only apply when we have meaningful query words
            if strict_min_term_matches > 0 && !query_content_words.is_empty() {
                let haystack = format!("{} {}", r.title.to_lowercase(), r.snippet.to_lowercase());
                let matches = query_content_words
                    .iter()
                    .filter(|w| haystack.contains(w.as_str()))
                    .count();
                if matches < strict_min_term_matches {
                    tracing::debug!(
                        "Filtered title-mismatch: {:?} (matches={}, need {}, words={:?})",
                        r.title,
                        matches,
                        strict_min_term_matches,
                        &query_content_words[..query_content_words.len().min(3)]
                    );
                    return false;
                }
            }
            true
        });
        if ranked.len() < pre_filter_count {
            tracing::debug!(
                "Filtered {} low-relevance/off-topic results",
                pre_filter_count - ranked.len()
            );
        }
        if ranked.len() < 5 && pre_filter_count >= 5 {
            tracing::debug!(
                "Relevance filter was too strict ({} -> {}) — relaxing thresholds",
                pre_filter_count,
                ranked.len()
            );
            ranked = ranked_before_filter.clone();
            ranked.retain(|r| {
                if r.score.unwrap_or(0.0) < 0.02 {
                    return false;
                }
                if relaxed_min_term_matches == 0 || query_content_words.is_empty() {
                    return true;
                }
                let haystack = format!("{} {}", r.title.to_lowercase(), r.snippet.to_lowercase());
                query_content_words
                    .iter()
                    .filter(|w| haystack.contains(w.as_str()))
                    .count()
                    >= relaxed_min_term_matches
            });
        }
        if ranked.len() < 3 && pre_filter_count >= 3 {
            tracing::debug!(
                "Relevance filtering left too few results ({}). Backfilling from pre-filter set.",
                ranked.len()
            );
            let mut backfill = ranked_before_filter
                .iter()
                .filter(|r| r.score.unwrap_or(0.0) >= 0.02)
                .cloned()
                .collect::<Vec<_>>();
            backfill.truncate(3);
            if backfill.len() > ranked.len() {
                ranked = backfill;
            }
        }

        // Step 5b: For temporal queries, enforce recency and cap static-content sources.
        if intent == rank::fusion::QueryIntent::CurrentEvents {
            // Extract the year from the query (e.g., "2025" from "breakthroughs 2025")
            let query_year = extract_query_year(query);
            let cap_static_sources = should_cap_static_sources(multilingual_query, &ranked);

            let mut wiki_count = 0u32;
            let mut arxiv_count = 0u32;
            ranked.retain(|r| {
                // Cap Wikipedia and ArXiv to max 1 each
                if cap_static_sources && r.url.contains("wikipedia.org") {
                    wiki_count += 1;
                    return wiki_count <= 1;
                }
                if cap_static_sources && r.url.contains("arxiv.org") {
                    arxiv_count += 1;
                    return arxiv_count <= 1;
                }
                // If query mentions a year, filter out results published in older years
                if let Some(qy) = query_year {
                    if let Some(ref date) = r.published_date {
                        if let Some(pub_year) = date.get(..4).and_then(|s| s.parse::<u32>().ok()) {
                            if pub_year + 1 < qy {
                                return false; // published 2+ years before query year
                            }
                        }
                    }
                }
                true
            });
        }

        // Step 5c: If filtering/caps left fewer than requested results, backfill from
        // the pre-filter ranked pool without extra network calls.
        if ranked.len() < max as usize {
            let mut seen: HashSet<String> = ranked.iter().map(|r| r.url.clone()).collect();
            for candidate in &ranked_before_filter {
                if ranked.len() >= max as usize {
                    break;
                }
                if candidate.score.unwrap_or(0.0) < 0.02 {
                    continue;
                }
                if seen.insert(candidate.url.clone()) {
                    ranked.push(candidate.clone());
                }
            }
        }

        // Step 5d: hard safety net — if filters produced zero but we had candidates,
        // return the top pre-filter items to avoid empty responses.
        if ranked.is_empty() && !ranked_before_filter.is_empty() {
            ranked = ranked_before_filter
                .iter()
                .take(max as usize)
                .cloned()
                .collect();
        }

        // Step 6: Diversify domains in top-N, then take top N.
        ranked = diversify_by_domain(ranked, max as usize);
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
        "tavily" => Some(BackendId::Tavily),
        "serper" => Some(BackendId::Serper),
        "exa" => Some(BackendId::Exa),
        "firecrawl" => Some(BackendId::Firecrawl),
        _ => None,
    }
}

/// Extract a 4-digit year from the query (e.g., "breakthroughs 2025" → Some(2025)).
fn extract_query_year(query: &str) -> Option<u32> {
    query
        .split_whitespace()
        .filter_map(|w| {
            let w = w.trim_matches(|c: char| !c.is_ascii_digit());
            if w.len() == 4 {
                w.parse::<u32>().ok().filter(|y| (2020..=2030).contains(y))
            } else {
                None
            }
        })
        .next()
}

/// Extract meaningful content words from a query, stripping English stopwords.
///
/// Used by the title-term relevance filter to ensure results contain at least
/// one substantive query term. Returns lowercase words with length >= 3.
fn extract_content_words(query: &str) -> Vec<String> {
    const STOPWORDS: &[&str] = &[
        "a", "an", "the", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
        "is", "are", "was", "were", "be", "been", "have", "has", "do", "does", "did", "will",
        "would", "could", "should", "may", "might", "what", "which", "who", "when", "where", "how",
        "why", "best", "top", "good", "vs", "vs.", "versus", "most", "more", "less", "all", "any",
        "some", "many", "can", "its", "it", "this", "that", "than", "then", "from", "into",
        "about", "like", "get", "use", "used",
    ];
    query
        .split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|w| w.len() >= 3 && !STOPWORDS.contains(&w.as_str()))
        .filter(|w| !w.chars().all(|c| c.is_ascii_digit()))
        .collect()
}

/// True when query contains non-ASCII alphabetic characters (e.g. CJK, accented scripts).
///
/// Used to avoid over-applying ASCII word-match filters to multilingual queries.
fn has_non_ascii_letters(query: &str) -> bool {
    query.chars().any(|c| c.is_alphabetic() && !c.is_ascii())
}

/// Whether to cap static-content domains (Wikipedia/ArXiv) for temporal queries.
///
/// Disable caps for multilingual queries or when there are no non-static alternatives,
/// to preserve recall on sparse or cross-lingual retrieval.
fn should_cap_static_sources(multilingual_query: bool, ranked: &[ResultItem]) -> bool {
    if multilingual_query {
        return false;
    }
    ranked
        .iter()
        .any(|r| !r.url.contains("wikipedia.org") && !r.url.contains("arxiv.org"))
}

/// Reorder results to improve top-page domain diversity without reducing recall.
///
/// Keeps per-domain relevance order and round-robins across domains.
fn diversify_by_domain(results: Vec<ResultItem>, max: usize) -> Vec<ResultItem> {
    use std::collections::VecDeque;

    if results.len() <= 2 || max <= 2 {
        return results;
    }

    let mut buckets: HashMap<String, VecDeque<ResultItem>> = HashMap::new();
    let mut domain_order: Vec<String> = Vec::new();

    for item in results {
        let key = domain_key(&item.url);
        let bucket = buckets.entry(key.clone()).or_insert_with(|| {
            domain_order.push(key.clone());
            VecDeque::new()
        });
        bucket.push_back(item);
    }

    let mut out = Vec::with_capacity(max);
    loop {
        let mut progressed = false;
        for key in &domain_order {
            if out.len() >= max {
                break;
            }
            if let Some(bucket) = buckets.get_mut(key) {
                if let Some(item) = bucket.pop_front() {
                    out.push(item);
                    progressed = true;
                }
            }
        }
        if !progressed || out.len() >= max {
            break;
        }
    }
    out
}

fn domain_key(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_ascii_lowercase()))
        .unwrap_or_else(|| url.to_ascii_lowercase())
}

/// Per-backend timeout cap to avoid long tail latency from flaky scrapers.
fn backend_timeout_for(id: &BackendId, default_timeout: Duration) -> Duration {
    let cap = match id {
        BackendId::DuckDuckGo | BackendId::Google | BackendId::Bing => Duration::from_secs(7),
        BackendId::Reddit => Duration::from_secs(3),
        BackendId::StackOverflow => Duration::from_secs(4),
        BackendId::Searxng => Duration::from_secs(3),
        _ => default_timeout,
    };
    default_timeout.min(cap)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_content_words_strips_stopwords() {
        let words = extract_content_words("best Rust async runtimes 2025");
        assert!(words.contains(&"rust".to_string()));
        assert!(words.contains(&"async".to_string()));
        assert!(words.contains(&"runtimes".to_string()));
        assert!(!words.contains(&"2025".to_string()));
        assert!(!words.contains(&"best".to_string())); // stopword
    }

    #[test]
    fn extract_content_words_short_words_removed() {
        let words = extract_content_words("is it a good idea");
        // "is", "it", "a", "good" are stopwords or <3 chars
        assert!(words.is_empty() || !words.contains(&"is".to_string()));
    }

    #[test]
    fn has_non_ascii_letters_detects_multilingual_queries() {
        assert!(has_non_ascii_letters("最新の生成aiニュース 2026"));
        assert!(has_non_ascii_letters(
            "quelles sont les avancees en ia générative"
        ));
        assert!(!has_non_ascii_letters("latest ai news 2026"));
    }

    #[test]
    fn should_cap_static_sources_disabled_for_multilingual() {
        let ranked = vec![ResultItem {
            title: "x".into(),
            url: "https://arxiv.org/abs/1234.5678".into(),
            snippet: "".into(),
            rank: 1,
            backend: BackendId::Arxiv,
            score: Some(0.5),
            published_date: None,
        }];
        assert!(!should_cap_static_sources(true, &ranked));
    }

    #[test]
    fn should_cap_static_sources_disabled_without_alternatives() {
        let ranked = vec![
            ResultItem {
                title: "x".into(),
                url: "https://arxiv.org/abs/1234.5678".into(),
                snippet: "".into(),
                rank: 1,
                backend: BackendId::Arxiv,
                score: Some(0.5),
                published_date: None,
            },
            ResultItem {
                title: "y".into(),
                url: "https://en.wikipedia.org/wiki/Test".into(),
                snippet: "".into(),
                rank: 2,
                backend: BackendId::Wikipedia,
                score: Some(0.4),
                published_date: None,
            },
        ];
        assert!(!should_cap_static_sources(false, &ranked));
    }

    #[test]
    fn should_cap_static_sources_enabled_with_non_static_mix() {
        let ranked = vec![
            ResultItem {
                title: "x".into(),
                url: "https://arxiv.org/abs/1234.5678".into(),
                snippet: "".into(),
                rank: 1,
                backend: BackendId::Arxiv,
                score: Some(0.5),
                published_date: None,
            },
            ResultItem {
                title: "z".into(),
                url: "https://example.com/news".into(),
                snippet: "".into(),
                rank: 2,
                backend: BackendId::Searxng,
                score: Some(0.6),
                published_date: None,
            },
        ];
        assert!(should_cap_static_sources(false, &ranked));
    }

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

    #[test]
    fn backend_timeout_caps_slow_scrapers() {
        let default = Duration::from_secs(30);
        assert_eq!(
            backend_timeout_for(&BackendId::DuckDuckGo, default),
            Duration::from_secs(7)
        );
        assert_eq!(
            backend_timeout_for(&BackendId::Reddit, default),
            Duration::from_secs(3)
        );
        assert_eq!(
            backend_timeout_for(&BackendId::StackOverflow, default),
            Duration::from_secs(4)
        );
        assert_eq!(
            backend_timeout_for(&BackendId::Searxng, default),
            Duration::from_secs(3)
        );
        assert_eq!(
            backend_timeout_for(&BackendId::Wikipedia, default),
            Duration::from_secs(30)
        );
    }

    #[test]
    fn diversify_by_domain_round_robins() {
        let mk = |url: &str, rank: u32| ResultItem {
            title: format!("t{rank}"),
            url: url.to_string(),
            snippet: String::new(),
            rank,
            backend: BackendId::Searxng,
            score: Some(1.0),
            published_date: None,
        };
        let input = vec![
            mk("https://a.com/1", 1),
            mk("https://a.com/2", 2),
            mk("https://a.com/3", 3),
            mk("https://b.com/1", 4),
            mk("https://c.com/1", 5),
        ];
        let out = diversify_by_domain(input, 5);
        let hosts: Vec<String> = out.iter().map(|r| domain_key(&r.url)).collect();
        assert_eq!(hosts[0], "a.com");
        assert_eq!(hosts[1], "b.com");
        assert_eq!(hosts[2], "c.com");
    }
}
