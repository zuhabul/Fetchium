//! Search backends and orchestrator (PRD §15).
//!
//! Phase 1: DuckDuckGo HTML scraper backend + search orchestrator.
//! Phase 2: Google, Bing, Scholar, SearXNG, Wikipedia, Brave, HN, ArXiv, GitHub, Reddit, SO.

pub mod duckduckgo;
pub mod orchestrator;

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
    /// Phase 1 backends are all HTTP-only (no headless required).
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
