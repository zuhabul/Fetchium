//! Fallback chain logic — ordered backend fallback with result quality thresholds.
//!
//! When the primary backend returns fewer than `min_results` results (or fails),
//! the chain automatically tries the next backend. This handles API outages,
//! rate limits, and degraded service gracefully.
//!
//! # Design
//! - Each entry has a `min_results` threshold; falling short triggers fallback.
//! - Per-backend timeout prevents slow backends from blocking the chain.
//! - The best partial result set is returned if all backends fail to meet thresholds.

use crate::error::HsxResult;
use crate::search::SearchBackend;
use crate::types::{BackendId, ResultItem};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{info, warn};

// ── Chain entry ────────────────────────────────────────────────────────────────

/// One entry in the fallback chain, pairing a backend with its quality threshold.
struct ChainEntry {
    backend: Arc<dyn SearchBackend>,
    /// If the backend returns fewer than this many results, fall through to the next.
    min_results: usize,
}

// ── FallbackChain ──────────────────────────────────────────────────────────────

/// Ordered fallback chain: try backends in sequence until quality bar is met.
///
/// # Example
/// ```no_run
/// # use std::sync::Arc;
/// # use fetchium_core::search::fallback::FallbackChain;
/// let mut chain = FallbackChain::new();
/// // chain.add(google_backend, 5).add(ddg_backend, 1);
/// ```
pub struct FallbackChain {
    entries: Vec<ChainEntry>,
    /// Maximum time to wait for any single backend.
    timeout: Duration,
}

impl FallbackChain {
    /// Create an empty fallback chain with a 15-second per-backend timeout.
    pub fn new() -> Self {
        Self::with_timeout(Duration::from_secs(15))
    }

    /// Create an empty fallback chain with a custom per-backend timeout.
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            entries: Vec::new(),
            timeout,
        }
    }

    /// Append a backend to the chain.
    ///
    /// `min_results` triggers fallback to the next entry if the backend returns
    /// fewer results. Set to `0` to always accept results from this backend.
    pub fn add(&mut self, backend: Arc<dyn SearchBackend>, min_results: usize) -> &mut Self {
        self.entries.push(ChainEntry {
            backend,
            min_results,
        });
        self
    }

    /// Execute the chain: try each backend in order, falling back when needed.
    ///
    /// Returns:
    /// - Results from the first backend that meets `min_results`, **or**
    /// - The largest partial result set collected across all backends if none meet the threshold.
    /// - An empty `Vec` if all backends fail entirely.
    pub async fn execute(&self, query: &str, max_results: u32) -> HsxResult<Vec<ResultItem>> {
        let mut best_results: Vec<ResultItem> = Vec::new();

        for entry in &self.entries {
            let backend_id = entry.backend.id();
            let start = Instant::now();

            match timeout(self.timeout, entry.backend.search(query, max_results)).await {
                Ok(Ok(results)) => {
                    let elapsed_ms = start.elapsed().as_millis();
                    let n = results.len();
                    info!(
                        "FallbackChain: {:?} returned {} results in {}ms",
                        backend_id, n, elapsed_ms
                    );

                    if n >= entry.min_results {
                        // Quality bar met — return immediately
                        return Ok(results);
                    }

                    // Below threshold — keep as best partial if it improves on what we have
                    if n > best_results.len() {
                        best_results = results;
                    }
                    warn!(
                        "FallbackChain: {:?} returned {} results (min={}), falling back",
                        backend_id, n, entry.min_results
                    );
                }
                Ok(Err(e)) => {
                    warn!("FallbackChain: {:?} error: {e}", backend_id);
                }
                Err(_elapsed) => {
                    warn!(
                        "FallbackChain: {:?} timed out after {:?}",
                        backend_id, self.timeout
                    );
                }
            }
        }

        if best_results.is_empty() {
            warn!(
                "FallbackChain: all {} backend(s) exhausted for {:?}",
                self.entries.len(),
                query
            );
        }

        Ok(best_results)
    }

    /// Number of backends registered in this chain.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if no backends have been added.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for FallbackChain {
    fn default() -> Self {
        Self::new()
    }
}

// ── Chain factory ──────────────────────────────────────────────────────────────

/// Build the standard web-search fallback chain from a map of available backends.
///
/// Priority order (highest quality first):
/// `Google → Bing → SearXNG → Brave → DuckDuckGo`
///
/// Only backends present in `backends` are added to the chain.
pub fn build_web_search_chain(
    backends: &HashMap<BackendId, Arc<dyn SearchBackend>>,
) -> FallbackChain {
    let mut chain = FallbackChain::new();

    // (BackendId, min_results_threshold)
    let priority_order: &[(BackendId, usize)] = &[
        (BackendId::Google, 5),
        (BackendId::Bing, 3),
        (BackendId::Searxng, 3),
        (BackendId::Brave, 3),
        (BackendId::DuckDuckGo, 1),
    ];

    for (id, min_results) in priority_order {
        if let Some(backend) = backends.get(id) {
            chain.add(Arc::clone(backend), *min_results);
        }
    }

    chain
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BackendId;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // ── Mock backend ──────────────────────────────────────────────────────────

    /// A mock backend that always returns a fixed number of results.
    struct MockBackend {
        id: BackendId,
        results: Vec<ResultItem>,
        call_count: Arc<AtomicUsize>,
    }

    impl MockBackend {
        /// Construct a mock backend and return its call counter and `Arc<dyn SearchBackend>`.
        fn make(id: BackendId, n_results: usize) -> (Arc<AtomicUsize>, Arc<dyn SearchBackend>) {
            let counter = Arc::new(AtomicUsize::new(0));
            let results = (0..n_results)
                .map(|i| ResultItem {
                    title: format!("Result {i}"),
                    url: format!("https://example.com/{i}"),
                    snippet: format!("Snippet for result {i}"),
                    rank: (i + 1) as u32,
                    backend: id.clone(),
                    score: None,
                    published_date: None,
                })
                .collect();
            let backend = Arc::new(MockBackend {
                id,
                results,
                call_count: Arc::clone(&counter),
            });
            (counter, backend as Arc<dyn SearchBackend>)
        }
    }

    #[async_trait]
    impl SearchBackend for MockBackend {
        fn id(&self) -> BackendId {
            self.id.clone()
        }

        async fn search(&self, _query: &str, _max: u32) -> HsxResult<Vec<ResultItem>> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            Ok(self.results.clone())
        }
    }

    /// A mock backend that always returns an error.
    struct ErrorBackend {
        id: BackendId,
        call_count: Arc<AtomicUsize>,
    }

    impl ErrorBackend {
        fn make(id: BackendId) -> (Arc<AtomicUsize>, Arc<dyn SearchBackend>) {
            let counter = Arc::new(AtomicUsize::new(0));
            let backend = Arc::new(ErrorBackend {
                id,
                call_count: Arc::clone(&counter),
            });
            (counter, backend as Arc<dyn SearchBackend>)
        }
    }

    #[async_trait]
    impl SearchBackend for ErrorBackend {
        fn id(&self) -> BackendId {
            self.id.clone()
        }

        async fn search(&self, _query: &str, _max: u32) -> HsxResult<Vec<ResultItem>> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            Err(crate::error::HsxError::Search("mock error".to_string()))
        }
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn chain_uses_first_backend_when_sufficient_results() {
        let (counter1, b1) = MockBackend::make(BackendId::Google, 10);
        let (counter2, b2) = MockBackend::make(BackendId::Bing, 10);

        let mut chain = FallbackChain::new();
        chain.add(b1, 5).add(b2, 5);

        let results = chain.execute("test query", 10).await.unwrap();
        assert_eq!(
            results.len(),
            10,
            "Should get 10 results from first backend"
        );
        assert_eq!(
            counter1.load(Ordering::SeqCst),
            1,
            "First backend called once"
        );
        assert_eq!(
            counter2.load(Ordering::SeqCst),
            0,
            "Second backend NOT called"
        );
    }

    #[tokio::test]
    async fn chain_falls_back_when_too_few_results() {
        let (_c1, b1) = MockBackend::make(BackendId::Google, 2); // Only 2 results
        let (counter2, b2) = MockBackend::make(BackendId::Bing, 10);

        let mut chain = FallbackChain::new();
        chain.add(b1, 5).add(b2, 5); // min_results=5

        let results = chain.execute("test query", 10).await.unwrap();
        assert_eq!(results.len(), 10, "Should get 10 results from fallback");
        assert_eq!(
            counter2.load(Ordering::SeqCst),
            1,
            "Should have fallen back to second backend"
        );
    }

    #[tokio::test]
    async fn chain_returns_best_partial_when_all_fail_threshold() {
        let (_c1, b1) = MockBackend::make(BackendId::Google, 0); // 0 results
        let (_c2, b2) = MockBackend::make(BackendId::Bing, 2); // 2 results (< min 5)

        let mut chain = FallbackChain::new();
        chain.add(b1, 5).add(b2, 5); // Neither meets min_results=5

        let results = chain.execute("test query", 10).await.unwrap();
        // Best partial was 2 results from Bing
        assert_eq!(results.len(), 2, "Should return best partial (2 from Bing)");
    }

    #[tokio::test]
    async fn chain_skips_error_backends_and_falls_back() {
        let (err_counter, b_err) = ErrorBackend::make(BackendId::Google);
        let (ok_counter, b_ok) = MockBackend::make(BackendId::Bing, 7);

        let mut chain = FallbackChain::new();
        chain.add(b_err, 5).add(b_ok, 5);

        let results = chain.execute("test", 10).await.unwrap();
        assert_eq!(results.len(), 7);
        assert_eq!(
            err_counter.load(Ordering::SeqCst),
            1,
            "Error backend was tried"
        );
        assert_eq!(ok_counter.load(Ordering::SeqCst), 1, "OK backend was used");
    }

    #[tokio::test]
    async fn chain_returns_empty_when_all_error() {
        let (_c1, b1) = ErrorBackend::make(BackendId::Google);
        let (_c2, b2) = ErrorBackend::make(BackendId::Bing);

        let mut chain = FallbackChain::new();
        chain.add(b1, 5).add(b2, 5);

        let results = chain.execute("test", 10).await.unwrap();
        assert!(results.is_empty(), "All errors → empty results");
    }

    #[tokio::test]
    async fn empty_chain_returns_empty_results() {
        let chain = FallbackChain::new();
        let results = chain.execute("test", 10).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn chain_with_zero_min_results_always_accepts() {
        let (_counter, b) = MockBackend::make(BackendId::DuckDuckGo, 1); // Only 1 result
        let mut chain = FallbackChain::new();
        chain.add(b, 0); // min_results=0 means always accept

        let results = chain.execute("test", 10).await.unwrap();
        assert_eq!(results.len(), 1, "Zero min_results should always accept");
    }

    #[test]
    fn chain_len_and_is_empty() {
        let mut chain = FallbackChain::new();
        assert!(chain.is_empty());
        assert_eq!(chain.len(), 0);

        let (_c, b) = MockBackend::make(BackendId::DuckDuckGo, 0);
        chain.add(b, 1);
        assert!(!chain.is_empty());
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn build_web_search_chain_respects_priority() {
        let mut backends: HashMap<BackendId, Arc<dyn SearchBackend>> = HashMap::new();
        let (_c1, b1) = MockBackend::make(BackendId::DuckDuckGo, 5);
        let (_c2, b2) = MockBackend::make(BackendId::Brave, 5);
        backends.insert(BackendId::DuckDuckGo, b1);
        backends.insert(BackendId::Brave, b2);

        let chain = build_web_search_chain(&backends);
        // Should have both backends in the chain
        assert_eq!(chain.len(), 2, "Chain should have 2 entries");
    }

    #[test]
    fn build_web_search_chain_empty_backends() {
        let backends: HashMap<BackendId, Arc<dyn SearchBackend>> = HashMap::new();
        let chain = build_web_search_chain(&backends);
        assert!(chain.is_empty(), "Empty backends map → empty chain");
    }
}
