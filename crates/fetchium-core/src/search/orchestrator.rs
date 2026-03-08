//! Search orchestrator — parallel dispatch, dedup, ranking (PRD §15).
//!
//! Phase 2: All HTTP backends + HyperFusion 8-signal ranking.
//! Dispatches to all enabled backends in parallel via tokio::spawn,
//! deduplicates via URL normalization + SimHash, then applies HyperFusion ranking.

use crate::error::HsxResult;
use crate::http::HttpClient;
use crate::query::locale::detect_query_locale;
use crate::rank;
use crate::rank::fusion::{detect_intent, hyperfusion_rank};
use crate::resilience::{Bulkhead, CircuitBreaker};
use crate::search::arxiv::ArxivBackend;
use crate::search::backend_selector::AdaptiveBackendSelector;
#[cfg(not(feature = "headless"))]
use crate::search::bing::BingBackend;
use crate::search::brave::BraveBackend;
use crate::search::dedup::{deduplicate, normalize_url};
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
    BackendId::DuckDuckGo,
    BackendId::Wikipedia,
    BackendId::HackerNews,
    BackendId::Reddit,
    BackendId::StackOverflow,
    BackendId::Bing,
    BackendId::Arxiv,
    BackendId::Github,
];

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_results_per_backend: 15,
            max_total_results: 10,
            backend_timeout: Duration::from_secs(6),
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
    pub fn from_fetchium_config(
        fetchium_config: &crate::config::HsxConfig,
        max_results: u32,
    ) -> Self {
        let mut enabled_backends = fetchium_config
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
            (BackendId::Tavily, &fetchium_config.search.tavily_api_key),
            (BackendId::Serper, &fetchium_config.search.serper_api_key),
            (BackendId::Exa, &fetchium_config.search.exa_api_key),
            (
                BackendId::Firecrawl,
                &fetchium_config.search.firecrawl_api_key,
            ),
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
            backend_timeout: Duration::from_secs(fetchium_config.search.timeout_secs),
            enabled_backends,
            simhash_threshold: fetchium_config.ranking.simhash_threshold,
            freshness_need: fetchium_config.ranking.freshness_need,
            use_hyperfusion: true,
            tavily_api_key: fetchium_config.search.tavily_api_key.clone(),
            serper_api_key: fetchium_config.search.serper_api_key.clone(),
            exa_api_key: fetchium_config.search.exa_api_key.clone(),
            firecrawl_api_key: fetchium_config.search.firecrawl_api_key.clone(),
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

        // Step 0a: Autoprompt — rewrite query for better backend results.
        // Normalizes questions, removes fillers, expands abbreviations.
        let ap = crate::query::autoprompt::autoprompt(query);
        let effective_query = &ap.rewritten;
        if ap.changed {
            info!("Autoprompt: {:?} → {:?}", query, effective_query);
        }

        // Step 0b: Detect query intent and select appropriate backends via ABS.
        // This prevents e.g. GitHub from being queried for "what is AI" definitions.
        let intent = detect_intent(effective_query);

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
        let locale = detect_query_locale(effective_query).map(|s| s.to_string());
        let search_ctx = SearchContext {
            intent,
            time_range,
            locale,
        };

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
            effective_query,
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

            // DDG runs conditionally after Google result check (Google-first cascade).
            // DDG internally cascades: direct_full → direct_lite → residential_full → residential_lite.
            // We only pay DataImpulse GB on DDG when Google AND DDG-direct both fail.
            if backend.id() == BackendId::DuckDuckGo {
                skipped_backends.push("DDG:deferred-to-google-first-gate".to_string());
                continue;
            }

            let backend = Arc::clone(backend);
            let q = effective_query.to_string();
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

        // Step 2: Collect results with early-return optimization.
        // Return as soon as we have enough results from quality backends OR a time budget expires.
        // Don't wait for any specific backend — return early when we have enough for ranking.
        let mut all: Vec<ResultItem> = Vec::new();
        let target_results = (max as usize) * 3; // 3x headroom for dedup/ranking
        let early_deadline = tokio::time::Instant::now() + Duration::from_millis(4500);
        let mut quality_backends_responded: u32 = 0;

        let mut remaining: futures::stream::FuturesUnordered<_> = handles.into_iter().collect();

        use futures::StreamExt;
        while let Some(result) = tokio::select! {
            r = remaining.next() => r,
            _ = tokio::time::sleep_until(early_deadline) => {
                if all.len() >= max as usize {
                    info!("Orchestrator: early-return deadline hit with {} results, {} backends still pending",
                          all.len(), remaining.len());
                    None
                } else {
                    // Not enough results yet, keep waiting for backends
                    remaining.next().await
                }
            }
        } {
            match result {
                Ok((backend_id, results)) => {
                    let quality = (results.len() as f64 / 10.0).min(1.0);
                    self.backend_selector
                        .report_outcome(&backend_id, results.len(), quality);
                    // Count quality backends (not Wikipedia/ArXiv which often produce
                    // noise that gets filtered later)
                    if !results.is_empty()
                        && !matches!(backend_id, BackendId::Wikipedia | BackendId::Arxiv)
                    {
                        quality_backends_responded += 1;
                    }
                    all.extend(results);
                    // Early exit when we have enough results from quality web search backends.
                    // SearXNG (Startpage/Yahoo) is our primary quality source — make sure
                    // it responds before we return early.
                    if all.len() >= target_results && quality_backends_responded >= 3 {
                        info!(
                            "Orchestrator: early-return with {} results, {} backends still pending",
                            all.len(),
                            remaining.len()
                        );
                        break;
                    }
                }
                Err(e) => {
                    self.metrics.record_error("backend_panic");
                    warn!("Backend task panicked: {e}");
                }
            }
        }

        // Google-first gate: dispatch DDG only if Google returned fewer than 5 results.
        // DDG's 4-tier cascade handles cost internally: direct_full → direct_lite → residential_full → residential_lite.
        // Residential proxy is only charged when BOTH Google and DDG-direct fail.
        {
            let google_count = all
                .iter()
                .filter(|r| r.backend == BackendId::Google)
                .count();
            let ddg_in_selected = selected_set.contains("DuckDuckGo");
            let ddg_allowed = self.circuit_breaker.should_allow("DuckDuckGo");

            if ddg_in_selected && ddg_allowed {
                if google_count >= 5 {
                    info!("Orchestrator: Google={google_count} results — DDG skipped (cost save)");
                } else {
                    info!("Orchestrator: Google={google_count} results — dispatching DDG cascade");
                    if let Some(ddg_backend) = self
                        .backends
                        .iter()
                        .find(|b| b.id() == BackendId::DuckDuckGo)
                    {
                        let ddg = Arc::clone(ddg_backend);
                        let q = effective_query.to_string();
                        let ctx = search_ctx.clone();
                        match timeout(
                            Duration::from_secs(5),
                            ddg.search_with_context(&q, per_backend, &ctx),
                        )
                        .await
                        {
                            Ok(Ok(ddg_results)) => {
                                self.circuit_breaker.record_success("DuckDuckGo");
                                self.backend_selector.report_outcome(
                                    &BackendId::DuckDuckGo,
                                    ddg_results.len(),
                                    (ddg_results.len() as f64 / 10.0).min(1.0),
                                );
                                info!(
                                    "DDG conditional: {} results for {:?}",
                                    ddg_results.len(),
                                    effective_query
                                );
                                all.extend(ddg_results);
                            }
                            Ok(Err(e)) => {
                                self.circuit_breaker.record_failure("DuckDuckGo");
                                warn!("DDG conditional error: {e}");
                            }
                            Err(_) => {
                                self.circuit_breaker.record_failure("DuckDuckGo");
                                warn!("DDG conditional timed out");
                            }
                        }
                    }
                }
            }
        }

        let rescue_threshold = ((max as usize) / 2).max(3);
        if all.len() <= rescue_threshold {
            info!(
                "Orchestrator: low recall from selected backends for {:?} ({} results) — running fallback sweep",
                effective_query,
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
                let q = effective_query.to_string();
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
                info!(
                    "Orchestrator: no results from any backend for {:?}",
                    effective_query
                );
                return Ok(Vec::new());
            }
        }

        // Step 2b: Reciprocal Rank Fusion (RRF) pre-scoring.
        // Compute RRF score per URL across all backends: score = Σ 1/(rank_i + k).
        // Results appearing in multiple backends get higher scores. k=60 is the
        // standard constant from Cormack, Clarke & Buettcher (2009).
        let rrf_scores: HashMap<String, f64> = {
            let mut url_scores: HashMap<String, f64> = HashMap::new();
            const RRF_K: f64 = 60.0;
            for item in &all {
                let norm = normalize_url(&item.url);
                let rank = item.rank.max(1) as f64; // rank is 1-based from backends
                *url_scores.entry(norm).or_insert(0.0) += 1.0 / (rank + RRF_K);
            }
            url_scores
        };

        // Step 3: Deduplicate (URL normalization + SimHash)
        let deduped = deduplicate(all, self.config.simhash_threshold);

        // Step 4: Rank
        // HyperFusion computes its own BM25 via the signals module; no pre-scoring needed.
        // Reuse `intent` detected in Step 0 — no need to re-run detect_intent().
        let mut ranked = if self.config.use_hyperfusion {
            let mut results = deduped;
            hyperfusion_rank(
                &mut results,
                effective_query,
                intent,
                effective_freshness,
                &self.weight_overrides,
            );
            results
        } else {
            rank::rerank(deduped, effective_query)
        };

        // Step 4b: Blend RRF pre-scores with HyperFusion scores.
        // RRF rewards results that appear in multiple backends (cross-source consensus).
        // Blend: 80% HyperFusion + 20% normalized RRF. Enough to boost multi-backend
        // results without overriding relevance signals.
        {
            let rrf_max = rrf_scores.values().cloned().fold(0.0_f64, f64::max);
            if rrf_max > 1e-9 {
                for r in &mut ranked {
                    let norm = normalize_url(&r.url);
                    let rrf = rrf_scores.get(&norm).copied().unwrap_or(0.0) / rrf_max;
                    if let Some(score) = r.score.as_mut() {
                        *score = *score * 0.80 + rrf * 0.20;
                    }
                }
                ranked.sort_by(|a, b| {
                    b.score
                        .unwrap_or(0.0)
                        .partial_cmp(&a.score.unwrap_or(0.0))
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
        }

        // Step 4c: Post-retrieval relevance boost (Perplexity-style).
        // Lightweight term-overlap scoring applied AFTER HyperFusion to penalize
        // results where query terms barely appear. This catches pollution from high-authority
        // domains that scored well on authority/consensus but have low actual relevance.
        {
            let boost_words = extract_content_words(effective_query);
            if boost_words.len() >= 2 {
                for r in &mut ranked {
                    let haystack =
                        format!("{} {}", r.title.to_lowercase(), r.snippet.to_lowercase());
                    let matches = boost_words
                        .iter()
                        .filter(|w| haystack.contains(w.as_str()))
                        .count();
                    let coverage = matches as f64 / boost_words.len() as f64;
                    // Scale: 100% coverage → 1.0x (no change), 0% → 0.3x penalty
                    // Steeper penalty catches off-topic results from high-authority domains
                    let boost = 0.3 + (coverage * 0.7);
                    if let Some(score) = r.score.as_mut() {
                        *score *= boost;
                    }
                }
                // Re-sort by adjusted scores
                ranked.sort_by(|a, b| {
                    b.score
                        .unwrap_or(0.0)
                        .partial_cmp(&a.score.unwrap_or(0.0))
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
        }

        // Step 5: Filter low-relevance garbage results
        // (a) Score threshold: discard results with near-zero HyperFusion score.
        // (b) Title-term check: require at least one non-trivial query word in
        //     title+snippet. Prevents high-authority domains gaming BM25 on stopwords.
        // (c) Homepage filter: remove generic homepages (e.g. foxnews.com, nbcnews.com)
        //     that backends sometimes return when the actual article URL is unavailable.
        // (d) Spam domain blocklist: known junk forum/spam sites
        // (e) Image-only URL filter: Reddit/HN image posts with no textual content
        let query_content_words = extract_content_words(effective_query);
        let ranked_before_filter = ranked.clone();
        let pre_filter_count = ranked.len();
        let multilingual_query = has_non_ascii_letters(query);

        // Step 4b: Language-aware filtering (Exa-style).
        // For multilingual queries, boost results in the same script/language
        // and demote results that are clearly in a different language.
        if multilingual_query {
            let query_scripts = detect_scripts(query);
            ranked.retain(|r| {
                let result_text = format!("{} {}", r.title, r.snippet);
                let result_scripts = detect_scripts(&result_text);
                // Keep result if it shares at least one non-Latin script with the query,
                // OR if it's from a known authoritative domain (Wikipedia handles multilingual well)
                let shares_script = query_scripts.iter().any(|s| result_scripts.contains(s));
                let is_authoritative = r.url.contains("wikipedia.org");
                shares_script || is_authoritative || result_scripts.contains(&Script::Latin)
            });
            // If language filtering removed too many, fall back to unfiltered
            if ranked.len() < 3 {
                ranked = ranked_before_filter.clone();
            }
        }
        // Proportional term matching: for long queries require ~40% of content words
        let strict_min_term_matches = if multilingual_query {
            // For multilingual, still require 1 match if we have content words
            if query_content_words.len() >= 2 {
                1
            } else {
                0
            }
        } else if query_content_words.len() >= 7 {
            (query_content_words.len() * 2 / 5).max(3) // ~40% for very long queries
        } else if query_content_words.len() >= 5 {
            3
        } else if query_content_words.len() >= 3 {
            2
        } else {
            1
        };
        // Relaxed threshold: for comparison/code queries with many entities,
        // still require more than 1 term to avoid single-entity Wikipedia pages.
        let relaxed_min_term_matches = if query_content_words.len() >= 4 {
            2 // For multi-entity queries, require 2 even in relaxed mode
        } else {
            strict_min_term_matches.min(1)
        };
        ranked.retain(|r| {
            // (a) score threshold
            if r.score.unwrap_or(0.0) < 0.10 {
                return false;
            }
            // (d) Spam domain blocklist
            if is_spam_domain(&r.url) {
                tracing::debug!("Filtered spam domain: {:?} ({})", r.title, r.url);
                return false;
            }
            // (e) Image-only URL filter (e.g. i.redd.it/xxx.png)
            if is_image_url(&r.url) {
                tracing::debug!("Filtered image URL: {:?} ({})", r.title, r.url);
                return false;
            }
            // (c) Homepage filter: reject URLs that are just domain roots with no path
            if is_homepage_url(&r.url) {
                tracing::debug!("Filtered homepage: {:?} ({})", r.title, r.url);
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
                // Always apply safety filters even in relaxed mode
                if is_spam_domain(&r.url) || is_image_url(&r.url) || is_homepage_url(&r.url) {
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
                .filter(|r| {
                    if r.score.unwrap_or(0.0) < 0.02
                        || is_spam_domain(&r.url)
                        || is_image_url(&r.url)
                    {
                        return false;
                    }
                    // Even in backfill, require at least 1 content word match
                    if !query_content_words.is_empty() {
                        let haystack =
                            format!("{} {}", r.title.to_lowercase(), r.snippet.to_lowercase());
                        query_content_words
                            .iter()
                            .any(|w| haystack.contains(w.as_str()))
                    } else {
                        true
                    }
                })
                .cloned()
                .collect::<Vec<_>>();
            backfill.truncate(3);
            if backfill.len() > ranked.len() {
                ranked = backfill;
            }
        }

        // Step 5b: Cap Wikipedia/ArXiv/GitHub to prevent over-representation.
        // Practical/casual queries get stricter Wikipedia caps since Wikipedia
        // articles about generic topics (Marathon, Paris) drown out actionable results.
        {
            let query_year = extract_query_year(effective_query);
            let is_temporal = intent == rank::fusion::QueryIntent::CurrentEvents;
            let is_casual = intent == rank::fusion::QueryIntent::Casual;
            let is_practical = is_practical_query(effective_query);
            let is_howto = intent == rank::fusion::QueryIntent::HowTo;
            let is_code = intent == rank::fusion::QueryIntent::Code;
            let is_comparison = intent == rank::fusion::QueryIntent::Comparison;
            // HowTo/Code/Casual: Wikipedia articles rarely useful; Comparison allows 1
            let wiki_cap: u32 = if is_temporal || is_practical || is_casual || is_howto || is_code {
                0
            } else if is_comparison {
                1
            } else {
                2
            };
            // GitHub repos: only useful for Code queries, not tutorials/HowTo
            let github_cap: u32 = if is_howto || is_comparison || is_casual {
                0
            } else if is_code {
                2
            } else {
                1
            };
            let arxiv_cap: u32 = if is_temporal { 1 } else { 2 };

            let mut wiki_count = 0u32;
            let mut arxiv_count = 0u32;
            let mut github_count = 0u32;
            ranked.retain(|r| {
                if r.url.contains("wikipedia.org") {
                    wiki_count += 1;
                    if wiki_count > wiki_cap {
                        return false;
                    }
                }
                if r.url.contains("arxiv.org") {
                    arxiv_count += 1;
                    if arxiv_count > arxiv_cap {
                        return false;
                    }
                }
                if r.url.contains("github.com/") && !r.url.contains("github.com/topics") {
                    github_count += 1;
                    if github_count > github_cap {
                        return false;
                    }
                }
                // Staleness filter: if query mentions a year, drop outdated results.
                // CurrentEvents: strict (1-year tolerance).
                // All other queries: loose (2-year tolerance) — catches old product reviews.
                if let Some(qy) = query_year {
                    if let Some(ref date) = r.published_date {
                        if let Some(pub_year) = date.get(..4).and_then(|s| s.parse::<u32>().ok()) {
                            let tolerance = if is_temporal { 1 } else { 2 };
                            if pub_year + tolerance < qy {
                                return false;
                            }
                        }
                    }
                }
                true
            });
        }

        // Step 5c: Per-domain diversity cap and Reddit spam filtering.
        // Prevents any single source from dominating results.
        // Reddit/Wikipedia capped at 2, other domains at 3.
        {
            let mut domain_counts: HashMap<String, u32> = HashMap::new();
            ranked.retain(|r| {
                // Filter Reddit spam subreddits
                let url_lower = r.url.to_lowercase();
                if url_lower.contains("reddit.com")
                    && (url_lower.contains("/r/udemyfree")
                        || url_lower.contains("/r/freecourse")
                        || url_lower.contains("/r/coupons")
                        || url_lower.contains("/r/deals")
                        || url_lower.contains("/r/free_udemy")
                        || url_lower.contains("/r/onlinefreebies"))
                {
                    return false;
                }
                let domain = extract_domain(&r.url);
                let cap = if domain == "wikipedia.org" || domain == "arxiv.org" {
                    2
                } else if domain == "reddit.com" {
                    // Comparison benefits from community discussion threads (3 allowed).
                    // Opinion: cap at 2 — Reddit posts hurt authority scores for opinion queries.
                    if matches!(intent, rank::fusion::QueryIntent::Comparison) {
                        3
                    } else {
                        2
                    }
                } else {
                    3
                };
                let count = domain_counts.entry(domain).or_insert(0);
                *count += 1;
                *count <= cap
            });
        }

        // Step 5e: If filtering/caps left fewer than requested results, backfill from
        // the pre-filter ranked pool without extra network calls.
        // Apply the same safety filters to backfill candidates.
        if ranked.len() < max as usize {
            let mut seen: HashSet<String> = ranked.iter().map(|r| r.url.clone()).collect();
            for candidate in &ranked_before_filter {
                if ranked.len() >= max as usize {
                    break;
                }
                if candidate.score.unwrap_or(0.0) < 0.02 {
                    continue;
                }
                // Apply same safety filters as Step 5
                if is_spam_domain(&candidate.url)
                    || is_image_url(&candidate.url)
                    || is_homepage_url(&candidate.url)
                {
                    continue;
                }
                if seen.insert(candidate.url.clone()) {
                    ranked.push(candidate.clone());
                }
            }
        }

        // Step 5f: hard safety net — if filters produced zero but we had candidates,
        // return the top pre-filter items to avoid empty responses.
        if ranked.is_empty() && !ranked_before_filter.is_empty() {
            ranked = ranked_before_filter
                .iter()
                .filter(|r| !is_spam_domain(&r.url) && !is_image_url(&r.url))
                .take(max as usize)
                .cloned()
                .collect();
        }

        // Step 6: Diversify domains in top-N, then take top N.
        ranked = diversify_by_domain(ranked, max as usize);

        // Step 7: Semantic reranker (Exa-style second pass).
        // After all filtering, use embedding cosine similarity to fine-tune ordering.
        // This catches cases where term-matching ranked results incorrectly.
        #[cfg(feature = "embeddings")]
        {
            use crate::embeddings;
            if ranked.len() >= 2 {
                if let Ok(query_emb) = embeddings::embed_async(effective_query).await {
                    // Build combined text for each result
                    let texts: Vec<String> = ranked
                        .iter()
                        .map(|r| format!("{} {}", r.title, r.snippet))
                        .collect();
                    let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
                    if let Ok(result_embs) = embeddings::embed_batch_async(&text_refs).await {
                        // Compute semantic score for each result
                        let mut scored: Vec<(usize, f64)> = ranked
                            .iter()
                            .enumerate()
                            .map(|(i, r)| {
                                let semantic =
                                    embeddings::cosine_similarity(&query_emb, &result_embs[i])
                                        as f64;
                                let fusion = r.score.unwrap_or(0.5);
                                // Blend: 60% fusion score + 40% semantic rerank
                                let blended = fusion * 0.6 + semantic * 0.4;
                                (i, blended)
                            })
                            .collect();
                        scored.sort_by(|a, b| {
                            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
                        });
                        let reranked: Vec<ResultItem> = scored
                            .iter()
                            .map(|&(idx, score)| {
                                let mut r = ranked[idx].clone();
                                r.score = Some(score);
                                r
                            })
                            .collect();
                        ranked = reranked;
                        // Re-normalize scores to [0, 1]
                        if ranked.len() >= 2 {
                            let max_score = ranked
                                .iter()
                                .filter_map(|r| r.score)
                                .fold(f64::NEG_INFINITY, f64::max);
                            let min_score = ranked
                                .iter()
                                .filter_map(|r| r.score)
                                .fold(f64::INFINITY, f64::min);
                            let range = max_score - min_score;
                            if range > 1e-9 {
                                for r in &mut ranked {
                                    if let Some(s) = r.score {
                                        r.score = Some((s - min_score) / range);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

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
        "a",
        "an",
        "the",
        "and",
        "or",
        "but",
        "in",
        "on",
        "at",
        "to",
        "for",
        "of",
        "with",
        "by",
        "is",
        "are",
        "was",
        "were",
        "be",
        "been",
        "have",
        "has",
        "do",
        "does",
        "did",
        "will",
        "would",
        "could",
        "should",
        "may",
        "might",
        "what",
        "which",
        "who",
        "when",
        "where",
        "how",
        "why",
        "best",
        "top",
        "good",
        "vs",
        "vs.",
        "versus",
        "most",
        "more",
        "less",
        "all",
        "any",
        "some",
        "many",
        "can",
        "its",
        "it",
        "this",
        "that",
        "than",
        "then",
        "from",
        "into",
        "about",
        "like",
        "get",
        "use",
        "used",
        // Generic verbs/nouns that match too broadly when used as sole filter
        "new",
        "latest",
        "overview",
        "guide",
        "introduction",
        "advances",
        "build",
        "create",
        "make",
        "using",
        "step",
        "set",
        // Short words that cause false positive matches across many topics
        "go",
        "up",
        "no",
        "so",
        "if",
        "as",
        "my",
        "me",
        "we",
        "he",
        "us",
    ];
    query
        .split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|w| w.len() >= 2 && !STOPWORDS.contains(&w.as_str()))
        // Keep short numeric tokens (error codes like 429, 404, 500; versions like v8)
        // but filter out years (4-digit numbers) and plain long numbers
        .filter(|w| {
            if w.chars().all(|c| c.is_ascii_digit()) {
                w.len() == 3 // keep 3-digit codes (404, 429, 500, 754)
            } else {
                true
            }
        })
        .collect()
}

/// True when query contains non-ASCII alphabetic characters (e.g. CJK, accented scripts).
///
/// Used to avoid over-applying ASCII word-match filters to multilingual queries.
fn has_non_ascii_letters(query: &str) -> bool {
    query.chars().any(|c| c.is_alphabetic() && !c.is_ascii())
}

/// Detect if a query is practical/actionable (how-to, DIY, travel, shopping).
/// Used to reduce Wikipedia's influence on results for everyday queries.
fn is_practical_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    const PRACTICAL_PATTERNS: &[&str] = &[
        "how to ",
        "how do ",
        "how can ",
        "best way to ",
        "cheapest ",
        "where to ",
        "what to do ",
        "tips for ",
        "guide to ",
        "fix ",
        "repair ",
        "remove ",
        "clean ",
        "install ",
        "buy ",
        "price ",
        "cost ",
        "cheap ",
        "budget ",
        "recipe ",
        "workout ",
        "exercise ",
        "travel ",
        "flights ",
        "hotel ",
        "restaurant ",
        "can i ",
        "should i ",
        "is it safe ",
    ];
    PRACTICAL_PATTERNS.iter().any(|p| lower.contains(p))
}

/// Script categories for language-aware filtering.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(clippy::upper_case_acronyms)]
enum Script {
    Latin,
    CJK, // Chinese, Japanese, Korean
    Bengali,
    Devanagari,
    Arabic,
    Cyrillic,
    Other,
}

/// Detect which scripts are present in a text.
fn detect_scripts(text: &str) -> HashSet<Script> {
    let mut scripts = HashSet::new();
    for c in text.chars() {
        if !c.is_alphabetic() {
            continue;
        }
        if c.is_ascii() {
            scripts.insert(Script::Latin);
        } else {
            let cp = c as u32;
            if (0x4E00..=0x9FFF).contains(&cp)
                || (0x3040..=0x30FF).contains(&cp)
                || (0xAC00..=0xD7AF).contains(&cp)
            {
                scripts.insert(Script::CJK);
            } else if (0x0980..=0x09FF).contains(&cp) {
                scripts.insert(Script::Bengali);
            } else if (0x0900..=0x097F).contains(&cp) {
                scripts.insert(Script::Devanagari);
            } else if (0x0600..=0x06FF).contains(&cp) {
                scripts.insert(Script::Arabic);
            } else if (0x0400..=0x04FF).contains(&cp) {
                scripts.insert(Script::Cyrillic);
            } else if (0x00C0..=0x024F).contains(&cp) {
                // Latin Extended (accented characters: é, ñ, ü, etc.)
                scripts.insert(Script::Latin);
            } else {
                scripts.insert(Script::Other);
            }
        }
    }
    scripts
}

/// Returns true if the URL is a generic homepage (no meaningful path).
///
/// Filters out results like "https://foxnews.com/", "https://www.nbcnews.com/",
/// "https://apnews.com/" which backends sometimes return instead of actual articles.
/// Extract the registrable domain from a URL (e.g. "reddit.com" from "https://www.reddit.com/r/rust").
fn extract_domain(url: &str) -> String {
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    let host = without_scheme.split('/').next().unwrap_or(without_scheme);
    let host = host.split(':').next().unwrap_or(host); // strip port
                                                       // Strip "www." prefix
    let host = host.strip_prefix("www.").unwrap_or(host);
    // For subdomains like "i.redd.it" or "en.wikipedia.org", get the main domain
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() >= 2 {
        format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1])
    } else {
        host.to_string()
    }
}

fn is_homepage_url(url: &str) -> bool {
    // Strip scheme
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    // Find the first '/' after the domain
    let path = match without_scheme.find('/') {
        Some(idx) => &without_scheme[idx..],
        None => return true, // No path at all
    };
    // Strip trailing slashes and query params
    let path_only = path.split('?').next().unwrap_or(path);
    let path_trimmed = path_only.trim_matches('/');
    // Empty path = homepage
    path_trimmed.is_empty()
}

/// Check if a URL points to a known spam/junk domain.
fn is_spam_domain(url: &str) -> bool {
    const SPAM_DOMAINS: &[&str] = &[
        "smoaky.com",
        "brainly.com",
        "answers.com",
        "quizlet.com/explanations",
        "chegg.com",
        "coursehero.com",
        "bartleby.com",
        "studocu.com",
        "scribd.com/document",
        "issuu.com",
        "slideshare.net",
        "pinterest.com/pin/",
        "tiktok.com",
        "facebook.com/photo",
        "instagram.com/p/",
        // StackExchange niche sites that pollute general search
        "ell.stackexchange.com",
        "english.stackexchange.com",
        "ux.stackexchange.com",
        // Content farms
        "quora.com/unanswered",
        // Commercial sites that pollute results via keyword matching
        "signs.com",
    ];
    let url_lower = url.to_lowercase();
    SPAM_DOMAINS.iter().any(|d| url_lower.contains(d))
}

/// Check if a URL is a direct image link (no textual content to rank on).
fn is_image_url(url: &str) -> bool {
    let url_lower = url.to_lowercase();
    // Direct image URLs from Reddit, Imgur, etc.
    if url_lower.contains("i.redd.it/") || url_lower.contains("i.imgur.com/") {
        return true;
    }
    // File extension check
    let path = url_lower.split('?').next().unwrap_or(&url_lower);
    path.ends_with(".png")
        || path.ends_with(".jpg")
        || path.ends_with(".jpeg")
        || path.ends_with(".gif")
        || path.ends_with(".webp")
        || path.ends_with(".svg")
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
        // SearXNG aggregates search engines through proxies — needs time for Startpage/Yahoo
        BackendId::Searxng => Duration::from_millis(4500),
        BackendId::Reddit => Duration::from_secs(3),
        BackendId::HackerNews => Duration::from_secs(2),
        BackendId::Wikipedia => Duration::from_secs(2),
        BackendId::StackOverflow => Duration::from_secs(3),
        BackendId::Arxiv => Duration::from_secs(3),
        BackendId::Github => Duration::from_secs(3),
        // Premium API backends — fast APIs
        BackendId::Tavily | BackendId::Serper | BackendId::Exa | BackendId::Firecrawl => {
            Duration::from_secs(4)
        }
        // Scrapers are unreliable, cap aggressively
        BackendId::DuckDuckGo => Duration::from_secs(4),
        BackendId::Google | BackendId::Bing => Duration::from_secs(3),
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
    fn spam_domain_blocked() {
        assert!(is_spam_domain(
            "https://www.smoaky.com/forum/index.php?topic=123"
        ));
        assert!(is_spam_domain("https://brainly.com/question/12345"));
        assert!(!is_spam_domain("https://stackoverflow.com/questions/123"));
    }

    #[test]
    fn image_url_filtered() {
        assert!(is_image_url("https://i.redd.it/abc123.png"));
        assert!(is_image_url("https://i.imgur.com/xyz.jpg"));
        assert!(is_image_url("https://example.com/photo.jpeg"));
        assert!(!is_image_url("https://reddit.com/r/rust/comments/abc"));
        assert!(!is_image_url("https://example.com/api/image?id=123"));
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
        let mut fetchium_config = crate::config::HsxConfig::default();
        fetchium_config.search.backends = vec!["duckduckgo".to_string(), "wikipedia".to_string()];
        fetchium_config.ranking.simhash_threshold = 8;
        fetchium_config.ranking.freshness_need = 0.8;
        let cfg = OrchestratorConfig::from_fetchium_config(&fetchium_config, 20);
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
            Duration::from_secs(4)
        );
        assert_eq!(
            backend_timeout_for(&BackendId::Reddit, default),
            Duration::from_secs(3)
        );
        assert_eq!(
            backend_timeout_for(&BackendId::StackOverflow, default),
            Duration::from_secs(3)
        );
        assert_eq!(
            backend_timeout_for(&BackendId::Searxng, default),
            Duration::from_secs(6)
        );
        assert_eq!(
            backend_timeout_for(&BackendId::Wikipedia, default),
            Duration::from_secs(2)
        );
        assert_eq!(
            backend_timeout_for(&BackendId::Google, default),
            Duration::from_secs(3)
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
