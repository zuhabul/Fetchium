//! Search backends and orchestrator (PRD §15).

use async_trait::async_trait;
use crate::error::HsxResult;
use crate::types::{BackendId, ResultItem};

/// Trait for search backends.
#[async_trait]
pub trait SearchBackend: Send + Sync {
    /// Backend identifier.
    fn id(&self) -> BackendId;

    /// Whether this backend requires a headless browser.
    fn requires_headless(&self) -> bool {
        false
    }

    /// Execute a search query and return results.
    async fn search(&self, query: &str, max_results: u32) -> HsxResult<Vec<ResultItem>>;
}
