//! Output formatters — markdown, JSON, CSV, YAML, HTML, segments (PRD §26).
//!
//! Phase 1: Markdown, JSON, plain text, and segments formatters.
//! Phase 5+: CSV, YAML, HTML, PDF, DOCX formatters.

use crate::types::{AgentSearchResult, ResultItem, SearchResult, Segment};
use serde_json::Value;

// ─── Search Result Formatters ────────────────────────────────────

/// Format search results as human-readable Markdown.
///
/// Produces a clean, terminal-friendly document with:
/// - Header with query and result count
/// - Numbered results with title, URL, and snippet
/// - Metadata footer
pub fn format_search_markdown(result: &SearchResult) -> String {
    let mut out = String::new();

    // Header
    out.push_str(&format!("# Search Results: {}\n\n", result.meta.query));
    out.push_str(&format!(
        "_Found {} results in {}ms_\n\n",
        result.items.len(),
        result.meta.duration_ms
    ));
    out.push_str("---\n\n");

    if result.items.is_empty() {
        out.push_str("No results found.\n");
        return out;
    }

    for item in &result.items {
        out.push_str(&format!("## {}. {}\n", item.rank, item.title));
        out.push_str(&format!("**URL:** <{}>\n\n", item.url));
        if !item.snippet.is_empty() {
            out.push_str(&format!("{}\n\n", item.snippet));
        }
        if let Some(date) = &item.published_date {
            out.push_str(&format!("_Published: {date}_\n\n"));
        }
        out.push_str("---\n\n");
    }

    out
}

/// Format search results as compact, terminal-friendly plain text.
pub fn format_search_text(result: &SearchResult) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "Results for: {} ({} found, {}ms)\n\n",
        result.meta.query,
        result.items.len(),
        result.meta.duration_ms,
    ));

    for item in &result.items {
        out.push_str(&format!("{}. {}\n", item.rank, item.title));
        out.push_str(&format!("   {}\n", item.url));
        if !item.snippet.is_empty() {
            let snippet = wrap_text(&item.snippet, 72);
            for line in snippet.lines() {
                out.push_str(&format!("   {line}\n"));
            }
        }
        out.push('\n');
    }

    out
}

/// Format search results as a JSON string (pretty-printed).
pub fn format_search_json(result: &SearchResult) -> String {
    serde_json::to_string_pretty(result)
        .unwrap_or_else(|e| format!("{{\"error\": \"Serialization failed: {e}\"}}"))
}

// ─── Fetch / Content Formatters ──────────────────────────────────

/// Format extracted content as Markdown.
///
/// Produces a clean document with title, metadata, and content.
pub fn format_content_markdown(
    title: &str,
    url: &str,
    content: &str,
    tokens: u32,
    metadata: Option<&str>,
) -> String {
    let mut out = String::new();

    if !title.is_empty() {
        out.push_str(&format!("# {title}\n\n"));
    }

    out.push_str(&format!("**Source:** <{url}>\n"));
    out.push_str(&format!("**Tokens:** ~{tokens}\n"));

    if let Some(meta) = metadata {
        if !meta.is_empty() {
            out.push_str(&format!("**Info:** {meta}\n"));
        }
    }

    out.push_str("\n---\n\n");
    out.push_str(content);
    out.push('\n');

    out
}

/// Format extracted content as plain text.
pub fn format_content_text(title: &str, url: &str, content: &str) -> String {
    let mut out = String::new();
    if !title.is_empty() {
        out.push_str(&format!("{title}\n"));
        out.push_str(&"=".repeat(title.len().min(60)));
        out.push('\n');
    }
    out.push_str(&format!("Source: {url}\n\n"));
    out.push_str(content);
    out.push('\n');
    out
}

// ─── Agent Output Formatters ─────────────────────────────────────

/// Format an AgentSearchResult as compact JSON for AI consumption.
///
/// This is the primary output format for `agent-search` and `agent-fetch`.
/// Every field is included — the agent decides what to use.
pub fn format_agent_json(result: &AgentSearchResult) -> String {
    serde_json::to_string_pretty(result)
        .unwrap_or_else(|e| format!("{{\"error\": \"Serialization failed: {e}\"}}"))
}

/// Format segments as a JSON array optimized for AI consumption.
///
/// Each segment includes: type, content, relevance score, and token count.
/// This is the most token-efficient format for passing to an LLM.
pub fn format_segments_json(segments: &[Segment]) -> String {
    let json_segments: Vec<Value> = segments
        .iter()
        .map(|seg| {
            serde_json::json!({
                "type": seg.seg_type,
                "tokens": seg.tokens,
                "relevance": (seg.relevance * 100.0).round() / 100.0,
                "content": seg.content,
            })
        })
        .collect();

    serde_json::to_string_pretty(&json_segments).unwrap_or_else(|_| "[]".to_string())
}

/// Format a list of ResultItems as a markdown table for display.
pub fn format_results_table(items: &[ResultItem]) -> String {
    if items.is_empty() {
        return "No results.\n".to_string();
    }

    let mut out = String::new();
    out.push_str("| # | Title | URL |\n");
    out.push_str("|---|-------|-----|\n");

    for item in items {
        let title = truncate_str(&item.title, 60);
        let url = truncate_str(&item.url, 60);
        out.push_str(&format!("| {} | {} | {} |\n", item.rank, title, url));
    }

    out
}

// ─── Helpers ─────────────────────────────────────────────────────

/// Wrap text at word boundaries to the given column width.
pub fn wrap_text(text: &str, width: usize) -> String {
    let mut result = String::with_capacity(text.len());
    let mut line_len = 0usize;

    for word in text.split_whitespace() {
        let word_len = word.len();
        if line_len > 0 && line_len + 1 + word_len > width {
            result.push('\n');
            line_len = 0;
        }
        if line_len > 0 {
            result.push(' ');
            line_len += 1;
        }
        result.push_str(word);
        line_len += word_len;
    }

    result
}

/// Truncate a string to max_len chars, appending "..." if truncated.
///
/// Uses char-boundary-safe slicing to avoid panics on multi-byte characters.
pub fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    let target = max_len.saturating_sub(3);
    let safe_end = safe_byte_index(s, target);
    let truncated = &s[..safe_end];
    match truncated.rfind(' ') {
        Some(pos) if pos > safe_end / 2 => format!("{}...", &truncated[..pos]),
        _ => format!("{truncated}..."),
    }
}

/// Find the last valid UTF-8 char boundary at or before `target` byte index.
fn safe_byte_index(s: &str, target: usize) -> usize {
    if target >= s.len() {
        return s.len();
    }
    // Walk backwards from target to find a char boundary
    let mut idx = target;
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{BackendId, PdsTier, ResourceTier, SearchMeta, SearchMode};
    use std::collections::HashMap;

    fn make_search_result(query: &str, items: Vec<ResultItem>) -> SearchResult {
        SearchResult {
            meta: SearchMeta {
                query: query.into(),
                mode: SearchMode::Search,
                tier: PdsTier::Summary,
                tokens_used: 500,
                tokens_budget: 4000,
                sources_fetched: items.len() as u32,
                sources_validated: items.len() as u32,
                validation_pass_rate: 1.0,
                duration_ms: 1200,
                resource_tier: ResourceTier::Standard,
                timestamp: "2026-02-23T12:00:00Z".into(),
                result_id: "test-001".into(),
                content_hashes: HashMap::new(),
            },
            items,
        }
    }

    fn make_item(title: &str, url: &str, snippet: &str, rank: u32) -> ResultItem {
        ResultItem {
            title: title.into(),
            url: url.into(),
            snippet: snippet.into(),
            rank,
            backend: BackendId::DuckDuckGo,
            score: Some(0.8),
            published_date: None,
        }
    }

    #[test]
    fn markdown_contains_query() {
        let result = make_search_result("rust async", vec![]);
        let md = format_search_markdown(&result);
        assert!(md.contains("rust async"));
        assert!(md.contains("No results found"));
    }

    #[test]
    fn markdown_includes_all_results() {
        let items = vec![
            make_item("Rust Book", "https://doc.rust-lang.org", "Learn Rust", 1),
            make_item("Tokio Docs", "https://tokio.rs", "Async runtime", 2),
        ];
        let result = make_search_result("rust async", items);
        let md = format_search_markdown(&result);
        assert!(md.contains("Rust Book"));
        assert!(md.contains("Tokio Docs"));
        assert!(md.contains("doc.rust-lang.org"));
    }

    #[test]
    fn json_is_valid() {
        let items = vec![make_item("Test", "https://test.com", "snippet", 1)];
        let result = make_search_result("test query", items);
        let json = format_search_json(&result);
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(parsed["meta"]["query"], "test query");
    }

    #[test]
    fn text_wrapping() {
        let text = "This is a long sentence that should be wrapped at the given column width for readability.";
        let wrapped = wrap_text(text, 40);
        for line in wrapped.lines() {
            assert!(line.len() <= 45); // small slack for long words
        }
    }

    #[test]
    fn truncate_str_long() {
        let s = "Hello world this is a very long string that exceeds the limit";
        let t = truncate_str(s, 20);
        assert!(t.len() <= 23);
        assert!(t.ends_with("..."));
    }

    #[test]
    fn truncate_str_short() {
        let s = "Short";
        assert_eq!(truncate_str(s, 20), "Short");
    }

    #[test]
    fn segments_json_format() {
        let segs = vec![Segment {
            seg_type: crate::types::SegmentType::Paragraph,
            relevance: 0.85,
            tokens: 42,
            content: serde_json::Value::String("Test content".into()),
            source_ref: Some(0),
        }];
        let json = format_segments_json(&segs);
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert!(parsed.is_array());
        assert_eq!(parsed[0]["tokens"], 42);
    }
}
