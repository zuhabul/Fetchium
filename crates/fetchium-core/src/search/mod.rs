//! Search backends and orchestrator (PRD §15).
//!
//! Phase 1: DuckDuckGo HTML scraper.
//! Phase 2: 10 additional backends (HTTP + headless Chromium).
//!
//! ## HTTP backends (no feature flag required):
//! - SearXNG (self-hosted localhost:4040 — aggregates 9+ engines), DuckDuckGo, Wikipedia,
//!   Brave, HackerNews, ArXiv, GitHub, Reddit, StackOverflow
//!
//! ## Headless backends (`--features headless`):
//! - Google, Bing, Google Scholar
//!
//! ## SearXNG Self-Hosted (Primary — free, unlimited, CAPTCHA-free)
//! Run `cd ~/searxng-local && docker compose up -d` to start the local aggregator.
//! Set `SEARXNG_URL=***REMOVED***` to use exclusively.

// HTTP backends (always available)
pub mod arxiv;
pub mod backend_selector;
pub mod brave;
pub mod dedup;
pub mod duckduckgo;
pub mod exa;
pub mod fallback;
pub mod firecrawl;
pub mod github;
pub mod hackernews;
pub mod latency;
pub mod orchestrator;
pub mod reddit;
pub mod searxng;
pub mod serper;
pub mod stackoverflow;
pub mod tavily;
pub mod wikipedia;

pub use backend_selector::{AbsConfig, AdaptiveBackendSelector, BackendSelection};
pub use latency::{LatencyConfig, LatencyPredictor, LatencyStats};

// Headless backends (compiled unconditionally; return empty results without `headless` feature)
pub mod bing;
pub mod google;
pub mod scholar;

use crate::error::HsxResult;
use crate::types::{BackendId, ResultItem};
use async_trait::async_trait;

/// Trait implemented by every search backend.
///
/// Backends are responsible for:
/// 1. Querying their respective search endpoint
/// 2. Parsing results into the unified `ResultItem` schema
/// 3. Reporting their ID and headless requirements
#[async_trait]
pub trait SearchBackend: Send + Sync {
    /// Unique identifier for this backend.
    fn id(&self) -> BackendId;

    /// Whether this backend requires a headless Chromium browser.
    /// HTTP-only backends return `false` (default).
    fn requires_headless(&self) -> bool {
        false
    }

    /// Execute a search and return at most `max_results` results.
    ///
    /// Implementations should:
    /// - Return partial results on soft failures (not full errors)
    /// - Return `Err` only for hard failures (network down, auth broken, etc.)
    /// - Never panic
    async fn search(&self, query: &str, max_results: u32) -> HsxResult<Vec<ResultItem>>;
}
