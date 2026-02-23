//! Semantic Content Segmentation (SCS).
//!
//! PRD SS8.4: Segment content into typed blocks, each in its most
//! token-efficient representation. Key savings:
//! - Tables: JSON arrays instead of markdown tables (60% fewer tokens)
//! - Lists: JSON arrays instead of markdown bullets (30% fewer tokens)
//! - Facts: Structured claim+confidence (40% fewer tokens)
//! - Data: key:value pairs instead of prose (50% fewer tokens)

use crate::extract::ExtractedContent;
use crate::token::counter::count_tokens;
use crate::types::{Segment, SegmentType};
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{json, Value};
use tracing::debug;

/// Result of SCS processing.
#[derive(Debug, Clone)]
pub struct ScsResult {
    /// Typed segments in document order.
    pub segments: Vec<Segment>,
    /// Total tokens across all segments.
    pub total_tokens: u32,
    /// Token savings compared to flat text.
    pub tokens_saved: u32,
}

static TABLE_ROW_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*\|(.+\|)+\s*$").expect("valid regex"));
static TABLE_SEPARATOR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*\|[\s\-:]+(\|[\s\-:]+)+\|?\s*$").expect("valid regex"));
static LIST_ITEM_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*[-*+]\s+(.+)$").expect("valid regex"));
static ORDERED_LIST_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*\d+[.)]\s+(.+)$").expect("valid regex"));
static CODE_FENCE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^```(\w*)\s*$").expect("valid regex"));
static HEADING_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(#{1,6})\s+(.+)$").expect("valid regex"));
static KV_PATTERN_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(.{3,40}):\s+(.+)$").expect("valid regex"));
static QUOTE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^>\s*(.+)$").expect("valid regex"));

/// Run SCS on extracted content, producing typed segments.
pub fn segment(content: &ExtractedContent) -> ScsResult {
    let text = &content.text;
    let original_tokens = content.tokens;

    let segments = parse_into_segments(text);

    let total_tokens: u32 = segments.iter().map(|s| s.tokens).sum();
    let tokens_saved = original_tokens.saturating_sub(total_tokens);

    debug!(
        "SCS: {} segments, {} tokens (saved {} from {})",
        segments.len(),
        total_tokens,
        tokens_saved,
        original_tokens
    );

    ScsResult {
        segments,
        total_tokens,
        tokens_saved,
    }
}

/// Parse text into typed segments.
fn parse_into_segments(text: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.is_empty() {
            i += 1;
            continue;
        }

        if let Some(caps) = CODE_FENCE_RE.captures(line) {
            let lang = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
            let (code_block, end_line) = collect_code_block(&lines, i + 1);
            if !code_block.is_empty() {
                let seg_content = json!({
                    "language": lang,
                    "code": code_block,
                });
                let tokens = count_tokens_json_value(&seg_content);
                segments.push(Segment {
                    seg_type: SegmentType::Code,
                    relevance: 0.0,
                    tokens,
                    content: seg_content,
                    source_ref: None,
                });
                i = end_line + 1;
                continue;
            }
        }

        if TABLE_ROW_RE.is_match(line) {
            let (table_seg, end_line) = parse_table(&lines, i);
            if let Some(seg) = table_seg {
                segments.push(seg);
                i = end_line + 1;
                continue;
            }
        }

        if let Some(caps) = HEADING_RE.captures(line) {
            let level = caps.get(1).map(|m| m.as_str().len()).unwrap_or(1);
            let text = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
            let seg_content = json!({
                "level": level,
                "text": text,
            });
            let tokens = count_tokens_json_value(&seg_content);
            segments.push(Segment {
                seg_type: SegmentType::Heading,
                relevance: 0.0,
                tokens,
                content: seg_content,
                source_ref: None,
            });
            i += 1;
            continue;
        }

        if QUOTE_RE.is_match(line) {
            let (quote_text, end_line) = collect_quotes(&lines, i);
            let seg_content = json!({
                "text": quote_text,
            });
            let tokens = count_tokens_json_value(&seg_content);
            segments.push(Segment {
                seg_type: SegmentType::Quote,
                relevance: 0.0,
                tokens,
                content: seg_content,
                source_ref: None,
            });
            i = end_line + 1;
            continue;
        }

        if LIST_ITEM_RE.is_match(line) {
            let (items, end_line) = collect_list_items(&lines, i, false);
            let seg_content = json!({
                "ordered": false,
                "items": items,
            });
            let tokens = count_tokens_json_value(&seg_content);
            segments.push(Segment {
                seg_type: SegmentType::List,
                relevance: 0.0,
                tokens,
                content: seg_content,
                source_ref: None,
            });
            i = end_line + 1;
            continue;
        }

        if ORDERED_LIST_RE.is_match(line) {
            let (items, end_line) = collect_list_items(&lines, i, true);
            let seg_content = json!({
                "ordered": true,
                "items": items,
            });
            let tokens = count_tokens_json_value(&seg_content);
            segments.push(Segment {
                seg_type: SegmentType::List,
                relevance: 0.0,
                tokens,
                content: seg_content,
                source_ref: None,
            });
            i = end_line + 1;
            continue;
        }

        if KV_PATTERN_RE.is_match(line) {
            let (kv_pairs, end_line) = collect_kv_pairs(&lines, i);
            if kv_pairs.len() >= 2 {
                let seg_content = json!(kv_pairs);
                let tokens = count_tokens_json_value(&seg_content);
                segments.push(Segment {
                    seg_type: SegmentType::Data,
                    relevance: 0.0,
                    tokens,
                    content: seg_content,
                    source_ref: None,
                });
                i = end_line + 1;
                continue;
            }
        }

        let (para_text, end_line) = collect_paragraph(&lines, i);
        if !para_text.is_empty() {
            let tokens = count_tokens(&para_text);
            segments.push(Segment {
                seg_type: SegmentType::Paragraph,
                relevance: 0.0,
                tokens,
                content: Value::String(para_text),
                source_ref: None,
            });
            i = end_line + 1;
        } else {
            i += 1;
        }
    }

    segments
}

fn count_tokens_json_value(val: &Value) -> u32 {
    let json_str = serde_json::to_string(val).unwrap_or_default();
    count_tokens(&json_str)
}

fn collect_code_block(lines: &[&str], start: usize) -> (String, usize) {
    let mut code = String::new();
    let mut i = start;
    while i < lines.len() {
        if lines[i].trim().starts_with("```") {
            return (code.trim_end().to_string(), i);
        }
        code.push_str(lines[i]);
        code.push('\n');
        i += 1;
    }
    (code.trim_end().to_string(), i.saturating_sub(1))
}

fn parse_table(lines: &[&str], start: usize) -> (Option<Segment>, usize) {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut i = start;

    while i < lines.len() && TABLE_ROW_RE.is_match(lines[i].trim()) {
        let line = lines[i].trim();
        if TABLE_SEPARATOR_RE.is_match(line) {
            i += 1;
            continue;
        }
        let cells: Vec<String> = line
            .split('|')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect();
        if !cells.is_empty() {
            rows.push(cells);
        }
        i += 1;
    }

    if rows.len() < 2 {
        return (None, start);
    }

    let headers = rows.remove(0);
    let seg_content = json!({
        "headers": headers,
        "rows": rows,
    });
    let tokens = count_tokens_json_value(&seg_content);

    let seg = Segment {
        seg_type: SegmentType::Table,
        relevance: 0.0,
        tokens,
        content: seg_content,
        source_ref: None,
    };

    (Some(seg), i.saturating_sub(1))
}

fn collect_quotes(lines: &[&str], start: usize) -> (String, usize) {
    let mut text = String::new();
    let mut i = start;
    while i < lines.len() {
        if let Some(caps) = QUOTE_RE.captures(lines[i].trim()) {
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(caps.get(1).map(|m| m.as_str()).unwrap_or(""));
            i += 1;
        } else {
            break;
        }
    }
    (text, i.saturating_sub(1))
}

fn collect_list_items(lines: &[&str], start: usize, ordered: bool) -> (Vec<String>, usize) {
    let mut items = Vec::new();
    let mut i = start;
    let pattern = if ordered {
        &*ORDERED_LIST_RE
    } else {
        &*LIST_ITEM_RE
    };

    while i < lines.len() {
        let trimmed = lines[i].trim();
        if let Some(caps) = pattern.captures(trimmed) {
            items.push(
                caps.get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default(),
            );
            i += 1;
        } else if trimmed.is_empty() {
            break;
        } else {
            break;
        }
    }
    (items, i.saturating_sub(1))
}

fn collect_kv_pairs(
    lines: &[&str],
    start: usize,
) -> (Vec<serde_json::Map<String, Value>>, usize) {
    let mut pairs = Vec::new();
    let mut i = start;

    while i < lines.len() {
        let trimmed = lines[i].trim();
        if let Some(caps) = KV_PATTERN_RE.captures(trimmed) {
            let key = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let val = caps
                .get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let mut map = serde_json::Map::new();
            map.insert("key".into(), Value::String(key));
            map.insert("value".into(), Value::String(val));
            pairs.push(map);
            i += 1;
        } else if trimmed.is_empty() {
            break;
        } else {
            break;
        }
    }
    (pairs, i.saturating_sub(1))
}

fn collect_paragraph(lines: &[&str], start: usize) -> (String, usize) {
    let mut text = String::new();
    let mut i = start;

    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.is_empty() {
            break;
        }
        if HEADING_RE.is_match(trimmed)
            || CODE_FENCE_RE.is_match(trimmed)
            || TABLE_ROW_RE.is_match(trimmed)
            || LIST_ITEM_RE.is_match(trimmed)
            || ORDERED_LIST_RE.is_match(trimmed)
        {
            break;
        }
        if !text.is_empty() {
            text.push(' ');
        }
        text.push_str(trimmed);
        i += 1;
    }

    (text, i.saturating_sub(1).max(start))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::ContentMetadata;
    use crate::types::CepLayer;

    fn make_content(text: &str) -> ExtractedContent {
        ExtractedContent {
            title: "Test".into(),
            text: text.into(),
            layer_used: CepLayer::Layer1,
            tokens: count_tokens(text),
            metadata: ContentMetadata::default(),
        }
    }

    #[test]
    fn scs_detects_headings() {
        let content = make_content("# Main Title\n\nSome paragraph content here.");
        let result = segment(&content);
        assert!(result
            .segments
            .iter()
            .any(|s| s.seg_type == SegmentType::Heading));
    }

    #[test]
    fn scs_detects_code_blocks() {
        let content =
            make_content("Some text\n\n```rust\nfn main() {\n    println!(\"hello\");\n}\n```\n\nMore text");
        let result = segment(&content);
        assert!(result
            .segments
            .iter()
            .any(|s| s.seg_type == SegmentType::Code));
    }

    #[test]
    fn scs_detects_lists() {
        let content = make_content("Items:\n\n- First item\n- Second item\n- Third item");
        let result = segment(&content);
        assert!(result
            .segments
            .iter()
            .any(|s| s.seg_type == SegmentType::List));
    }

    #[test]
    fn scs_table_to_json() {
        let content = make_content(
            "| Name | Price |\n|------|-------|\n| Widget A | $10 |\n| Widget B | $20 |",
        );
        let result = segment(&content);
        let table_seg = result
            .segments
            .iter()
            .find(|s| s.seg_type == SegmentType::Table);
        assert!(table_seg.is_some(), "Should detect table segment");
        let table = table_seg.unwrap();
        assert!(table.content.get("headers").is_some());
        assert!(table.content.get("rows").is_some());
    }

    #[test]
    fn scs_reduces_tokens() {
        let markdown_table = "| Feature | Status | Notes |\n|---------|--------|-------|\n| Auth | Done | OAuth2 |\n| Search | WIP | BM25 |\n| Cache | Done | LRU |";
        let content = make_content(markdown_table);
        let result = segment(&content);
        assert!(
            result.total_tokens <= content.tokens,
            "SCS ({}) should not exceed raw tokens ({})",
            result.total_tokens,
            content.tokens
        );
    }
}
