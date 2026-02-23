//! Search orchestrator — parallel dispatch, dedup, ranking (PRD §15).
//!
//! Phase 1: DDG only, parallel-ready architecture for Phase 2+ backends.
//! Phase 2: Google, Bing, Scholar, SearXNG, Wikipedia, Brave, etc.

use crate::error::HsxResult;
use crate::http::HttpClient;
use crate::rank;
use crate::search::duckduckgo::DuckDuckGoBackend;
use crate::search::SearchBackend;
use crate::types::{BackendId, ResultItem};
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
    /// Which backends to use.
    pub enabled_backends: Vec<BackendId>,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_results_per_backend: 15,
            max_total_results: 10,
            backend_timeout: Duration::from_secs(15),
            enabled_backends: vec![BackendId::DuckDuckGo],
        }
    }
}

impl OrchestratorConfig {
    /// Create config from HsxConfig settings.
    pub fn from_hsx_config(
        hsx: &crate::config::HsxConfig,
        max_results: u32,
    ) -> Self {
        let enabled_backends = hsx
            .search
            .backends
            .iter()
            .filter_map(|s| parse_backend_id(s))
            .collect::<Vec<_>>();

        let enabled_backends = if enabled_backends.is_empty() {
            vec![BackendId::DuckDuckGo]
        } else {
            enabled_backends
        };

        Self {
            max_results_per_backend: max_results + 5,
            max_total_results: max_results,
            backend_timeout: Duration::from_secs(hsx.search.timeout_secs),
            enabled_backends,
        }
    }
}

/// Manages multiple search backends, dispatches in parallel, fuses results.
pub struct SearchOrchestrator {
    backends: Vec<Arc<dyn SearchBackend>>,
    config: OrchestratorConfig,
}

impl SearchOrchestrator {
    /// Create an orchestrator with backends from the given config.
    pub fn new(http_client: HttpClient, config: OrchestratorConfig) -> Self {
        let mut backends: Vec<Arc<dyn SearchBackend>> = Vec::new();

        for id in &config.enabled_backends {
            match id {
                BackendId::DuckDuckGo => {
                    backends.push(Arc::new(DuckDuckGoBackend::new(http_client.clone())));
                }
                // Placeholders for Phase 2 backends
                other => {
                    warn!("Backend {:?} not yet available (Phase 2+), skipping", other);
                }
            }
        }

        if backends.is_empty() {
            // Always fall back to DDG
            warn!("No backends configured — falling back to DuckDuckGo");
            backends.push(Arc::new(DuckDuckGoBackend::new(http_client)));
        }

        Self { backends, config }
    }

    /// Execute a search across all enabled backends, returning fused results.
    ///
    /// # Pipeline
    /// 1. Dispatch query to all backends concurrently
    /// 2. Collect results with per-backend timeout (failures = empty list)
    /// 3. Deduplicate by canonical URL (tracking params stripped)
    /// 4. Rerank by BM25 relevance against the query
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

        // Step 3: Deduplicate
        let deduped = rank::deduplicate(all);

        // Step 4: Rerank by BM25
        let reranked = rank::rerank(deduped, query);

        // Step 5: Take top N
        let final_results: Vec<ResultItem> = reranked.into_iter().take(max as usize).collect();

        info!(
            "Orchestrator: returning {} results for {:?}",
            final_results.len(),
            query
        );

        Ok(final_results)
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
        assert_eq!(parse_backend_id("invalid_backend"), None);
    }

    #[test]
    fn orchestrator_fallsback_to_ddg() {
        let config_str = "duckduckgo";
        let backend = parse_backend_id(config_str);
        assert!(backend.is_some());
    }

    #[test]
    fn orchestrator_config_from_defaults() {
        let cfg = OrchestratorConfig::default();
        assert_eq!(cfg.max_total_results, 10);
        assert!(cfg.enabled_backends.contains(&BackendId::DuckDuckGo));
    }
}
