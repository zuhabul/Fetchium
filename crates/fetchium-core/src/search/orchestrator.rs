//! Search orchestrator — parallel dispatch, dedup, ranking (PRD §15).
//!
//! Phase 2: All HTTP backends + HyperFusion 8-signal ranking.
//! Dispatches to all enabled backends in parallel via tokio::spawn,
//! deduplicates via URL normalization + SimHash, then applies HyperFusion ranking.

use crate::ai::ollama::OllamaClient;
use crate::ai::types::{AiConfig, ChatMessage};
use crate::error::FetchiumResult;
use crate::http::HttpClient;
use crate::query::locale::{detect_query_language, detect_query_locale};
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
use tokio::task::JoinSet;
use tokio::time::timeout;
use tracing::{info, warn};

#[derive(Debug, Clone, Default)]
struct BengaliPlannerPlan {
    retry_query: Option<String>,
    source_pack_queries: Vec<String>,
}

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
    /// Premium API keys for optional paid backends.
    pub tavily_api_keys: Vec<String>,
    pub serper_api_keys: Vec<String>,
    pub exa_api_keys: Vec<String>,
    pub firecrawl_api_keys: Vec<String>,
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

/// All recommended default backends (free / self-hosted).
///
/// SearXNG self-hosted (localhost:4040) is first — it aggregates Google, Bing,
/// Brave, DuckDuckGo, Wikipedia, SO, GitHub, ArXiv, Reddit in a single request.
const ALL_DEFAULT_BACKENDS: &[BackendId] = &[
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
            tavily_api_keys: Vec::new(),
            serper_api_keys: Vec::new(),
            exa_api_keys: Vec::new(),
            firecrawl_api_keys: Vec::new(),
        }
    }
}

impl OrchestratorConfig {
    /// Create config from FetchiumConfig settings.
    ///
    /// Always ensures reliable API-based backends (Wikipedia, HN, Reddit, SO,
    /// Arxiv) are included even when the user's config only lists scrapers.
    /// This prevents total search failure when scrapers are CAPTCHA-blocked.
    pub fn from_fetchium_config(
        fetchium_config: &crate::config::FetchiumConfig,
        max_results: u32,
    ) -> Self {
        let internal_headroom = max_results.max(10);
        let append_reliable_fallbacks = std::env::var(
            "FETCHIUM_SEARCH_APPEND_RELIABLE_FALLBACKS",
        )
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            !matches!(normalized.as_str(), "0" | "false" | "no" | "off")
        })
        .unwrap_or(true);
        let mut enabled_backends = fetchium_config
            .search
            .backends
            .iter()
            .filter_map(|s| parse_backend_id(s))
            .collect::<Vec<_>>();

        if enabled_backends.is_empty() {
            enabled_backends = ALL_DEFAULT_BACKENDS.to_vec();
        } else if append_reliable_fallbacks {
            // Always ensure reliable API backends are present as fallbacks
            for backend in RELIABLE_API_BACKENDS {
                if !enabled_backends.contains(backend) {
                    enabled_backends.push(backend.clone());
                }
            }
        }

        Self {
            max_results_per_backend: internal_headroom + 5,
            max_total_results: max_results,
            backend_timeout: Duration::from_secs(fetchium_config.search.timeout_secs),
            enabled_backends,
            simhash_threshold: fetchium_config.ranking.simhash_threshold,
            freshness_need: fetchium_config.ranking.freshness_need,
            use_hyperfusion: true,
            tavily_api_keys: merge_api_keys(
                &fetchium_config.search.tavily_api_key,
                &fetchium_config.search.tavily_api_keys,
                "TAVILY_API_KEY",
            ),
            serper_api_keys: merge_api_keys(
                &fetchium_config.search.serper_api_key,
                &fetchium_config.search.serper_api_keys,
                "SERPER_API_KEY",
            ),
            exa_api_keys: merge_api_keys(
                &fetchium_config.search.exa_api_key,
                &fetchium_config.search.exa_api_keys,
                "EXA_API_KEY",
            ),
            firecrawl_api_keys: merge_api_keys(
                &fetchium_config.search.firecrawl_api_key,
                &fetchium_config.search.firecrawl_api_keys,
                "FIRECRAWL_API_KEY",
            ),
        }
    }
}

fn merge_api_keys(
    config_primary: &Option<String>,
    config_keys: &[String],
    env_var: &str,
) -> Vec<String> {
    let mut keys = Vec::new();
    if let Some(key) = config_primary.as_ref().filter(|key| !key.trim().is_empty()) {
        keys.push(key.clone());
    }
    for key in config_keys.iter().filter(|key| !key.trim().is_empty()) {
        if !keys.contains(key) {
            keys.push(key.clone());
        }
    }
    if let Ok(env_key) = std::env::var(env_var) {
        if !env_key.trim().is_empty() && !keys.contains(&env_key) {
            keys.push(env_key);
        }
    }
    keys
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
                    if config.tavily_api_keys.is_empty() {
                        warn!("Tavily backend enabled but no API key configured — skipping");
                    } else {
                        backends.push(Arc::new(TavilyBackend::new(
                            http_client.clone(),
                            config.tavily_api_keys.clone(),
                        )));
                    }
                }
                BackendId::Serper => {
                    if config.serper_api_keys.is_empty() {
                        warn!("Serper backend enabled but no API key configured — skipping");
                    } else {
                        backends.push(Arc::new(SerperBackend::new(
                            http_client.clone(),
                            config.serper_api_keys.clone(),
                        )));
                    }
                }
                BackendId::Exa => {
                    if config.exa_api_keys.is_empty() {
                        warn!("Exa backend enabled but no API key configured — skipping");
                    } else {
                        backends.push(Arc::new(ExaBackend::new(
                            http_client.clone(),
                            config.exa_api_keys.clone(),
                        )));
                    }
                }
                BackendId::Firecrawl => {
                    if config.firecrawl_api_keys.is_empty() {
                        warn!("Firecrawl backend enabled but no API key configured — skipping");
                    } else {
                        backends.push(Arc::new(FirecrawlBackend::new(
                            http_client.clone(),
                            config.firecrawl_api_keys.clone(),
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
                    if config.tavily_api_keys.is_empty() {
                        warn!("Tavily backend enabled but no API key configured — skipping");
                    } else {
                        backends.push(Arc::new(TavilyBackend::new(
                            http_client.clone(),
                            config.tavily_api_keys.clone(),
                        )));
                    }
                }
                BackendId::Serper => {
                    if config.serper_api_keys.is_empty() {
                        warn!("Serper backend enabled but no API key configured — skipping");
                    } else {
                        backends.push(Arc::new(SerperBackend::new(
                            http_client.clone(),
                            config.serper_api_keys.clone(),
                        )));
                    }
                }
                BackendId::Exa => {
                    if config.exa_api_keys.is_empty() {
                        warn!("Exa backend enabled but no API key configured — skipping");
                    } else {
                        backends.push(Arc::new(ExaBackend::new(
                            http_client.clone(),
                            config.exa_api_keys.clone(),
                        )));
                    }
                }
                BackendId::Firecrawl => {
                    if config.firecrawl_api_keys.is_empty() {
                        warn!("Firecrawl backend enabled but no API key configured — skipping");
                    } else {
                        backends.push(Arc::new(FirecrawlBackend::new(
                            http_client.clone(),
                            config.firecrawl_api_keys.clone(),
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
    ) -> FetchiumResult<Vec<ResultItem>> {
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
    async fn execute_search(
        &self,
        query: &str,
        requested_max: u32,
    ) -> FetchiumResult<Vec<ResultItem>> {
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
        let language = detect_query_language(effective_query).map(|s| s.to_string());
        let search_ctx = SearchContext {
            intent,
            time_range,
            locale,
            language,
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
        let health_sensitive_query = is_health_sensitive_query(effective_query);
        let strict_health_query = is_strict_health_query(effective_query);
        let health_explainer_query = is_health_explainer_query(effective_query);
        let language_learning_query = is_language_learning_query(effective_query);
        let query_has_non_ascii = has_non_ascii_letters(query);
        let original_query_language =
            detect_query_language(query).or_else(|| detect_query_language(effective_query));
        let effective_query_lower = effective_query.to_lowercase();
        let comparison_like_query = effective_query_lower.contains(" vs ")
            || effective_query_lower.contains("compare")
            || effective_query_lower.contains("comparison")
            || effective_query_lower.contains("versus");
        let protocol_numeric_query = effective_query_lower.contains("status code")
            || effective_query_lower.contains("http ")
            || effective_query_lower.contains("rfc ")
            || effective_query_lower.contains("error code");
        let long_query = effective_query.split_whitespace().count() >= 8;
        let deeper_recall_query =
            health_sensitive_query || comparison_like_query || protocol_numeric_query || long_query;
        let effective_query_scripts = detect_scripts(effective_query);
        let effective_query_has_non_latin_script =
            effective_query_scripts.iter().any(|s| *s != Script::Latin);
        let retrieval_max = if deeper_recall_query {
            requested_max.max(10)
        } else {
            requested_max
        };
        let selected_set: std::collections::HashSet<String> = selection
            .backends
            .iter()
            .filter(|b| !health_sensitive_query || health_sensitive_backend_allowed(b))
            .map(|b| format!("{b:?}"))
            .collect();
        let mut selected_set = selected_set;
        let searxng_available = available_ids.contains(&BackendId::Searxng);
        let bengali_technical_explainer_query =
            is_multilingual_technical_explainer_query(query, original_query_language)
                && matches!(original_query_language, Some("bd" | "bn"));
        let bengali_planner_plan = if bengali_technical_explainer_query {
            plan_bengali_technical_query(query)
                .await
                .unwrap_or_default()
        } else {
            BengaliPlannerPlan::default()
        };
        if (health_sensitive_query || query_has_non_ascii) && searxng_available {
            selected_set.insert(format!("{:?}", BackendId::Searxng));
        }
        if searxng_available && effective_query_has_non_latin_script {
            // Non-Latin-script queries fare better through SearXNG's federated
            // engines than DDG's HTML scraper on this host.
            selected_set.remove(&format!("{:?}", BackendId::DuckDuckGo));
        }
        if bengali_technical_explainer_query {
            selected_set.remove(&format!("{:?}", BackendId::Searxng));
            selected_set.remove(&format!("{:?}", BackendId::Wikipedia));
            selected_set.remove(&format!("{:?}", BackendId::HackerNews));
            selected_set.remove(&format!("{:?}", BackendId::Reddit));
            selected_set.remove(&format!("{:?}", BackendId::Arxiv));
            selected_set.remove(&format!("{:?}", BackendId::Github));
            selected_set.remove(&format!("{:?}", BackendId::StackOverflow));
            selected_set.remove(&format!("{:?}", BackendId::Bing));
            selected_set.insert(format!("{:?}", BackendId::DuckDuckGo));
            if available_ids.contains(&BackendId::Brave) {
                selected_set.insert(format!("{:?}", BackendId::Brave));
            }
            if available_ids.contains(&BackendId::Google) {
                selected_set.insert(format!("{:?}", BackendId::Google));
            }
        }
        if health_explainer_query {
            selected_set.remove(&format!("{:?}", BackendId::Google));
            selected_set.remove(&format!("{:?}", BackendId::Bing));
        }
        if language_learning_query
            || is_license_query(effective_query)
            || is_philosophy_query(effective_query)
        {
            selected_set.remove(&format!("{:?}", BackendId::Bing));
        }

        info!(
            "Orchestrator: {:?} intent={:?}, {} of {} backend(s) selected, max={}",
            effective_query,
            intent,
            selected_set.len(),
            self.backends.len(),
            requested_max
        );

        self.metrics.inc_searches();
        let _search_timer = self.metrics.start_operation("search_total");

        // Step 1: Parallel dispatch with circuit breaker + bulkhead protection
        let mut handles = JoinSet::new();
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
            let q = effective_query.to_string();
            let cb = self.circuit_breaker.clone();
            let bh = self.bulkhead.clone();
            let metrics = self.metrics.clone();
            let ctx = search_ctx.clone();

            handles.spawn(async move {
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
            });
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
        let _target_results = if deeper_recall_query {
            ((retrieval_max as usize) * 3).max(15)
        } else {
            (requested_max as usize) * 3 / 2 // Relaxed from 2x for faster response
        };

        // Dynamic early-return deadline: 1.5s for fast response, 6.0s hard timeout
        let early_deadline = tokio::time::Instant::now()
            + if deeper_recall_query {
                Duration::from_millis(2800)
            } else {
                Duration::from_millis(1500)
            };
        let hard_deadline = tokio::time::Instant::now() + timeout_dur;

        let mut quality_backends_responded: u32 = 0;
        let mut premium_backends_responded: u32 = 0;

        enum DispatchEvent<T> {
            Join(Option<Result<T, tokio::task::JoinError>>),
            ExitEarly,
        }

        let mut exit_early = false;
        while !handles.is_empty() {
            let event = tokio::select! {
                r = handles.join_next() => DispatchEvent::Join(r),
                _ = tokio::time::sleep_until(early_deadline) => {
                    if all.len() >= requested_max as usize
                        && quality_backends_responded >= if deeper_recall_query { 3 } else { 2 }
                    {
                        info!(
                            "Orchestrator: early-return (fast) with {} results, {} backends still pending",
                            all.len(),
                            handles.len()
                        );
                        DispatchEvent::ExitEarly
                    } else {
                        tokio::select! {
                            r = handles.join_next() => DispatchEvent::Join(r),
                            _ = tokio::time::sleep_until(hard_deadline) => {
                                info!("Orchestrator: hard timeout hit with {} results", all.len());
                                DispatchEvent::ExitEarly
                            }
                        }
                    }
                }
            };

            match event {
                DispatchEvent::Join(Some(Ok((backend_id, results)))) => {
                    let quality = (results.len() as f64 / 10.0).min(1.0);
                    self.backend_selector
                        .report_outcome(&backend_id, results.len(), quality);

                    if !results.is_empty() {
                        if matches!(
                            backend_id,
                            BackendId::Tavily
                                | BackendId::Serper
                                | BackendId::Exa
                                | BackendId::Firecrawl
                        ) {
                            premium_backends_responded += 1;
                            quality_backends_responded += 1;
                        } else if !matches!(backend_id, BackendId::Wikipedia | BackendId::Arxiv) {
                            quality_backends_responded += 1;
                        }
                    }

                    all.extend(results);

                    // Super-early return: if we have 1+ premium backend and enough results, return immediately.
                    // This dramatically improves tail latency when using high-quality paid APIs.
                    if !deeper_recall_query
                        && all.len() >= (requested_max as usize)
                        && premium_backends_responded >= 1
                    {
                        info!(
                            "Orchestrator: super-early return (premium) with {} results",
                            all.len()
                        );
                        exit_early = true;
                        break;
                    }

                    // Standard early return: enough results from enough quality sources.
                    // Relaxed from 2x headroom to 1.5x for faster response.
                    if all.len() >= ((requested_max as usize) * 3 / 2)
                        && quality_backends_responded >= if deeper_recall_query { 4 } else { 3 }
                    {
                        info!(
                            "Orchestrator: early-return (quality) with {} results, {} backends still pending",
                            all.len(),
                            handles.len()
                        );
                        exit_early = true;
                        break;
                    }
                }
                DispatchEvent::Join(Some(Err(e))) => {
                    self.metrics.record_error("backend_panic");
                    warn!("Backend task panicked: {e}");
                }
                DispatchEvent::Join(None) => break,
                DispatchEvent::ExitEarly => {
                    exit_early = true;
                    break;
                }
            }
        }

        if exit_early && !handles.is_empty() {
            handles.abort_all();
            while handles.join_next().await.is_some() {}
        }

        let rescue_threshold = ((retrieval_max as usize) / 2).max(3);
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
                if bengali_technical_explainer_query
                    && !matches!(
                        backend.id(),
                        BackendId::DuckDuckGo | BackendId::Brave | BackendId::Google
                    )
                {
                    continue;
                }
                if health_sensitive_query && !health_sensitive_backend_allowed(&backend.id()) {
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
            if all.is_empty() && !bengali_technical_explainer_query {
                info!(
                    "Orchestrator: no results from any backend for {:?}",
                    effective_query
                );
                return Ok(Vec::new());
            }
        }

        if health_sensitive_query
            && !all
                .iter()
                .any(|r| strong_health_candidate(r, effective_query))
        {
            info!(
                "Orchestrator: no strong health candidates for {:?} — retrying DuckDuckGo with extended timeout",
                effective_query
            );
            if let Some(backend) = self
                .backends
                .iter()
                .find(|backend| backend.id() == BackendId::DuckDuckGo)
                .cloned()
            {
                if let Ok(Ok(results)) = timeout(
                    Duration::from_secs(7),
                    backend.search_with_context(effective_query, per_backend, &search_ctx),
                )
                .await
                {
                    if !results.is_empty() {
                        info!(
                            "Orchestrator: DuckDuckGo health retry produced {} results",
                            results.len()
                        );
                        all.extend(results);
                    }
                }
            }
        }

        if intent == rank::fusion::QueryIntent::Comparison
            && !all.iter().any(|r| {
                let haystack = format!("{} {}", r.title.to_lowercase(), r.snippet.to_lowercase());
                comparison_entity_match_count(effective_query, &haystack)
                    >= comparison_entity_threshold(effective_query)
            })
        {
            info!(
                "Orchestrator: no strong comparison candidates for {:?} — retrying DuckDuckGo with extended timeout",
                effective_query
            );
            if let Some(backend) = self
                .backends
                .iter()
                .find(|backend| backend.id() == BackendId::DuckDuckGo)
                .cloned()
            {
                if let Ok(Ok(results)) = timeout(
                    Duration::from_secs(7),
                    backend.search_with_context(effective_query, per_backend, &search_ctx),
                )
                .await
                {
                    if !results.is_empty() {
                        info!(
                            "Orchestrator: DuckDuckGo comparison retry produced {} results",
                            results.len()
                        );
                        all.extend(results);
                    }
                }
            }
        }

        if matches!(
            intent,
            rank::fusion::QueryIntent::HowTo | rank::fusion::QueryIntent::Code
        ) && !all.iter().any(|r| strong_technical_candidate(r, intent))
        {
            let retry_query = docs_focused_query(effective_query);
            info!(
                "Orchestrator: no strong technical candidates for {:?} — retrying DuckDuckGo with {:?}",
                effective_query,
                retry_query
            );
            if let Some(backend) = self
                .backends
                .iter()
                .find(|backend| backend.id() == BackendId::DuckDuckGo)
                .cloned()
            {
                if let Ok(Ok(results)) = timeout(
                    Duration::from_secs(7),
                    backend.search_with_context(&retry_query, per_backend, &search_ctx),
                )
                .await
                {
                    if !results.is_empty() {
                        info!(
                            "Orchestrator: DuckDuckGo technical retry produced {} results",
                            results.len()
                        );
                        all.extend(results);
                    }
                }
            }
        }

        if bengali_technical_explainer_query
            && !all.iter().any(|r| {
                let haystack = format!("{} {}", r.title.to_lowercase(), r.snippet.to_lowercase());
                multilingual_technical_explainer_match_count(query, &haystack, &r.url) >= 2
            })
        {
            let retry_query = bengali_planner_plan
                .retry_query
                .clone()
                .unwrap_or_else(|| bengali_technical_retry_query(query, effective_query));
            info!(
                "Orchestrator: no strong Bengali technical explainer candidates for {:?} — retrying web backends with {:?}",
                effective_query,
                retry_query
            );
            for (backend_id, retry_timeout) in [
                (BackendId::Google, Duration::from_secs(10)),
                (BackendId::Bing, Duration::from_secs(10)),
                (BackendId::Brave, Duration::from_secs(8)),
                (BackendId::DuckDuckGo, Duration::from_secs(8)),
            ] {
                if let Some(backend) = self.backends.iter().find(|b| b.id() == backend_id).cloned()
                {
                    if let Ok(Ok(results)) = timeout(
                        retry_timeout,
                        backend.search_with_context(&retry_query, per_backend, &search_ctx),
                    )
                    .await
                    {
                        if !results.is_empty() {
                            all.extend(results.into_iter().filter(|r| {
                                let haystack = format!(
                                    "{} {}",
                                    r.title.to_lowercase(),
                                    r.snippet.to_lowercase()
                                );
                                multilingual_technical_trusted_source(&r.url)
                                    || multilingual_technical_explainer_match_count(
                                        effective_query,
                                        &haystack,
                                        &r.url,
                                    ) >= 2
                            }));
                        }
                    }
                }
            }
        }

        if bengali_technical_explainer_query
            && !all.iter().any(|r| {
                let haystack = format!("{} {}", r.title.to_lowercase(), r.snippet.to_lowercase());
                multilingual_technical_trusted_source(&r.url)
                    || multilingual_technical_explainer_match_count(query, &haystack, &r.url) >= 2
            })
        {
            let source_pack_queries = if bengali_planner_plan.source_pack_queries.is_empty() {
                bengali_source_pack_queries(query, effective_query)
            } else {
                bengali_planner_plan.source_pack_queries.clone()
            };
            if !source_pack_queries.is_empty() {
                info!(
                    "Orchestrator: Bengali source-pack fallback for {:?} with {} targeted queries",
                    effective_query,
                    source_pack_queries.len()
                );
            }
            for source_query in source_pack_queries {
                for (backend_id, retry_timeout) in [
                    (BackendId::Google, Duration::from_secs(10)),
                    (BackendId::Bing, Duration::from_secs(10)),
                    (BackendId::Brave, Duration::from_secs(8)),
                ] {
                    if let Some(backend) =
                        self.backends.iter().find(|b| b.id() == backend_id).cloned()
                    {
                        if let Ok(Ok(results)) = timeout(
                            retry_timeout,
                            backend.search_with_context(
                                &source_query,
                                per_backend.min(6),
                                &search_ctx,
                            ),
                        )
                        .await
                        {
                            if !results.is_empty() {
                                all.extend(results.into_iter().filter(|r| {
                                    multilingual_technical_trusted_source(&r.url)
                                        || bengali_ai_direct_source(&r.url)
                                }));
                            }
                        }
                    }
                }
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
            let boost_query_language =
                detect_query_language(effective_query).or_else(|| detect_query_language(query));
            let boost_words = extract_content_words(effective_query, boost_query_language);
            let query_phrases = extract_query_phrases(query, detect_query_language(query));
            let short_concept_query = is_short_concept_query(
                boost_words.len(),
                intent,
                strict_health_query,
                effective_query,
            );
            if !boost_words.is_empty() {
                for r in &mut ranked {
                    let haystack =
                        format!("{} {}", r.title.to_lowercase(), r.snippet.to_lowercase());
                    let matches = count_term_matches(&haystack, &boost_words);
                    let coverage = matches as f64 / boost_words.len() as f64;
                    // Scale: 100% coverage → 1.0x (no change), 0% → 0.3x penalty
                    // Steeper penalty catches off-topic results from high-authority domains
                    let mut boost = 0.3 + (coverage * 0.7);
                    let phrase_matches = count_phrase_matches(&haystack, &query_phrases);
                    if phrase_matches > 0 {
                        boost *= 1.0 + (phrase_matches.min(2) as f64 * 0.12);
                    }
                    if health_sensitive_query {
                        boost *= health_authority_multiplier(&r.url);
                    }
                    if strict_health_query {
                        boost *= medical_attribute_multiplier(effective_query, &haystack);
                    } else if health_explainer_query {
                        boost *= health_explainer_multiplier(effective_query, &haystack, &r.url);
                    } else if language_learning_query {
                        boost *= language_learning_multiplier(effective_query, &haystack, &r.url);
                    } else if is_license_query(effective_query) {
                        boost *= license_query_multiplier(effective_query, &haystack, &r.url);
                    } else if is_philosophy_query(effective_query) {
                        boost *= philosophy_query_multiplier(effective_query, &haystack, &r.url);
                    } else if is_multilingual_technical_explainer_query(
                        effective_query,
                        boost_query_language,
                    ) {
                        boost *= multilingual_technical_explainer_multiplier(
                            effective_query,
                            &haystack,
                            &r.url,
                        );
                    } else if intent == rank::fusion::QueryIntent::Comparison {
                        boost *= comparison_multiplier(effective_query, &haystack, &r.url);
                    } else if intent == rank::fusion::QueryIntent::HowTo
                        || intent == rank::fusion::QueryIntent::Code
                    {
                        boost *= technical_howto_multiplier(effective_query, &haystack, &r.url);
                    } else if health_sensitive_query {
                        boost *= broad_health_multiplier(effective_query, &haystack, &r.url);
                    }
                    if short_concept_query {
                        boost *= short_concept_title_match_boost(effective_query, &r.title, &r.url);
                    }
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
        let query_language =
            detect_query_language(effective_query).or_else(|| detect_query_language(query));
        let query_content_words = extract_content_words(effective_query, query_language);
        let specificity_words = extract_specificity_words(&query_content_words);
        let query_phrases = extract_query_phrases(query, detect_query_language(query));
        let medical_query_relaxed = strict_health_query;
        let ranked_before_filter = ranked.clone();
        let pre_filter_count = ranked.len();
        let multilingual_query = has_non_ascii_letters(query) || query_language.is_some();
        let multilingual_foreign_query = multilingual_query;

        // Step 4b: Language-aware filtering (Exa-style).
        // For multilingual queries, boost results in the same script/language
        // and demote results that are clearly in a different language.
        if multilingual_query {
            let query_has_non_latin = effective_query_has_non_latin_script;
            ranked.retain(|r| {
                let result_text = format!("{} {}", r.title, r.snippet);
                let result_scripts = detect_scripts(&result_text);
                // Keep result if it shares at least one non-Latin script with the query,
                // OR if it's from a known authoritative domain (Wikipedia handles multilingual well)
                let shares_script = effective_query_scripts
                    .iter()
                    .any(|s| result_scripts.contains(s));
                let is_authoritative = r.url.contains("wikipedia.org");
                shares_script
                    || is_authoritative
                    || (!query_has_non_latin && result_scripts.contains(&Script::Latin))
            });
            // If language filtering removed too many, fall back to unfiltered
            if ranked.len() < 3 {
                ranked = ranked_before_filter.clone();
            }
        }
        // Proportional term matching: for long queries require ~30% of content words (down from 40%).
        // This avoids over-filtering high-quality results that don't repeat the entire query.
        let strict_min_term_matches = if language_learning_query
            || is_license_query(effective_query)
            || is_philosophy_query(effective_query)
        {
            1
        } else if multilingual_query {
            // For multilingual, still require 1 match if we have content words
            if query_content_words.len() >= 2 {
                1
            } else {
                0
            }
        } else if medical_query_relaxed {
            2
        } else if query_content_words.len() >= 7 {
            (query_content_words.len() * 3 / 10).max(2) // ~30% for very long queries
        } else if query_content_words.len() >= 4 {
            2
        } else {
            1
        };
        // Relaxed threshold: for comparison/code queries with many entities,
        // still require more than 1 term to avoid single-entity Wikipedia pages.
        let relaxed_min_term_matches = if language_learning_query
            || is_license_query(effective_query)
            || is_philosophy_query(effective_query)
        {
            1
        } else if query_content_words.len() >= 4 {
            if medical_query_relaxed {
                1
            } else {
                2
            } // For multi-entity queries, require 2 even in relaxed mode
        } else {
            strict_min_term_matches.min(1)
        };
        ranked.retain(|r| {
            let haystack = format!("{} {}", r.title.to_lowercase(), r.snippet.to_lowercase());
            if is_multilingual_technical_explainer_query(query, query_language)
                && multilingual_technical_explainer_blocked_domain(&r.url)
            {
                return false;
            }
            let comparison_matches = comparison_entity_match_count(effective_query, &haystack);
            let strong_technical_coverage =
                strong_technical_coverage(effective_query, &haystack, &r.url);
            let language_learning_matches =
                language_learning_match_count(effective_query, &haystack, &r.url);
            let medical_attribute_matches =
                medical_attribute_match_count(effective_query, &haystack);
            let license_candidate = is_license_query(effective_query)
                && license_query_match_count(effective_query, &haystack, &r.url) >= 2;
            let philosophy_candidate = is_philosophy_query(effective_query)
                && philosophy_query_match_count(effective_query, &haystack, &r.url) >= 2;
            let multilingual_technical_matches =
                multilingual_technical_explainer_match_count(effective_query, &haystack, &r.url);
            let trusted_bengali_technical_candidate = matches!(query_language, Some("bn" | "bd"))
                && is_multilingual_technical_explainer_query(effective_query, query_language)
                && multilingual_technical_trusted_source(&r.url)
                && multilingual_technical_matches >= 1;
            let score_floor = if (intent == rank::fusion::QueryIntent::Comparison
                && comparison_matches >= comparison_entity_threshold(effective_query))
                || (language_learning_query && language_learning_matches >= 2)
            {
                0.02
            } else if (strict_health_query && medical_attribute_matches > 0)
                || license_candidate
                || philosophy_candidate
                || trusted_bengali_technical_candidate
                || (is_multilingual_technical_explainer_query(effective_query, query_language)
                    && multilingual_technical_matches >= 2)
                || (matches!(
                    intent,
                    rank::fusion::QueryIntent::HowTo | rank::fusion::QueryIntent::Code
                ) && strong_technical_coverage)
            {
                0.03
            } else {
                0.05 // Lower default floor from 0.10 for better recall
            };

            // (a) score threshold
            if r.score.unwrap_or(0.0) < score_floor {
                return false;
            }
            if strict_health_query && r.url.contains("wikipedia.org") {
                return false;
            }
            if community_fallback_blocked(
                intent,
                health_sensitive_query,
                language_learning_query,
                multilingual_foreign_query,
                r,
            ) {
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
            if strict_health_query && generic_disease_overview(effective_query, &haystack, &r.url) {
                tracing::debug!(
                    "Filtered generic disease overview for strict medical comparison: {:?}",
                    r.title
                );
                return false;
            }
            // (b) title-term check — only apply when we have meaningful query words
            if strict_min_term_matches > 0 && !query_content_words.is_empty() {
                let matches = count_term_matches(&haystack, &query_content_words);
                let specificity_matches = count_term_matches(&haystack, &specificity_words);
                let phrase_matches = count_phrase_matches(&haystack, &query_phrases);
                let health_explainer_matches =
                    health_explainer_match_count(effective_query, &haystack, &r.url);
                let broad_health_matches =
                    broad_health_match_count(effective_query, &haystack, &r.url);
                let multilingual_technical_matches = multilingual_technical_explainer_match_count(
                    effective_query,
                    &haystack,
                    &r.url,
                );
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
                if !(medical_query_relaxed
                    || specificity_words.is_empty()
                    || specificity_matches != 0
                    || phrase_matches != 0
                    || (language_learning_query && language_learning_matches >= 2)
                    || trusted_bengali_technical_candidate
                    || (is_license_query(effective_query)
                        && license_query_match_count(effective_query, &haystack, &r.url) >= 2)
                    || (is_philosophy_query(effective_query)
                        && philosophy_query_match_count(effective_query, &haystack, &r.url) >= 2))
                {
                    tracing::debug!(
                        "Filtered low-specificity match: {:?} (specificity_words={:?})",
                        r.title,
                        specificity_words
                    );
                    return false;
                }
                if matches!(
                    intent,
                    rank::fusion::QueryIntent::HowTo | rank::fusion::QueryIntent::Code
                ) && specificity_words.len() >= 3
                    && specificity_matches < 2
                    && phrase_matches == 0
                    && !strong_technical_coverage
                {
                    tracing::debug!(
                        "Filtered low-specificity technical howto/code match: {:?}",
                        r.title
                    );
                    return false;
                }
                if strict_health_query && medical_attribute_matches == 0 {
                    tracing::debug!("Filtered strict-health low-attribute match: {:?}", r.title);
                    return false;
                }
                if health_explainer_query && health_explainer_matches < 2 {
                    tracing::debug!(
                        "Filtered health-explainer low-coverage match: {:?}",
                        r.title
                    );
                    return false;
                }
                if language_learning_query && language_learning_matches < 2 {
                    tracing::debug!(
                        "Filtered language-learning low-coverage match: {:?}",
                        r.title
                    );
                    return false;
                }
                if is_multilingual_technical_explainer_query(effective_query, query_language)
                    && multilingual_technical_matches < 2
                    && !trusted_bengali_technical_candidate
                {
                    tracing::debug!(
                        "Filtered multilingual technical explainer low-coverage match: {:?}",
                        r.title
                    );
                    return false;
                }
                if intent == rank::fusion::QueryIntent::Comparison
                    && comparison_matches < comparison_entity_threshold(effective_query)
                {
                    tracing::debug!(
                        "Filtered comparison low-entity-coverage match: {:?}",
                        r.title
                    );
                    return false;
                }
                if health_sensitive_query
                    && !strict_health_query
                    && !health_explainer_query
                    && broad_health_matches < 2
                {
                    tracing::debug!("Filtered broad-health low-coverage match: {:?}", r.title);
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
        let preserve_precision = !specificity_words.is_empty()
            && !medical_query_relaxed
            && intent != rank::fusion::QueryIntent::Comparison
            && !language_learning_query
            && !is_multilingual_technical_explainer_query(effective_query, query_language);
        let has_strong_health_candidates = health_sensitive_query
            && ranked
                .iter()
                .any(|r| strong_health_candidate(r, effective_query));
        let has_strong_technical_candidates =
            matches!(
                intent,
                rank::fusion::QueryIntent::HowTo | rank::fusion::QueryIntent::Code
            ) && ranked.iter().any(|r| strong_technical_candidate(r, intent));

        if has_strong_health_candidates {
            ranked.retain(|r| strong_health_candidate(r, effective_query));
        }
        if has_strong_technical_candidates {
            ranked.retain(|r| strong_technical_candidate(r, intent));
        }

        if ranked.len() < 5 && pre_filter_count >= 5 && !preserve_precision {
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
                if community_fallback_blocked(
                    intent,
                    health_sensitive_query,
                    language_learning_query,
                    multilingual_foreign_query,
                    r,
                ) {
                    return false;
                }
                if has_strong_health_candidates && !strong_health_candidate(r, effective_query) {
                    return false;
                }
                if has_strong_technical_candidates && !strong_technical_candidate(r, intent) {
                    return false;
                }
                let haystack = format!("{} {}", r.title.to_lowercase(), r.snippet.to_lowercase());
                if strict_health_query
                    && generic_disease_overview(effective_query, &haystack, &r.url)
                {
                    return false;
                }
                if relaxed_min_term_matches == 0 || query_content_words.is_empty() {
                    return true;
                }
                let matches = count_term_matches(&haystack, &query_content_words);
                let specificity_matches = count_term_matches(&haystack, &specificity_words);
                let phrase_matches = count_phrase_matches(&haystack, &query_phrases);
                let medical_attribute_matches =
                    medical_attribute_match_count(effective_query, &haystack);
                let health_explainer_matches =
                    health_explainer_match_count(effective_query, &haystack, &r.url);
                let language_learning_matches =
                    language_learning_match_count(effective_query, &haystack, &r.url);
                let broad_health_matches =
                    broad_health_match_count(effective_query, &haystack, &r.url);
                let trusted_license_keepalive = is_license_query(effective_query)
                    && trusted_license_candidate(effective_query, &haystack, &r.url);
                matches >= relaxed_min_term_matches
                    && (!strict_health_query || medical_attribute_matches > 0)
                    && (!health_explainer_query || health_explainer_matches >= 2)
                    && (!language_learning_query || language_learning_matches >= 2)
                    && (!health_sensitive_query
                        || strict_health_query
                        || health_explainer_query
                        || broad_health_matches >= 2)
                    && (trusted_license_keepalive
                        || medical_query_relaxed
                        || specificity_words.is_empty()
                        || specificity_matches > 0
                        || phrase_matches > 0)
            });
        }
        if ranked.len() < 3 && pre_filter_count >= 3 && !preserve_precision {
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
                        let matches_any = count_term_matches(&haystack, &query_content_words) > 0;
                        let specificity_matches = count_term_matches(&haystack, &specificity_words);
                        let phrase_matches = count_phrase_matches(&haystack, &query_phrases);
                        let medical_attribute_matches =
                            medical_attribute_match_count(effective_query, &haystack);
                        let health_explainer_matches =
                            health_explainer_match_count(effective_query, &haystack, &r.url);
                        let language_learning_matches =
                            language_learning_match_count(effective_query, &haystack, &r.url);
                        let broad_health_matches =
                            broad_health_match_count(effective_query, &haystack, &r.url);
                        matches_any
                            && (!strict_health_query || medical_attribute_matches > 0)
                            && (!health_explainer_query || health_explainer_matches >= 2)
                            && (!language_learning_query || language_learning_matches >= 2)
                            && (!health_sensitive_query
                                || strict_health_query
                                || health_explainer_query
                                || broad_health_matches >= 2)
                            && (medical_query_relaxed
                                || specificity_words.is_empty()
                                || specificity_matches > 0
                                || phrase_matches > 0)
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
            let short_concept_query = is_short_concept_query(
                query_content_words.len(),
                intent,
                strict_health_query,
                effective_query,
            );
            // HowTo/Code/Casual: Wikipedia articles rarely useful; Comparison allows 1
            let wiki_cap: u32 = if strict_health_query
                || is_multilingual_technical_explainer_query(effective_query, query_language)
                || is_temporal
                || is_practical
                || is_casual
                || is_howto
                || is_code
            {
                if short_concept_query {
                    1
                } else {
                    0
                }
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
                } else if domain == "reddit.com" || domain == "ycombinator.com" {
                    // Comparison benefits from community discussion threads (3 allowed).
                    // Opinion: cap at 2 — Reddit posts hurt authority scores for opinion queries.
                    if matches!(
                        intent,
                        rank::fusion::QueryIntent::HowTo | rank::fusion::QueryIntent::Code
                    ) || health_sensitive_query
                    {
                        0
                    } else if matches!(intent, rank::fusion::QueryIntent::Comparison) {
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
        if ranked.len() < requested_max as usize && !preserve_precision {
            let mut seen: HashSet<String> = ranked.iter().map(|r| r.url.clone()).collect();
            for candidate in &ranked_before_filter {
                if ranked.len() >= requested_max as usize {
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
                if community_fallback_blocked(
                    intent,
                    health_sensitive_query,
                    language_learning_query,
                    multilingual_foreign_query,
                    candidate,
                ) {
                    continue;
                }
                if has_strong_health_candidates
                    && !strong_health_candidate(candidate, effective_query)
                {
                    continue;
                }
                if has_strong_technical_candidates && !strong_technical_candidate(candidate, intent)
                {
                    continue;
                }
                let candidate_haystack = format!(
                    "{} {}",
                    candidate.title.to_lowercase(),
                    candidate.snippet.to_lowercase()
                );
                let candidate_term_matches =
                    count_term_matches(&candidate_haystack, &query_content_words);
                let candidate_phrase_matches =
                    count_phrase_matches(&candidate_haystack, &query_phrases);
                let candidate_medical_matches =
                    medical_attribute_match_count(effective_query, &candidate_haystack);
                let candidate_health_explainer_matches = health_explainer_match_count(
                    effective_query,
                    &candidate_haystack,
                    &candidate.url,
                );
                let candidate_language_learning_matches = language_learning_match_count(
                    effective_query,
                    &candidate_haystack,
                    &candidate.url,
                );
                let candidate_multilingual_technical_matches =
                    multilingual_technical_explainer_match_count(
                        effective_query,
                        &candidate_haystack,
                        &candidate.url,
                    );
                let candidate_comparison_matches =
                    comparison_entity_match_count(effective_query, &candidate_haystack);
                let candidate_license_keepalive = is_license_query(effective_query)
                    && trusted_license_candidate(
                        effective_query,
                        &candidate_haystack,
                        &candidate.url,
                    );
                let candidate_broad_health_matches =
                    broad_health_match_count(effective_query, &candidate_haystack, &candidate.url);
                if strict_health_query
                    && generic_disease_overview(
                        effective_query,
                        &candidate_haystack,
                        &candidate.url,
                    )
                {
                    continue;
                }
                if !query_content_words.is_empty()
                    && candidate_term_matches < relaxed_min_term_matches
                    && candidate_phrase_matches == 0
                    && !candidate_license_keepalive
                {
                    continue;
                }
                if strict_health_query
                    && (!health_backfill_allowed(&candidate.url) || candidate_medical_matches == 0)
                {
                    continue;
                }
                if health_explainer_query && candidate_health_explainer_matches < 2 {
                    continue;
                }
                if language_learning_query && candidate_language_learning_matches < 2 {
                    continue;
                }
                if is_multilingual_technical_explainer_query(effective_query, query_language)
                    && candidate_multilingual_technical_matches < 2
                {
                    continue;
                }
                if intent == rank::fusion::QueryIntent::Comparison
                    && candidate_comparison_matches == 0
                {
                    continue;
                }
                if health_sensitive_query
                    && !strict_health_query
                    && !health_explainer_query
                    && candidate_broad_health_matches < 2
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
        if ranked.is_empty() && !ranked_before_filter.is_empty() && !preserve_precision {
            ranked = ranked_before_filter
                .iter()
                .filter(|r| {
                    let haystack =
                        format!("{} {}", r.title.to_lowercase(), r.snippet.to_lowercase());
                    let medical_attribute_matches =
                        medical_attribute_match_count(effective_query, &haystack);
                    let health_explainer_matches =
                        health_explainer_match_count(effective_query, &haystack, &r.url);
                    let language_learning_matches =
                        language_learning_match_count(effective_query, &haystack, &r.url);
                    let multilingual_technical_matches =
                        multilingual_technical_explainer_match_count(
                            effective_query,
                            &haystack,
                            &r.url,
                        );
                    let comparison_matches =
                        comparison_entity_match_count(effective_query, &haystack);
                    let license_keepalive = is_license_query(effective_query)
                        && trusted_license_candidate(effective_query, &haystack, &r.url);
                    let broad_health_matches =
                        broad_health_match_count(effective_query, &haystack, &r.url);
                    if strict_health_query
                        && generic_disease_overview(effective_query, &haystack, &r.url)
                    {
                        return false;
                    }
                    let term_matches = count_term_matches(&haystack, &query_content_words);
                    let phrase_matches = count_phrase_matches(&haystack, &query_phrases);
                    !is_spam_domain(&r.url)
                        && !is_image_url(&r.url)
                        && !community_fallback_blocked(
                            intent,
                            health_sensitive_query,
                            language_learning_query,
                            multilingual_foreign_query,
                            r,
                        )
                        && (!has_strong_health_candidates
                            || strong_health_candidate(r, effective_query))
                        && (!has_strong_technical_candidates
                            || strong_technical_candidate(r, intent))
                        && (query_content_words.is_empty()
                            || term_matches >= relaxed_min_term_matches
                            || phrase_matches > 0
                            || license_keepalive)
                        && (!health_explainer_query || health_explainer_matches >= 2)
                        && (!language_learning_query || language_learning_matches >= 2)
                        && (!is_multilingual_technical_explainer_query(
                            effective_query,
                            query_language,
                        ) || multilingual_technical_matches >= 2)
                        && (intent != rank::fusion::QueryIntent::Comparison
                            || comparison_matches > 0)
                        && (!health_sensitive_query
                            || strict_health_query
                            || health_explainer_query
                            || broad_health_matches >= 2)
                        && (!strict_health_query
                            || (health_backfill_allowed(&r.url) && medical_attribute_matches > 0))
                })
                .take(requested_max as usize)
                .cloned()
                .collect();
        }

        // Step 6: Diversify domains in top-N, then take top N.
        ranked = diversify_by_domain(ranked, requested_max as usize);

        if strict_health_query {
            ranked.retain(|r| {
                let haystack = format!("{} {}", r.title.to_lowercase(), r.snippet.to_lowercase());
                !generic_disease_overview(effective_query, &haystack, &r.url)
                    && health_backfill_allowed(&r.url)
                    && medical_attribute_match_count(effective_query, &haystack) > 0
            });
        }

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

        ranked.truncate(requested_max as usize);

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
fn extract_content_words(query: &str, language: Option<&str>) -> Vec<String> {
    let stopwords = stopwords_for_language(language);

    let mut words = query
        .split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|w| w.len() >= 2 && !stopwords.contains(&w.as_str()))
        // Keep short numeric tokens (error codes like 429, 404, 500; versions like v8)
        // but filter out years (4-digit numbers) and plain long numbers
        .filter(|w| {
            if w.chars().all(|c| c.is_ascii_digit()) {
                w.len() == 3 // keep 3-digit codes (404, 429, 500, 754)
            } else {
                true
            }
        })
        .fold(Vec::new(), |mut acc, word| {
            if !acc.contains(&word) {
                acc.push(word);
            }
            acc
        });

    if words.len() > 8 {
        let tail_start = words.len().saturating_sub(4);
        let mut reduced = Vec::with_capacity(8);
        for word in words
            .iter()
            .take(3)
            .chain(words.iter().skip(tail_start.saturating_sub(1)))
        {
            if !reduced.contains(word) {
                reduced.push(word.clone());
            }
        }
        words = reduced;
    }

    words
}

fn stopwords_for_language(language: Option<&str>) -> &'static [&'static str] {
    const ENGLISH_STOPWORDS: &[&str] = &[
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
    const GERMAN_STOPWORDS: &[&str] = &[
        "der", "die", "das", "und", "oder", "aber", "in", "im", "am", "an", "zu", "zum", "zur",
        "für", "von", "mit", "ist", "sind", "war", "waren", "sein", "wird", "werden", "hat",
        "haben", "was", "wie", "wo", "wann", "warum", "welche", "welcher", "welches", "ein",
        "eine", "einer", "einem", "einen", "nicht", "mehr", "weniger", "auch", "über", "unter",
        "nach", "vor", "bei", "als", "denn", "einfach",
    ];
    const SPANISH_STOPWORDS: &[&str] = &[
        "el", "la", "los", "las", "un", "una", "unos", "unas", "y", "o", "pero", "de", "del", "en",
        "por", "para", "con", "sin", "es", "son", "fue", "fueron", "ser", "estar", "como", "qué",
        "que", "dónde", "donde", "cuándo", "cuando", "cuál", "cual", "porque", "mejor", "más",
        "mas", "muy", "también", "tambien",
    ];
    const FRENCH_STOPWORDS: &[&str] = &[
        "le", "la", "les", "un", "une", "des", "du", "de", "et", "ou", "mais", "dans", "sur",
        "pour", "avec", "sans", "est", "sont", "était", "etait", "étaient", "etaient", "être",
        "etre", "avoir", "comment", "quoi", "qui", "que", "quel", "quelle", "quelles", "pourquoi",
        "où", "ou", "quand", "plus", "moins", "très", "tres", "aussi",
    ];

    match language {
        Some("de") => GERMAN_STOPWORDS,
        Some("es") => SPANISH_STOPWORDS,
        Some("fr") => FRENCH_STOPWORDS,
        _ => ENGLISH_STOPWORDS,
    }
}

fn extract_specificity_words(query_content_words: &[String]) -> Vec<String> {
    if query_content_words.len() < 6 {
        return Vec::new();
    }

    query_content_words
        .iter()
        .rev()
        .filter(|word| word.len() >= 3 && !is_generic_specificity_word(word))
        .take(3)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

fn is_generic_specificity_word(word: &str) -> bool {
    matches!(
        word,
        "comparison"
            | "compare"
            | "versus"
            | "guide"
            | "tutorial"
            | "overview"
            | "review"
            | "reviews"
            | "explained"
            | "introduction"
            | "latest"
            | "best"
            | "top"
            | "medication"
            | "medicine"
            | "drug"
            | "drugs"
            | "treatment"
            | "treatments"
            | "symptom"
            | "symptoms"
            | "side"
            | "effect"
            | "effects"
            | "blood"
            | "pressure"
            | "hypertension"
            | "antihypertensive"
    )
}

fn extract_query_phrases(query: &str, language: Option<&str>) -> Vec<String> {
    const ALWAYS_KEEP_PHRASES: &[&str] = &["blood pressure", "side effects", "machine learning"];
    let stopwords = stopwords_for_language(language);
    let tokens = query
        .split_whitespace()
        .map(|word| {
            word.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|word| word.len() >= 2 && !stopwords.contains(&word.as_str()))
        .collect::<Vec<_>>();

    if tokens.len() < 2 {
        return Vec::new();
    }

    let mut phrases = Vec::new();
    for window in tokens.windows(2) {
        let left = &window[0];
        let right = &window[1];
        let phrase = format!("{left} {right}");
        if !ALWAYS_KEEP_PHRASES.contains(&phrase.as_str()) {
            if is_generic_specificity_word(left) && is_generic_specificity_word(right) {
                continue;
            }
            if left.len() < 3 && right.len() < 3 {
                continue;
            }
        }
        if !phrases.contains(&phrase) {
            phrases.push(phrase);
        }
    }

    if phrases.len() > 5 {
        let tail_start = phrases.len().saturating_sub(3);
        let mut reduced = Vec::with_capacity(5);
        for phrase in phrases
            .iter()
            .take(2)
            .chain(phrases.iter().skip(tail_start))
        {
            if !reduced.contains(phrase) {
                reduced.push(phrase.clone());
            }
        }
        phrases = reduced;
    }

    phrases
}

fn count_term_matches(haystack: &str, terms: &[String]) -> usize {
    let tokens: Vec<&str> = haystack
        .split(|c: char| !c.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .collect();

    terms
        .iter()
        .filter(|term| {
            if term.len() <= 3 {
                tokens.iter().any(|token| token.eq_ignore_ascii_case(term))
            } else {
                haystack.contains(term.as_str())
            }
        })
        .count()
}

fn count_phrase_matches(haystack: &str, phrases: &[String]) -> usize {
    phrases
        .iter()
        .filter(|phrase| haystack.contains(phrase.as_str()))
        .count()
}

fn is_short_concept_query(
    term_count: usize,
    intent: rank::fusion::QueryIntent,
    strict_health_query: bool,
    query: &str,
) -> bool {
    term_count > 0
        && term_count <= 3
        && intent != rank::fusion::QueryIntent::CurrentEvents
        && intent != rank::fusion::QueryIntent::HowTo
        && intent != rank::fusion::QueryIntent::Code
        && intent != rank::fusion::QueryIntent::Comparison
        && !is_practical_query(query)
        && !strict_health_query
}

fn normalize_exact_match_text(text: &str) -> String {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(|token| token.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ")
}

fn short_concept_title_match_boost(query: &str, title: &str, url: &str) -> f64 {
    let query_norm = normalize_exact_match_text(query);
    if query_norm.is_empty() {
        return 1.0;
    }

    let title_norm = normalize_exact_match_text(title);
    let url_norm = normalize_exact_match_text(url);

    if title_norm == query_norm {
        return 1.45;
    }
    if query_norm.contains(' ') {
        if title_norm.starts_with(&format!("{query_norm} ")) || title_norm.contains(&query_norm) {
            return 1.30;
        }
    } else if title_norm
        .split_whitespace()
        .any(|token| token == query_norm.as_str())
    {
        return 1.20;
    }
    if url_norm.contains(&query_norm) {
        return 1.10;
    }
    1.0
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

fn is_health_sensitive_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    [
        "medication",
        "medicine",
        "drug",
        "drugs",
        "dose",
        "dosage",
        "side effect",
        "side effects",
        "symptom",
        "symptoms",
        "treatment",
        "treatments",
        "diagnosis",
        "disease",
        "blood pressure",
        "hypertension",
        "diabetes",
        "vaccine",
        "vaccination",
        "therapy",
        "pain",
        "cancer",
        "insulin",
        "sleep apnea",
        "apnea",
    ]
    .iter()
    .any(|pattern| lower.contains(pattern))
}

fn is_strict_health_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    let has_medication_word = [
        "medication",
        "medicine",
        "drug",
        "drugs",
        "dose",
        "dosage",
        "side effect",
        "side effects",
    ]
    .iter()
    .any(|pattern| lower.contains(pattern));

    let is_health_comparison = has_medication_word
        && (lower.contains("compare")
            || lower.contains("comparison")
            || lower.contains("versus")
            || lower.contains(" vs "));

    let has_medication_signal = has_medication_word || is_health_comparison;

    let has_targeted_treatment_signal =
        (lower.contains("treatment") || lower.contains("treatments") || lower.contains("therapy"))
            && (lower.contains("medication")
                || lower.contains("medicine")
                || lower.contains("drug")
                || lower.contains("side effect")
                || lower.contains("comparison")
                || lower.contains("versus")
                || lower.contains(" vs "));
    let has_explainer_signal = [
        "how it works",
        "how does it work",
        "explained",
        "explain",
        "what is",
        "mechanism",
        "technology",
    ]
    .iter()
    .any(|pattern| lower.contains(pattern));

    (has_medication_signal || has_targeted_treatment_signal) && !has_explainer_signal
}

fn is_health_explainer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    is_health_sensitive_query(query)
        && [
            "how it works",
            "how does it work",
            "explained",
            "explain",
            "what is",
            "mechanism",
            "technology",
            "works by",
        ]
        .iter()
        .any(|pattern| lower.contains(pattern))
}

fn is_language_learning_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    let has_learning_verb = [
        "learn",
        "learning",
        "apprendre",
        "aprender",
        "lernen",
        "study",
        "studying",
    ]
    .iter()
    .any(|pattern| lower.contains(pattern));
    let has_language_target = [
        "french",
        "français",
        "francais",
        "spanish",
        "español",
        "espanol",
        "german",
        "deutsch",
        "italian",
        "italiano",
    ]
    .iter()
    .any(|pattern| lower.contains(pattern));
    has_learning_verb && has_language_target
}

fn health_sensitive_backend_allowed(backend: &BackendId) -> bool {
    matches!(
        backend,
        BackendId::Searxng
            | BackendId::DuckDuckGo
            | BackendId::Google
            | BackendId::Bing
            | BackendId::Brave
            | BackendId::Tavily
            | BackendId::Serper
            | BackendId::Exa
            | BackendId::Firecrawl
    )
}

fn docs_focused_query(query: &str) -> String {
    let lower = query.to_lowercase();
    if ["docs", "documentation", "manual", "reference"]
        .iter()
        .any(|pattern| lower.contains(pattern))
    {
        query.to_string()
    } else {
        let normalized = lower
            .replace("step by step", " ")
            .replace("guide to", " ")
            .replace("guide for", " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        format!("{normalized} docs")
    }
}

fn health_authority_multiplier(url: &str) -> f64 {
    let domain = extract_domain(url);
    match domain.as_str() {
        "nih.gov"
        | "medlineplus.gov"
        | "cdc.gov"
        | "nhs.uk"
        | "mayoclinic.org"
        | "clevelandclinic.org"
        | "heart.org"
        | "drugs.com" => 1.45,
        "webmd.com" | "healthline.com" | "medicalnewstoday.com" | "emedicinehealth.com" => 1.25,
        "wikipedia.org" => 0.25,
        "reddit.com" | "github.com" | "stackoverflow.com" => 0.60,
        _ => 1.0,
    }
}

fn health_backfill_allowed(url: &str) -> bool {
    let domain = extract_domain(url);
    matches!(
        domain.as_str(),
        "nih.gov"
            | "medlineplus.gov"
            | "cdc.gov"
            | "nhs.uk"
            | "mayoclinic.org"
            | "clevelandclinic.org"
            | "heart.org"
            | "drugs.com"
            | "rxlist.com"
            | "webmd.com"
            | "healthline.com"
            | "medicalnewstoday.com"
            | "emedicinehealth.com"
            | "aafp.org"
    )
}

fn medical_attribute_multiplier(query: &str, haystack: &str) -> f64 {
    let query_lower = query.to_lowercase();
    let haystack = haystack.to_lowercase();
    let mut multiplier = 1.0;

    let wants_side_effects =
        query_lower.contains("side effect") || query_lower.contains("adverse effect");
    let wants_comparison = query_lower.contains("comparison")
        || query_lower.contains("compare")
        || query_lower.contains("versus")
        || query_lower.contains(" vs ");

    if wants_side_effects {
        if haystack.contains("side effect")
            || haystack.contains("side effects")
            || haystack.contains("adverse effect")
            || haystack.contains("adverse effects")
            || haystack.contains("risk")
            || haystack.contains("risks")
            || haystack.contains("tolerability")
        {
            multiplier *= 1.35;
        } else {
            multiplier *= 0.70;
        }
    }

    if wants_comparison {
        if haystack.contains("compare")
            || haystack.contains("comparison")
            || haystack.contains("versus")
            || haystack.contains("vs")
            || haystack.contains("differ")
            || haystack.contains("difference")
            || haystack.contains("class")
            || haystack.contains("classes")
        {
            multiplier *= 1.20;
        } else {
            multiplier *= 0.82;
        }
    }

    if haystack.contains("without medication")
        || haystack.contains("lifestyle")
        || haystack.contains("exercise")
    {
        multiplier *= 0.55;
    }

    multiplier
}

fn medical_attribute_match_count(query: &str, haystack: &str) -> usize {
    let query_lower = query.to_lowercase();
    let haystack = haystack.to_lowercase();
    let mut count = 0;

    if (query_lower.contains("side effect") || query_lower.contains("adverse effect"))
        && (haystack.contains("side effect")
            || haystack.contains("side effects")
            || haystack.contains("adverse effect")
            || haystack.contains("adverse effects")
            || haystack.contains("risk")
            || haystack.contains("risks")
            || haystack.contains("tolerability"))
    {
        count += 1;
    }

    if (query_lower.contains("comparison")
        || query_lower.contains("compare")
        || query_lower.contains("versus")
        || query_lower.contains(" vs "))
        && (haystack.contains("compare")
            || haystack.contains("comparison")
            || haystack.contains("comparative")
            || haystack.contains("effectiveness")
            || haystack.contains("versus")
            || haystack.contains("difference")
            || haystack.contains("differ"))
    {
        count += 1;
    }

    if (query_lower.contains("medication")
        || query_lower.contains("drug")
        || query_lower.contains("treatment"))
        && (haystack.contains("antihypertensive")
            || haystack.contains("antihypertensive medication")
            || haystack.contains("antihypertensive medications")
            || haystack.contains("ace inhibitor")
            || haystack.contains("angiotensin-converting enzyme")
            || haystack.contains("arb")
            || haystack.contains("angiotensin receptor blocker")
            || haystack.contains("beta blocker")
            || haystack.contains("beta-blocker")
            || haystack.contains("diuretic")
            || haystack.contains("thiazide")
            || haystack.contains("calcium channel blocker")
            || haystack.contains("ccb"))
    {
        count += 1;
    }

    count
}

fn generic_disease_overview(query: &str, haystack: &str, url: &str) -> bool {
    let query_lower = query.to_lowercase();
    let domain = extract_domain(url);
    let asks_side_effect_comparison = (query_lower.contains("side effect")
        || query_lower.contains("adverse effect"))
        && (query_lower.contains("comparison")
            || query_lower.contains("compare")
            || query_lower.contains("versus")
            || query_lower.contains(" vs "));
    if !asks_side_effect_comparison {
        return false;
    }
    let has_condition_overview = haystack.contains("hypertension")
        && (haystack.contains("fact sheet")
            || haystack.contains("overview")
            || haystack.contains("risk factors")
            || haystack.contains("symptoms")
            || haystack.contains("prevention"));
    let has_explicit_side_effect_specifics = haystack.contains("side effect")
        || haystack.contains("side effects")
        || haystack.contains("adverse effect")
        || haystack.contains("adverse effects")
        || haystack.contains("tolerability")
        || haystack.contains("compare")
        || haystack.contains("comparison")
        || haystack.contains("comparative")
        || haystack.contains("versus")
        || haystack.contains("difference");
    let lacks_class_specifics = !has_explicit_side_effect_specifics
        && !haystack.contains("ace inhibitor")
        && !haystack.contains("arb")
        && !haystack.contains("beta blocker")
        && !haystack.contains("diuretic")
        && !haystack.contains("calcium channel blocker");
    has_condition_overview
        && lacks_class_specifics
        && matches!(
            domain.as_str(),
            "who.int" | "www.who.int" | "wikipedia.org" | "webmd.com" | "www.webmd.com"
        )
}

fn health_explainer_multiplier(query: &str, haystack: &str, url: &str) -> f64 {
    let query_lower = query.to_lowercase();
    let haystack = haystack.to_lowercase();
    let mut multiplier = 1.0;
    let domain = extract_domain(url);

    if haystack.contains("how it works")
        || haystack.contains("mechanism")
        || haystack.contains("explained")
        || haystack.contains("works by")
        || haystack.contains("messenger rna")
        || haystack.contains("technology")
    {
        multiplier *= 1.25;
    }

    if query_lower.contains("mrna vaccine") {
        if haystack.contains("moderna")
            || haystack.contains("pfizer")
            || haystack.contains("comirnaty")
            || haystack.contains("covid-19 vaccine")
        {
            multiplier *= 0.75;
        }
        if domain == "wikipedia.org" {
            multiplier *= 0.35;
        }
        if matches!(
            domain.as_str(),
            "nih.gov" | "medlineplus.gov" | "cdc.gov" | "mayoclinic.org" | "clevelandclinic.org"
        ) {
            multiplier *= 1.20;
        }
    }

    multiplier
}

fn health_explainer_match_count(query: &str, haystack: &str, url: &str) -> usize {
    let query_lower = query.to_lowercase();
    let haystack = haystack.to_lowercase();
    let mut count = 0;
    let domain = extract_domain(url);

    if query_lower.contains("vaccine")
        && (haystack.contains("vaccine")
            || haystack.contains("vaccines")
            || matches!(
                domain.as_str(),
                "nih.gov"
                    | "medlineplus.gov"
                    | "cdc.gov"
                    | "mayoclinic.org"
                    | "clevelandclinic.org"
                    | "hopkinsmedicine.org"
                    | "osu.edu"
            ))
    {
        count += 1;
    }

    if query_lower.contains("mrna")
        && (haystack.contains("mrna") || haystack.contains("messenger rna"))
    {
        count += 1;
    }

    if (query_lower.contains("how it works")
        || query_lower.contains("mechanism")
        || query_lower.contains("technology")
        || query_lower.contains("explained"))
        && (haystack.contains("how it works")
            || haystack.contains("mechanism")
            || haystack.contains("technology")
            || haystack.contains("works by")
            || haystack.contains("explained"))
    {
        count += 1;
    }

    count
}

fn language_learning_multiplier(query: &str, haystack: &str, url: &str) -> f64 {
    let matches = language_learning_match_count(query, haystack, url);
    let mut multiplier = match matches {
        3.. => 1.35,
        2 => 1.15,
        1 => 0.75,
        _ => 0.45,
    };
    let domain = extract_domain(url);
    if domain == "reddit.com" {
        multiplier *= 0.75;
    }
    if haystack.contains("politic")
        || haystack.contains("débat")
        || haystack.contains("debate")
        || haystack.contains("ambassador")
        || haystack.contains("foreign legion")
    {
        multiplier *= 0.40;
    }
    multiplier
}

fn language_learning_match_count(query: &str, haystack: &str, url: &str) -> usize {
    let query_lower = query.to_lowercase();
    let mut count = 0;
    let domain = extract_domain(url);

    if (query_lower.contains("french")
        || query_lower.contains("français")
        || query_lower.contains("francais"))
        && (haystack.contains("french")
            || haystack.contains("français")
            || haystack.contains("francais"))
    {
        count += 1;
    }

    if haystack.contains("learn")
        || haystack.contains("learning")
        || haystack.contains("apprendre")
        || haystack.contains("course")
        || haystack.contains("cours")
        || haystack.contains("grammar")
        || haystack.contains("grammaire")
        || haystack.contains("vocabulary")
        || haystack.contains("vocabulaire")
        || haystack.contains("pronunciation")
        || haystack.contains("prononciation")
        || haystack.contains("lesson")
        || haystack.contains("lessons")
        || haystack.contains("méthode")
        || haystack.contains("methode")
    {
        count += 1;
    }

    if haystack.contains("language")
        || haystack.contains("langue")
        || haystack.contains("fluency")
        || haystack.contains("practice")
        || haystack.contains("pratiquer")
        || matches!(
            domain.as_str(),
            "duolingo.com"
                | "wikihow.com"
                | "talkpal.ai"
                | "lawlessfrench.com"
                | "frenchpod101.com"
                | "lingq.com"
                | "tv5monde.com"
                | "rfi.fr"
                | "francaisfacile.com"
                | "francaisavecpierre.com"
                | "ef.fr"
                | "preply.com"
        )
    {
        count += 1;
    }

    count
}

fn is_multilingual_technical_explainer_query(query: &str, language: Option<&str>) -> bool {
    if !matches!(language, Some("bn" | "bd" | "es" | "de" | "fr")) {
        return false;
    }
    let lower = query.to_lowercase();
    [
        "kubernetes",
        "container",
        "orchestration",
        "database",
        "dbms",
        "networking",
        "python",
        "programming",
        "artificial intelligence",
        "machine learning",
        "কুবেরনেটিস",
        "কন্টেইনার",
        "অর্কেস্ট্রেশন",
        "ডেটাবেস",
        "নেটওয়ার্কিং",
        "পাইথন",
        "প্রোগ্রামিং",
        "কৃত্রিম বুদ্ধিমত্তা",
    ]
    .iter()
    .any(|pattern| lower.contains(pattern))
}

fn multilingual_technical_explainer_multiplier(query: &str, haystack: &str, url: &str) -> f64 {
    let mut multiplier = 1.0;
    let domain = extract_domain(url);
    if matches!(
        domain.as_str(),
        "cloud.google.com"
            | "docs.cloud.google.com"
            | "kubernetes.io"
            | "docs.python.org"
            | "python.org"
            | "ibm.com"
            | "coursera.org"
            | "networklessons.com"
            | "computernetworkingnotes.com"
            | "oracle.com"
            | "mongodb.com"
            | "postgresql.org"
            | "aws.amazon.com"
            | "developer.mozilla.org"
    ) {
        multiplier *= 1.30;
    }
    if domain == "wikipedia.org" {
        multiplier *= 0.30;
    }
    if matches!(
        domain.as_str(),
        "reddit.com" | "redd.it" | "ycombinator.com" | "youtube.com" | "youtu.be"
    ) {
        multiplier *= 0.20;
    }

    let query_lower = query.to_lowercase();
    if (query_lower.contains("database") || query_lower.contains("ডেটাবেস"))
        && (haystack.contains("database")
            || haystack.contains("dbms")
            || haystack.contains("database management system"))
    {
        multiplier *= 1.15;
    }
    if (query_lower.contains("networking") || query_lower.contains("নেটওয়ার্কিং"))
        && (haystack.contains("networking")
            || haystack.contains("network")
            || haystack.contains("computer network"))
    {
        multiplier *= 1.15;
    }
    if (query_lower.contains("artificial intelligence") || query_lower.contains("কৃত্রিম বুদ্ধিমত্তা"))
        && (haystack.contains("artificial intelligence") || haystack.contains("ai"))
    {
        multiplier *= 1.15;
    }
    if (query_lower.contains("kubernetes") || query_lower.contains("কুবেরনেটিস"))
        && haystack.contains("kubernetes")
    {
        multiplier *= 1.15;
    }
    multiplier
}

fn multilingual_technical_explainer_match_count(query: &str, haystack: &str, url: &str) -> usize {
    let query_lower = query.to_lowercase();
    let domain = extract_domain(url);
    let mut count = 0;

    if (query_lower.contains("database") || query_lower.contains("ডেটাবেস"))
        && (haystack.contains("database")
            || haystack.contains("dbms")
            || haystack.contains("database management system"))
    {
        count += 1;
    }
    if (query_lower.contains("networking") || query_lower.contains("নেটওয়ার্কিং"))
        && (haystack.contains("networking")
            || haystack.contains("network")
            || haystack.contains("computer network"))
    {
        count += 1;
    }
    if (query_lower.contains("artificial intelligence") || query_lower.contains("কৃত্রিম বুদ্ধিমত্তা"))
        && (haystack.contains("artificial intelligence") || haystack.contains("ai"))
    {
        count += 1;
    }
    if (query_lower.contains("kubernetes") || query_lower.contains("কুবেরনেটিস"))
        && haystack.contains("kubernetes")
    {
        count += 1;
    }
    if (query_lower.contains("python") || query_lower.contains("পাইথন"))
        && (haystack.contains("python") || haystack.contains("programming language"))
    {
        count += 1;
    }
    if (query_lower.contains("programming") || query_lower.contains("প্রোগ্রামিং"))
        && (haystack.contains("programming")
            || haystack.contains("coding")
            || haystack.contains("learn to code"))
    {
        count += 1;
    }
    if matches!(
        domain.as_str(),
        "cloud.google.com"
            | "docs.cloud.google.com"
            | "kubernetes.io"
            | "docs.python.org"
            | "python.org"
            | "ibm.com"
            | "coursera.org"
            | "networklessons.com"
            | "computernetworkingnotes.com"
            | "oracle.com"
            | "mongodb.com"
            | "postgresql.org"
            | "techtarget.com"
            | "aws.amazon.com"
            | "developer.mozilla.org"
    ) {
        count += 1;
    }
    count
}

fn multilingual_technical_explainer_blocked_domain(url: &str) -> bool {
    matches!(
        extract_domain(url).as_str(),
        "arxiv.org" | "github.com" | "zhihu.com" | "reddit.com" | "redd.it" | "ycombinator.com"
    )
}

async fn plan_bengali_technical_query(query: &str) -> Option<BengaliPlannerPlan> {
    let ai_config = AiConfig {
        default_model: Some("qwen3.5:8b".to_string()),
        timeout_secs: 20,
        temperature: 0.1,
        ..AiConfig::default()
    };
    let ollama = OllamaClient::new(&ai_config);
    if !ollama.is_available().await {
        return None;
    }
    let system = ChatMessage {
        role: "system".to_string(),
        content: "You are a Bengali technical search planner. Return ONLY compact JSON with keys retry_query (string) and source_pack_queries (array of up to 5 strings). The goal is to find beginner-friendly Bengali or English explainer pages for the user's Bengali query. Prefer trusted domains and concrete retrieval queries. Do not include markdown.".to_string(),
    };
    let user = ChatMessage {
        role: "user".to_string(),
        content: format!(
            "User query: {query}\nReturn JSON only. Example: {{\"retry_query\":\"what is kubernetes docs\",\"source_pack_queries\":[\"site:kubernetes.io kubernetes overview\"]}}"
        ),
    };
    let response = ollama.chat("qwen3.5:8b", &[system, user], 0.1).await.ok()?;
    parse_bengali_planner_response(&response)
}

fn parse_bengali_planner_response(text: &str) -> Option<BengaliPlannerPlan> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    let json_text = &text[start..=end];
    let value: serde_json::Value = serde_json::from_str(json_text).ok()?;
    let retry_query = value
        .get("retry_query")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let mut source_pack_queries = value
        .get("source_pack_queries")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .take(5)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    source_pack_queries.retain(|q| !q.is_empty());
    if retry_query.is_none() && source_pack_queries.is_empty() {
        return None;
    }
    Some(BengaliPlannerPlan {
        retry_query,
        source_pack_queries,
    })
}

fn bengali_technical_retry_query(original_query: &str, effective_query: &str) -> String {
    let original_lower = original_query.to_lowercase();
    if original_lower.contains("কৃত্রিম বুদ্ধিমত্তা") {
        return "what is artificial intelligence ai basics beginner guide ibm microsoft google"
            .to_string();
    }
    if original_lower.contains("ডেটাবেস") || effective_query.contains("database") {
        return "what is database management system dbms basics oracle mongodb postgresql"
            .to_string();
    }
    if original_lower.contains("নেটওয়ার্কিং") {
        return "what is computer networking basics guide cisco cloudflare".to_string();
    }
    if original_lower.contains("কুবেরনেটিস") {
        return "what is kubernetes container orchestration basics docs".to_string();
    }
    if original_lower.contains("পাইথন") {
        return "what is python programming language basics docs".to_string();
    }
    if original_lower.contains("প্রোগ্রামিং") {
        return "how to learn programming basics beginner guide".to_string();
    }
    docs_focused_query(effective_query)
}

fn bengali_source_pack_queries(original_query: &str, effective_query: &str) -> Vec<String> {
    let original_lower = original_query.to_lowercase();
    let effective_lower = effective_query.to_lowercase();
    if original_lower.contains("কৃত্রিম বুদ্ধিমত্তা")
        || effective_lower.contains("artificial intelligence")
    {
        return vec![
            "site:ibm.com artificial intelligence basics".to_string(),
            "site:cloud.google.com artificial intelligence overview".to_string(),
            "site:microsoft.com artificial intelligence beginner guide".to_string(),
            "site:teachers.gov.bd কৃত্রিম বুদ্ধিমত্তা".to_string(),
            "site:prothomalo.com কৃত্রিম বুদ্ধিমত্তা".to_string(),
        ];
    }
    if original_lower.contains("ডেটাবেস") || effective_lower.contains("database") {
        return vec![
            "site:oracle.com database management system basics".to_string(),
            "site:mongodb.com what is a database".to_string(),
            "site:postgresql.org docs introduction database".to_string(),
        ];
    }
    if original_lower.contains("নেটওয়ার্কিং") || effective_lower.contains("networking")
    {
        return vec![
            "site:cloudflare.com what is computer networking".to_string(),
            "site:cisco.com networking basics".to_string(),
            "site:networklessons.com networking basics".to_string(),
        ];
    }
    if original_lower.contains("কুবেরনেটিস") || effective_lower.contains("kubernetes")
    {
        return vec![
            "site:kubernetes.io kubernetes overview".to_string(),
            "site:cloud.google.com kubernetes overview".to_string(),
            "site:aws.amazon.com kubernetes basics".to_string(),
        ];
    }
    Vec::new()
}

fn multilingual_technical_trusted_source(url: &str) -> bool {
    matches!(
        extract_domain(url).as_str(),
        "kubernetes.io"
            | "cloud.google.com"
            | "docs.cloud.google.com"
            | "aws.amazon.com"
            | "azure.microsoft.com"
            | "redhat.com"
            | "ibm.com"
            | "oracle.com"
            | "postgresql.org"
            | "mongodb.com"
            | "microsoft.com"
            | "cloudflare.com"
            | "cisco.com"
            | "docs.python.org"
            | "python.org"
            | "developer.mozilla.org"
            | "computernetworkingnotes.com"
            | "networklessons.com"
            | "digitalocean.com"
            | "geeksforgeeks.org"
            | "techtarget.com"
    )
}

fn bengali_ai_direct_source(url: &str) -> bool {
    matches!(
        extract_domain(url).as_str(),
        "teachers.gov.bd" | "prothomalo.com"
    )
}

fn is_license_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    (lower.contains("license") || lower.contains("licence"))
        && (lower.contains("mit") || lower.contains("apache") || lower.contains("gpl"))
}

fn license_query_multiplier(query: &str, haystack: &str, url: &str) -> f64 {
    let domain = extract_domain(url);
    let mut multiplier = comparison_multiplier(query, haystack, url);
    if matches!(
        domain.as_str(),
        "choosealicense.com" | "opensource.org" | "gnu.org" | "apache.org" | "soos.io"
    ) {
        multiplier *= 1.35;
    }
    if domain == "wikipedia.org" {
        multiplier *= 0.85;
    }
    if matches!(domain.as_str(), "reddit.com" | "medium.com" | "dev.to") {
        multiplier *= 0.45;
    }
    multiplier
}

fn is_philosophy_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("utilitarianism")
        || lower.contains("deontology")
        || lower.contains("deontological")
        || lower.contains("ethics")
        || lower.contains("moral philosophy")
}

fn philosophy_query_multiplier(query: &str, haystack: &str, url: &str) -> f64 {
    let domain = extract_domain(url);
    let mut multiplier = comparison_multiplier(query, haystack, url);
    if matches!(
        domain.as_str(),
        "stanford.edu" | "utm.edu" | "britannica.com" | "plato.stanford.edu"
    ) {
        multiplier *= 1.35;
    }
    if matches!(domain.as_str(), "reddit.com" | "medium.com") {
        multiplier *= 0.45;
    }
    multiplier
}

fn license_query_match_count(query: &str, haystack: &str, url: &str) -> usize {
    let mut count = 0;
    let query_lower = query.to_lowercase();
    let domain = extract_domain(url);
    for token in ["mit", "apache", "gpl"] {
        if query_lower.contains(token) && haystack.contains(token) {
            count += 1;
        }
    }
    if haystack.contains("license") || haystack.contains("licenses") {
        count += 1;
    }
    if matches!(
        domain.as_str(),
        "choosealicense.com" | "opensource.org" | "gnu.org" | "apache.org"
    ) {
        count += 1;
    }
    count
}

fn trusted_license_candidate(query: &str, haystack: &str, url: &str) -> bool {
    if license_query_match_count(query, haystack, url) < 2 {
        return false;
    }
    let domain = extract_domain(url);
    matches!(
        domain.as_str(),
        "choosealicense.com"
            | "opensource.org"
            | "gnu.org"
            | "apache.org"
            | "wikipedia.org"
            | "stackoverflow.com"
            | "stackexchange.com"
            | "soos.io"
    )
}

fn philosophy_query_match_count(query: &str, haystack: &str, url: &str) -> usize {
    let mut count = 0;
    let query_lower = query.to_lowercase();
    let domain = extract_domain(url);
    for token in ["utilitarianism", "deontology", "deontological", "ethics"] {
        if query_lower.contains(token) && haystack.contains(token) {
            count += 1;
        }
    }
    if matches!(
        domain.as_str(),
        "stanford.edu" | "utm.edu" | "britannica.com"
    ) {
        count += 1;
    }
    count
}

fn broad_health_multiplier(query: &str, haystack: &str, url: &str) -> f64 {
    let matches = broad_health_match_count(query, haystack, url);
    let mut multiplier = match matches {
        3.. => 1.30,
        2 => 1.12,
        1 => 0.80,
        _ => 0.45,
    };
    let domain = extract_domain(url);
    if matches!(domain.as_str(), "reddit.com" | "youtube.com" | "youtu.be") {
        multiplier *= 0.35;
    }
    multiplier
}

fn broad_health_match_count(query: &str, haystack: &str, url: &str) -> usize {
    let query_lower = query.to_lowercase();
    let mut count = 0;
    let domain = extract_domain(url);

    // 1. Generic disease/condition matcher:
    // If the query contains any long, specific words that appear in the haystack, count it.
    let content_words = extract_content_words(&query_lower, None);
    let mut specific_health_matches = 0;
    for word in &content_words {
        if word.len() > 4
            && haystack.contains(word.as_str())
            && !["symptom", "cause", "treatment", "diagnosis", "disease"].contains(&word.as_str())
        {
            specific_health_matches += 1;
        }
    }
    // Cap at 1 for the condition itself to require a facet match below
    if specific_health_matches > 0 {
        count += 1;
    }
    // For phrases like "long covid"
    if query_lower.contains("long covid") && haystack.contains("long covid") {
        count += 1; // Extra boost if exact phrase matches
    }

    // 2. Intent facet matchers (causes, symptoms, treatments)
    if (query_lower.contains("cause")
        || query_lower.contains("diagnosis")
        || query_lower.contains("symptom")
        || query_lower.contains("treatment")
        || query_lower.contains("option"))
        && (haystack.contains("cause")
            || haystack.contains("diagnos")
            || haystack.contains("symptom")
            || haystack.contains("treatment")
            || haystack.contains("therapy")
            || haystack.contains("option"))
    {
        count += 1;
    }

    // 3. Keep hardcoded common ones as fallback for synonyms
    if query_lower.contains("sleep apnea")
        && (haystack.contains("obstructive sleep apnea") || haystack.contains("osa"))
    {
        count += 1;
    }
    if query_lower.contains("diabetes")
        && (haystack.contains("type 2 diabetes") || haystack.contains("t2d"))
    {
        count += 1;
    }
    if (query_lower.contains("guideline")
        || query_lower.contains("guidelines")
        || query_lower.contains("recommendation")
        || query_lower.contains("standards of care"))
        && (haystack.contains("guideline")
            || haystack.contains("recommendation")
            || haystack.contains("standards of care")
            || haystack.contains("consensus"))
    {
        count += 1;
    }
    if query_lower.contains("management")
        && (haystack.contains("management")
            || haystack.contains("care")
            || haystack.contains("therapy"))
    {
        count += 1;
    }
    if (query_lower.contains("creatine")
        || query_lower.contains("supplement")
        || query_lower.contains("supplementation"))
        && (haystack.contains("creatine")
            || haystack.contains("supplement")
            || haystack.contains("supplementation"))
    {
        count += 1;
    }
    if (query_lower.contains("benefit") || query_lower.contains("risk"))
        && (haystack.contains("benefit")
            || haystack.contains("benefits")
            || haystack.contains("risk")
            || haystack.contains("risks")
            || haystack.contains("safety")
            || haystack.contains("side effect")
            || haystack.contains("side effects"))
    {
        count += 1;
    }
    if matches!(
        domain.as_str(),
        "nih.gov"
            | "medlineplus.gov"
            | "cdc.gov"
            | "nhs.uk"
            | "mayoclinic.org"
            | "clevelandclinic.org"
            | "webmd.com"
            | "healthline.com"
            | "medicalnewstoday.com"
            | "emedicinehealth.com"
            | "lung.org"
            | "diabetes.org"
            | "aace.com"
            | "acponline.org"
            | "va.gov"
            | "medscape.com"
    ) {
        count += 1;
    }

    count
}

fn comparison_multiplier(query: &str, haystack: &str, url: &str) -> f64 {
    let entity_matches = comparison_entity_match_count(query, haystack);
    let mut multiplier = match entity_matches {
        3.. => 1.35,
        2 => 1.18,
        1 => 0.72,
        _ => 0.40,
    };
    let domain = extract_domain(url);
    if matches!(
        domain.as_str(),
        "reddit.com" | "youtube.com" | "youtu.be" | "arxiv.org"
    ) {
        multiplier *= 0.35;
    }
    if haystack.contains("comparison")
        || haystack.contains("compare")
        || haystack.contains("versus")
        || haystack.contains("difference")
        || haystack.contains("which to choose")
        || haystack.contains("pros and cons")
    {
        multiplier *= 1.15;
    }
    multiplier
}

fn comparison_entity_match_count(query: &str, haystack: &str) -> usize {
    extract_comparison_entities(query)
        .into_iter()
        .filter(|entity| {
            if entity.len() <= 3 {
                haystack
                    .split(|c: char| !c.is_alphanumeric() && c != '#')
                    .any(|token| token == entity)
            } else if entity.contains(' ') {
                // If the entity is multiple words, require all of them to be present
                entity
                    .split_whitespace()
                    .all(|word| haystack.contains(word))
            } else {
                haystack.contains(entity)
            }
        })
        .count()
}

fn comparison_entity_threshold(query: &str) -> usize {
    let entity_count = extract_comparison_entities(query).len();
    if entity_count >= 2 {
        2
    } else {
        1
    }
}

fn extract_comparison_entities(query: &str) -> Vec<String> {
    let lower = query.to_lowercase();
    let mut normalized = lower
        .replace(" compared to ", " vs ")
        .replace(" versus ", " vs ")
        .replace(" vs. ", " vs ");
    if !normalized.contains(" vs ") {
        return Vec::new();
    }
    normalized = normalized.replace("comparison", " ");
    normalized
        .split(" vs ")
        .filter_map(|segment| {
            let tokens = segment
                .split_whitespace()
                .filter(|token| !is_generic_comparison_token(token))
                .collect::<Vec<_>>();
            if tokens.is_empty() {
                None
            } else {
                Some(tokens.join(" "))
            }
        })
        .collect()
}

fn is_generic_comparison_token(token: &str) -> bool {
    matches!(
        token,
        "open"
            | "source"
            | "license"
            | "licenses"
            | "which"
            | "database"
            | "databases"
            | "framework"
            | "frameworks"
            | "comparison"
            | "compare"
            | "choose"
            | "to"
            | "the"
            | "a"
            | "an"
            | "for"
            | "best"
            | "latest"
            | "guide"
            | "performance"
            | "benchmark"
            | "benchmarks"
            | "speed"
            | "test"
            | "tests"
            | "2024"
            | "2025"
            | "2026"
    )
}

fn technical_howto_multiplier(query: &str, haystack: &str, url: &str) -> f64 {
    let domain = extract_domain(url);
    let mut multiplier = match domain.as_str() {
        "reddit.com" => 0.30,
        "stackoverflow.com" | "superuser.com" | "serverfault.com" => 1.20,
        "docs.docker.com" | "developer.mozilla.org" | "kubernetes.io" | "docs.python.org" => 1.30,
        _ => 1.0,
    };
    if strong_technical_coverage(query, haystack, url) {
        multiplier *= 1.20;
    }
    multiplier
}

fn strong_technical_coverage(query: &str, haystack: &str, url: &str) -> bool {
    let query_lower = query.to_lowercase();
    let technical_terms = [
        "docker",
        "kubernetes",
        "ci",
        "cd",
        "pipeline",
        "pipelines",
        "deploy",
        "deployment",
        "production",
        "machine learning",
        "model",
    ];
    let matched = technical_terms
        .iter()
        .filter(|term| query_lower.contains(**term) && haystack.contains(**term))
        .count();
    matched >= 3 || (matched >= 2 && strong_technical_candidate_url(url))
}

fn strong_technical_candidate_url(url: &str) -> bool {
    let domain = extract_domain(url);
    url.contains("docs.docker.com")
        || matches!(
            domain.as_str(),
            "docker.com"
                | "kubernetes.io"
                | "mozilla.org"
                | "python.org"
                | "stackoverflow.com"
                | "superuser.com"
                | "serverfault.com"
        )
}

fn community_fallback_blocked(
    intent: rank::fusion::QueryIntent,
    health_sensitive_query: bool,
    language_learning_query: bool,
    multilingual_foreign_query: bool,
    result: &ResultItem,
) -> bool {
    if !(health_sensitive_query
        || language_learning_query
        || multilingual_foreign_query
        || matches!(
            intent,
            rank::fusion::QueryIntent::HowTo | rank::fusion::QueryIntent::Code
        ))
    {
        return false;
    }

    if matches!(result.backend, BackendId::Reddit | BackendId::HackerNews) {
        return true;
    }

    matches!(
        extract_domain(&result.url).as_str(),
        "reddit.com" | "redd.it" | "ycombinator.com" | "youtube.com" | "youtu.be"
    )
}

fn strong_health_candidate(result: &ResultItem, query: &str) -> bool {
    let haystack = format!(
        "{} {}",
        result.title.to_lowercase(),
        result.snippet.to_lowercase()
    );
    let domain = extract_domain(&result.url);
    if !matches!(
        domain.as_str(),
        "nih.gov"
            | "medlineplus.gov"
            | "cdc.gov"
            | "nhs.uk"
            | "mayoclinic.org"
            | "clevelandclinic.org"
            | "webmd.com"
            | "healthline.com"
            | "medicalnewstoday.com"
            | "emedicinehealth.com"
            | "heart.org"
            | "lung.org"
            | "ncbi.nlm.nih.gov"
            | "pubmed.ncbi.nlm.nih.gov"
            | "pmc.ncbi.nlm.nih.gov"
            | "rxlist.com"
            | "uptodate.com"
            | "aafp.org"
    ) {
        return false;
    }
    broad_health_match_count(query, &haystack, &result.url) >= 2
        || medical_attribute_match_count(query, &haystack) > 0
        || health_explainer_match_count(query, &haystack, &result.url) >= 2
}

fn strong_technical_candidate(result: &ResultItem, intent: rank::fusion::QueryIntent) -> bool {
    strong_technical_candidate_url(&result.url)
        || (intent == rank::fusion::QueryIntent::Code
            && extract_domain(&result.url) == "github.com")
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
        // DDG HTML scraping is flaky under load; keep it bounded and let SearXNG
        // handle deeper general-web recall for fragile queries.
        BackendId::DuckDuckGo => Duration::from_secs(5),
        BackendId::Google | BackendId::Bing => Duration::from_secs(3),
        _ => default_timeout,
    };
    default_timeout.min(cap)
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn extract_content_words_strips_stopwords() {
        let words = extract_content_words("best Rust async runtimes 2025", None);
        assert!(words.contains(&"rust".to_string()));
        assert!(words.contains(&"async".to_string()));
        assert!(words.contains(&"runtimes".to_string()));
        assert!(!words.contains(&"2025".to_string()));
        assert!(!words.contains(&"best".to_string())); // stopword
    }

    #[test]
    fn extract_content_words_short_words_removed() {
        let words = extract_content_words("is it a good idea", None);
        // "is", "it", "a", "good" are stopwords or <3 chars
        assert!(words.is_empty() || !words.contains(&"is".to_string()));
    }

    #[test]
    fn extract_content_words_uses_german_stopwords() {
        let words = extract_content_words("was ist quantencomputing einfach erklaert", Some("de"));
        assert!(!words.contains(&"was".to_string()));
        assert!(!words.contains(&"ist".to_string()));
        assert!(words.contains(&"quantencomputing".to_string()));
        assert!(words.contains(&"erklaert".to_string()));
    }

    #[test]
    fn extract_content_words_trims_long_queries_to_anchor_terms() {
        let words = extract_content_words(
            "how do i configure rust tokio websocket reconnect backoff heartbeat timeout handling in production",
            None,
        );
        assert!(words.len() <= 8);
        assert!(words.contains(&"rust".to_string()));
        assert!(words.contains(&"production".to_string()));
    }

    #[test]
    fn extract_content_words_keeps_tail_specificity_for_long_queries() {
        let words = extract_content_words(
            "step by step guide to deploying a machine learning model to production using Docker Kubernetes and CI CD pipelines",
            None,
        );
        assert!(words.contains(&"docker".to_string()));
        assert!(words.contains(&"kubernetes".to_string()));
        assert!(words.contains(&"pipelines".to_string()));
    }

    #[test]
    fn extract_specificity_words_prefers_long_query_tail_terms() {
        let specificity = extract_specificity_words(&[
            "deploying".to_string(),
            "machine".to_string(),
            "learning".to_string(),
            "production".to_string(),
            "docker".to_string(),
            "kubernetes".to_string(),
            "pipelines".to_string(),
        ]);
        assert_eq!(
            specificity,
            vec![
                "docker".to_string(),
                "kubernetes".to_string(),
                "pipelines".to_string()
            ]
        );
    }

    #[test]
    fn extract_specificity_words_skips_generic_comparison_terms() {
        let specificity = extract_specificity_words(&[
            "blood".to_string(),
            "pressure".to_string(),
            "medication".to_string(),
            "side".to_string(),
            "effects".to_string(),
            "comparison".to_string(),
        ]);
        assert!(specificity.is_empty());
    }

    #[test]
    fn parse_bengali_planner_response_extracts_retry_and_sources() {
        let plan = parse_bengali_planner_response(
            r#"{"retry_query":"what is artificial intelligence basics","source_pack_queries":["site:ibm.com artificial intelligence basics","site:teachers.gov.bd কৃত্রিম বুদ্ধিমত্তা"]}"#,
        )
        .expect("plan");
        assert_eq!(
            plan.retry_query.as_deref(),
            Some("what is artificial intelligence basics")
        );
        assert_eq!(plan.source_pack_queries.len(), 2);
        assert!(plan.source_pack_queries[0].contains("ibm.com"));
    }

    #[test]
    fn bengali_technical_retry_query_is_topic_specific() {
        assert!(bengali_technical_retry_query("কৃত্রিম বুদ্ধিমত্তা কী", "x").contains("ibm"));
        assert!(bengali_technical_retry_query("ডেটাবেস কী", "x").contains("oracle"));
    }

    #[test]
    fn extract_query_phrases_keeps_salient_medical_bigrams() {
        let phrases =
            extract_query_phrases("blood pressure medication side effects comparison", None);
        assert!(phrases.contains(&"blood pressure".to_string()));
        assert!(phrases.contains(&"side effects".to_string()));
        assert!(!phrases.contains(&"effects comparison".to_string()));
    }

    #[test]
    fn health_authority_multiplier_prefers_clinical_domains() {
        assert!(
            health_authority_multiplier("https://medlineplus.gov/highbloodpressure.html") > 1.0
        );
        assert!(
            health_authority_multiplier("https://en.wikipedia.org/wiki/Antihypertensive") < 1.0
        );
        assert_eq!(
            health_authority_multiplier("https://example.com/article"),
            1.0
        );
    }

    #[test]
    fn health_backfill_allows_clinical_domains_only() {
        assert!(health_backfill_allowed(
            "https://www.webmd.com/hypertension-high-blood-pressure/side-effects-high-blood-pressure-medications"
        ));
        assert!(health_backfill_allowed(
            "https://www.mayoclinic.org/diseases-conditions/high-blood-pressure/in-depth/high-blood-pressure-medication/art-20046280"
        ));
        assert!(!health_backfill_allowed(
            "https://en.wikipedia.org/wiki/Antihypertensive"
        ));
        assert!(!health_backfill_allowed("https://example.com/article"));
    }

    #[test]
    fn medical_attribute_multiplier_prefers_side_effect_comparisons() {
        let query = "hypertension antihypertensive drug classes comparison adverse effects ace inhibitor arb beta blocker diuretic calcium channel blocker";
        let strong = medical_attribute_multiplier(
            query,
            "compare adverse effects and side effects across antihypertensive classes",
        );
        let weak = medical_attribute_multiplier(
            query,
            "how to control high blood pressure without medication and exercise",
        );
        assert!(strong > 1.0);
        assert!(weak < 1.0);
        assert!(strong > weak);
    }

    #[test]
    fn medical_attribute_match_count_requires_comparison_or_side_effect_signals() {
        let query = "hypertension antihypertensive drug classes comparison adverse effects ace inhibitor arb beta blocker diuretic calcium channel blocker";
        let strong = medical_attribute_match_count(
            query,
            "comparative effectiveness and adverse effects across antihypertensive drug classes",
        );
        let weak = medical_attribute_match_count(
            query,
            "hypertension overview and causes from world health organization",
        );
        assert!(strong >= 2);
        assert_eq!(weak, 0);
    }

    #[test]
    fn medical_attribute_match_count_rejects_generic_medication_mentions() {
        let query = "blood pressure medication side effects comparison";
        let generic = medical_attribute_match_count(
            query,
            "high blood pressure is diagnosed if the reading is equal to or greater than 130/80 and medications may be used for treatment",
        );
        let specific = medical_attribute_match_count(
            query,
            "compare side effects of ace inhibitors versus beta blockers and diuretics",
        );
        assert_eq!(generic, 0);
        assert!(specific >= 2);
    }

    #[test]
    fn health_explainer_multiplier_prefers_generic_mechanism_pages() {
        let query = "mRNA vaccine technology how it works";
        let explainer = health_explainer_multiplier(
            query,
            "mRNA vaccine technology explained and how it works by delivering messenger RNA",
            "https://medlineplus.gov/vaccines.html",
        );
        let brand_page = health_explainer_multiplier(
            query,
            "Moderna COVID-19 vaccine information and safety data",
            "https://en.wikipedia.org/wiki/Moderna_COVID-19_vaccine",
        );
        assert!(explainer > 1.0);
        assert!(brand_page < explainer);
    }

    #[test]
    fn health_explainer_match_count_requires_vaccine_and_mechanism_signals() {
        let query = "mRNA vaccine technology how it works";
        let strong = health_explainer_match_count(
            query,
            "mRNA vaccine technology explained and how it works by delivering messenger RNA",
            "https://medlineplus.gov/vaccines.html",
        );
        let weak = health_explainer_match_count(
            query,
            "iphone force shutdown methods and excel tips",
            "https://jingyan.baidu.com/article/example.html",
        );
        assert!(strong >= 3);
        assert_eq!(weak, 0);
    }

    #[test]
    fn health_sensitive_backend_policy_excludes_wikipedia() {
        assert!(health_sensitive_backend_allowed(&BackendId::DuckDuckGo));
        assert!(!health_sensitive_backend_allowed(&BackendId::Wikipedia));
    }

    #[test]
    fn count_term_matches_requires_exact_match_for_short_terms() {
        let haystack = "stock ticker cd projekt and ci systems";
        let terms = vec!["cd".to_string(), "ci".to_string(), "proj".to_string()];
        assert_eq!(count_term_matches(haystack, &terms), 3);

        let noisy_haystack = "academic production medicine";
        let short_terms = vec!["ci".to_string(), "cd".to_string()];
        assert_eq!(count_term_matches(noisy_haystack, &short_terms), 0);
    }

    #[test]
    fn short_concept_query_detection_is_conservative() {
        assert!(is_short_concept_query(
            2,
            rank::fusion::QueryIntent::Informational,
            false,
            "spectator ion"
        ));
        assert!(!is_short_concept_query(
            4,
            rank::fusion::QueryIntent::Informational,
            false,
            "how spectator ions behave in precipitation reactions"
        ));
        assert!(!is_short_concept_query(
            2,
            rank::fusion::QueryIntent::HowTo,
            false,
            "how to use docker"
        ));
    }

    #[test]
    fn short_concept_title_match_boost_prefers_exact_titles() {
        let exact = short_concept_title_match_boost(
            "spectator ion",
            "Spectator ion",
            "https://en.wikipedia.org/wiki/Spectator_ion",
        );
        let partial = short_concept_title_match_boost(
            "spectator ion",
            "Precipitation reaction overview",
            "https://chem.libretexts.org/topics/precipitation_reaction",
        );
        assert!(exact > partial);
        assert!(exact > 1.0);
        assert_eq!(partial, 1.0);
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
    fn health_sensitive_query_detection() {
        assert!(is_health_sensitive_query(
            "blood pressure medication side effects comparison"
        ));
        assert!(is_health_sensitive_query("diabetes treatment options"));
        assert!(is_health_sensitive_query(
            "sleep apnea causes diagnosis treatment options"
        ));
        assert!(!is_health_sensitive_query("vector database comparison"));
    }

    #[test]
    fn broad_health_match_count_captures_diabetes_guidelines() {
        let query = "type 2 diabetes management latest guidelines";
        let count = broad_health_match_count(
            query,
            "standards of care and guideline recommendations for type 2 diabetes management",
            "https://professional.diabetes.org/standards-of-care",
        );
        assert!(count >= 3);
    }

    #[test]
    fn comparison_entity_match_count_requires_multiple_entities() {
        let query = "React vs Vue vs Angular framework comparison 2024";
        let strong = comparison_entity_match_count(
            query,
            "react vs vue vs angular comparison and framework tradeoffs",
        );
        let weak = comparison_entity_match_count(query, "2024 framework trends and benchmarks");
        assert!(strong >= 2);
        assert_eq!(weak, 0);
    }

    #[test]
    fn comparison_entities_strip_generic_tokens() {
        assert_eq!(
            extract_comparison_entities("open source license comparison MIT vs Apache vs GPL"),
            vec!["mit".to_string(), "apache".to_string(), "gpl".to_string()]
        );
        assert_eq!(
            extract_comparison_entities("PostgreSQL vs MySQL which database to choose"),
            vec!["postgresql".to_string(), "mysql".to_string()]
        );
    }

    #[test]
    fn strong_technical_coverage_accepts_multi_anchor_howto_articles() {
        assert!(strong_technical_coverage(
            "how to deploying machine learning model production docker kubernetes ci cd pipelines",
            "deploying machine learning models in production with kubernetes and docker",
            "https://www.buildpiper.io/blogs/deploying-machine-learning-models-in-production-with-kubernetes/",
        ));
    }

    #[test]
    fn language_learning_match_count_accepts_tv5monde() {
        let query = "comment apprendre le français rapidement";
        let matches = language_learning_match_count(
            query,
            "apprendre le français avec des cours et exercices pour débutants",
            "https://apprendre.tv5monde.com/fr",
        );
        assert!(matches >= 2);
    }

    #[test]
    fn multilingual_technical_explainer_detection_catches_bengali_database() {
        assert!(is_multilingual_technical_explainer_query(
            "ডেটাবেস কী",
            Some("bn")
        ));
    }

    #[test]
    fn multilingual_technical_explainer_multiplier_prefers_docs_over_wikipedia() {
        let query = "ডেটাবেস কী";
        let docs = multilingual_technical_explainer_multiplier(
            query,
            "what is database management system dbms fundamentals",
            "https://www.oracle.com/database/what-is-database/",
        );
        let wiki = multilingual_technical_explainer_multiplier(
            query,
            "database encyclopedia overview",
            "https://en.wikipedia.org/wiki/Database",
        );
        assert!(docs > wiki);
    }

    #[test]
    fn multilingual_technical_explainer_match_count_prefers_relevant_docs() {
        let query = "বাংলায় কুবেরনেটিস কী";
        let strong = multilingual_technical_explainer_match_count(
            query,
            "what is kubernetes container orchestration basics",
            "https://kubernetes.io/docs/home/",
        );
        let weak = multilingual_technical_explainer_match_count(
            query,
            "random unrelated baidu page",
            "https://zhidao.baidu.com/question/example.html",
        );
        assert!(strong >= 2);
        assert_eq!(weak, 0);
    }

    #[test]
    fn strong_health_candidate_accepts_rxlist_side_effect_page() {
        let result = ResultItem {
            title: "Blood Pressure Medications: Types, Side Effects".into(),
            url: "https://www.rxlist.com/high_blood_pressure_hypertension_medications/drugs-condition.htm".into(),
            snippet: "ACE inhibitors, beta blockers, diuretics and calcium channel blockers with side effects and uses.".into(),
            rank: 1,
            backend: BackendId::Searxng,
            score: Some(0.5),
            published_date: None,
        };
        assert!(strong_health_candidate(
            &result,
            "blood pressure medication side effects comparison"
        ));
    }

    #[test]
    fn trusted_license_candidate_accepts_choosealicense() {
        assert!(trusted_license_candidate(
            "open source license comparison MIT vs Apache vs GPL",
            "compare mit apache gpl open source licenses and obligations",
            "https://choosealicense.com/licenses/"
        ));
    }

    #[test]
    fn generic_disease_overview_rejected_for_side_effect_comparison() {
        assert!(generic_disease_overview(
            "blood pressure medication side effects comparison",
            "hypertension fact sheet overview symptoms prevention treatment and risk factors",
            "https://www.who.int/news-room/fact-sheets/detail/hypertension"
        ));
        assert!(!generic_disease_overview(
            "blood pressure medication side effects comparison",
            "compare side effects of ace inhibitors beta blockers and diuretics",
            "https://www.rxlist.com/high_blood_pressure_hypertension_medications/drugs-condition.htm"
        ));
    }

    #[test]
    fn docs_focused_query_adds_docs_anchor_once() {
        assert_eq!(
            docs_focused_query("docker compose networking between containers"),
            "docker compose networking between containers docs"
        );
        assert_eq!(
            docs_focused_query("docker compose networking docs"),
            "docker compose networking docs"
        );
        assert_eq!(
            docs_focused_query(
                "step by step guide to deploying a machine learning model to production using Docker Kubernetes and CI CD pipelines"
            ),
            "deploying a machine learning model to production using docker kubernetes and ci cd pipelines docs"
        );
    }

    #[test]
    fn strict_health_query_detection() {
        assert!(is_strict_health_query(
            "blood pressure medication side effects comparison"
        ));
        assert!(!is_strict_health_query(
            "sleep apnea causes diagnosis treatment options"
        ));
        assert!(!is_strict_health_query(
            "COVID long haulers symptoms treatment"
        ));
        assert!(!is_strict_health_query(
            "mRNA vaccine technology how it works"
        ));
        assert!(!is_strict_health_query("what is quantum computing"));
    }

    #[test]
    fn health_explainer_query_detection() {
        assert!(is_health_explainer_query(
            "mRNA vaccine technology how it works"
        ));
        assert!(is_health_explainer_query("what is insulin resistance"));
        assert!(!is_health_explainer_query(
            "blood pressure medication side effects comparison"
        ));
    }

    #[test]
    fn community_fallback_blocked_for_multilingual_queries() {
        let result = ResultItem {
            title: "What is Kubernetes?".into(),
            url: "https://stegosaurusdormant.com/kubernetes-ingresses/".into(),
            snippet: "HN story by example".into(),
            rank: 1,
            backend: BackendId::HackerNews,
            score: Some(0.4),
            published_date: None,
        };
        assert!(community_fallback_blocked(
            rank::fusion::QueryIntent::Informational,
            false,
            false,
            true,
            &result,
        ));
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
        let mut fetchium_config = crate::config::FetchiumConfig::default();
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
            Duration::from_secs(5)
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
            Duration::from_millis(4500)
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

    #[derive(Clone)]
    struct TestBackend {
        id: BackendId,
        delay: Duration,
        results: Vec<ResultItem>,
        cancelled: Arc<AtomicBool>,
    }

    #[async_trait]
    impl SearchBackend for TestBackend {
        fn id(&self) -> BackendId {
            self.id.clone()
        }

        async fn search(&self, _query: &str, _max_results: u32) -> FetchiumResult<Vec<ResultItem>> {
            struct DropGuard(Arc<AtomicBool>);
            impl Drop for DropGuard {
                fn drop(&mut self) {
                    self.0.store(true, Ordering::SeqCst);
                }
            }

            let guard = DropGuard(Arc::clone(&self.cancelled));
            tokio::time::sleep(self.delay).await;
            std::mem::forget(guard);
            Ok(self.results.clone())
        }
    }

    #[tokio::test]
    async fn early_return_aborts_pending_backend_tasks() {
        let fast = TestBackend {
            id: BackendId::Tavily,
            delay: Duration::from_millis(5),
            results: vec![ResultItem {
                title: "Fast".into(),
                url: "https://example.com/fast".into(),
                snippet: "fast result".into(),
                rank: 1,
                backend: BackendId::Tavily,
                score: Some(1.0),
                published_date: None,
            }],
            cancelled: Arc::new(AtomicBool::new(false)),
        };
        let slow_cancelled = Arc::new(AtomicBool::new(false));
        let slow = TestBackend {
            id: BackendId::Searxng,
            delay: Duration::from_secs(5),
            results: vec![ResultItem {
                title: "Slow".into(),
                url: "https://example.com/slow".into(),
                snippet: "slow result".into(),
                rank: 1,
                backend: BackendId::Searxng,
                score: Some(0.4),
                published_date: None,
            }],
            cancelled: Arc::clone(&slow_cancelled),
        };

        let orchestrator = SearchOrchestrator {
            backends: vec![Arc::new(fast), Arc::new(slow)],
            config: OrchestratorConfig {
                max_results_per_backend: 5,
                max_total_results: 1,
                backend_timeout: Duration::from_secs(30),
                enabled_backends: vec![BackendId::Tavily, BackendId::Searxng],
                simhash_threshold: 6,
                freshness_need: 0.5,
                use_hyperfusion: true,
                tavily_api_keys: Vec::new(),
                serper_api_keys: Vec::new(),
                exa_api_keys: Vec::new(),
                firecrawl_api_keys: Vec::new(),
            },
            weight_overrides: HashMap::new(),
            circuit_breaker: CircuitBreaker::new(),
            bulkhead: Bulkhead::new(),
            metrics: PipelineMetrics::new(),
            in_flight: Arc::new(AsyncMutex::new(HashMap::new())),
            backend_selector: AdaptiveBackendSelector::default(),
        };

        let started = std::time::Instant::now();
        let _results = orchestrator
            .search("polyandry", Some(1))
            .await
            .expect("search");
        assert!(started.elapsed() < Duration::from_secs(1));

        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(slow_cancelled.load(Ordering::SeqCst));
    }
}
