//! Shared test utilities for HyperSearchX unit and integration tests.
//!
//! Re-exported from `lib.rs` behind `#[cfg(test)] pub mod test_utils`.

use crate::types::{BackendId, ResultItem, SegmentType, Segment};

/// Build a synthetic `ResultItem` for testing rank and dedup logic.
pub fn make_result_item(title: &str, url: &str, snippet: &str) -> ResultItem {
    ResultItem {
        title: title.to_string(),
        url: url.to_string(),
        snippet: snippet.to_string(),
        rank: 0,
        backend: BackendId::DuckDuckGo,
        score: None,
        published_date: None,
    }
}

/// Build a synthetic `ResultItem` with a specific rank and backend.
pub fn make_result_item_ranked(
    title: &str,
    url: &str,
    snippet: &str,
    rank: u32,
    backend: BackendId,
) -> ResultItem {
    ResultItem {
        title: title.to_string(),
        url: url.to_string(),
        snippet: snippet.to_string(),
        rank,
        backend,
        score: None,
        published_date: None,
    }
}

/// Build a synthetic `Segment` for testing QATBE, SCS, PDS.
pub fn make_segment(text: &str, seg_type: SegmentType, relevance: f64) -> Segment {
    let tokens = (text.len() as f64 / 4.0).ceil() as u32;
    Segment {
        seg_type,
        relevance,
        tokens,
        content: serde_json::Value::String(text.to_string()),
        source_ref: None,
    }
}

/// Load a test HTML fixture from `tests/fixtures/` at the workspace root.
///
/// # Panics
/// Panics if the fixture file cannot be read (developer error).
pub fn load_fixture(name: &str) -> String {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to load fixture '{name}': {e} (path: {path:?})"))
}

/// Build N synthetic result items for stress-testing ranking.
pub fn make_result_items(count: usize) -> Vec<ResultItem> {
    (0..count)
        .map(|i| ResultItem {
            title: format!("Result {i}: Understanding Rust Concepts"),
            url: format!("https://example.com/page-{i}"),
            snippet: format!("This result covers Rust topic {i} with details about ownership and borrowing."),
            rank: i as u32,
            backend: if i % 2 == 0 { BackendId::DuckDuckGo } else { BackendId::Google },
            score: None,
            published_date: None,
        })
        .collect()
}
