//! Progressive Detail Streaming (PDS) — PRD §27.
//!
//! 4-tier content delivery system that adapts output depth to the
//! caller's token budget. Agents can request exactly the level of
//! detail they need without over-fetching.
//!
//! Tier 1 (key_facts):  ~200  tokens — top 3-5 facts, highest-relevance segments only
//! Tier 2 (summary):    ~1000 tokens — overview + key points, moderate context
//! Tier 3 (detailed):   ~5000 tokens — full content with all context
//! Tier 4 (complete):   unlimited    — raw extracted text, no truncation

use crate::token::counter::{count_tokens, TokenBudget};
use crate::types::{PdsTier, Segment, SegmentType};
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Token targets for each PDS tier.
pub const KEY_FACTS_BUDGET: u32 = 200;
pub const SUMMARY_BUDGET: u32 = 1_000;
pub const DETAILED_BUDGET: u32 = 5_000;

/// Result of a PDS operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdsResult {
    /// Tier applied.
    pub tier: PdsTier,
    /// Content adapted to the tier budget.
    pub content: String,
    /// Segments selected for this tier (for agent output).
    pub segments: Vec<Segment>,
    /// Tokens in the result.
    pub tokens_used: u32,
    /// Whether the content was truncated from the original.
    pub truncated: bool,
}

/// Apply PDS to extracted text, producing a tier-appropriate view.
///
/// # Arguments
/// * `text` - Full extracted text from CEP pipeline
/// * `segments` - Pre-scored QATBE segments (sorted by relevance, highest first)
/// * `tier` - Requested PDS tier
///
/// # Returns
/// A `PdsResult` containing content adapted to the tier budget.
pub fn apply_tier(text: &str, segments: &[Segment], tier: PdsTier) -> PdsResult {
    match tier {
        PdsTier::KeyFacts => apply_key_facts(text, segments),
        PdsTier::Summary => apply_summary(text, segments),
        PdsTier::Detailed => apply_detailed(text, segments),
        PdsTier::Complete => apply_complete(text, segments),
    }
}

/// Tier 1: key_facts — highest-relevance segments only, ~200 tokens.
///
/// Selects the top 3-5 segments by relevance score. If no pre-scored
/// segments are available, returns the first 200 tokens of text.
fn apply_key_facts(text: &str, segments: &[Segment]) -> PdsResult {
    let budget = KEY_FACTS_BUDGET;

    if segments.is_empty() {
        // Fallback: take first N tokens of raw text
        let content = truncate_to_budget(text, budget);
        let tokens_used = count_tokens(&content);
        let truncated = tokens_used < count_tokens(text);
        debug!("PDS key_facts (fallback): {} tokens", tokens_used);
        return PdsResult {
            tier: PdsTier::KeyFacts,
            content,
            segments: Vec::new(),
            tokens_used,
            truncated,
        };
    }

    // Take top-relevance segments within budget, max 5
    let mut selected = Vec::new();
    let mut tracker = TokenBudget::new(budget);
    let mut content_parts = Vec::new();

    // Segments should already be sorted by relevance desc
    for seg in segments.iter().take(10) {
        if tracker.is_exhausted() || selected.len() >= 5 {
            break;
        }
        if tracker.try_consume(seg.tokens) {
            let text_val = segment_to_text(seg);
            content_parts.push(text_val);
            selected.push(seg.clone());
        }
    }

    let content = content_parts.join("\n\n");
    let tokens_used = tracker.used;
    let truncated = tokens_used < count_tokens(text);

    debug!("PDS key_facts: {} segments, {} tokens", selected.len(), tokens_used);

    PdsResult {
        tier: PdsTier::KeyFacts,
        content,
        segments: selected,
        tokens_used,
        truncated,
    }
}

/// Tier 2: summary — overview + key points, ~1000 tokens.
///
/// Takes top segments by relevance up to the summary budget.
/// Adds a brief heading structure if the content permits.
fn apply_summary(text: &str, segments: &[Segment]) -> PdsResult {
    let budget = SUMMARY_BUDGET;

    if segments.is_empty() {
        let content = truncate_to_budget(text, budget);
        let tokens_used = count_tokens(&content);
        let truncated = tokens_used < count_tokens(text);
        debug!("PDS summary (fallback): {} tokens", tokens_used);
        return PdsResult {
            tier: PdsTier::Summary,
            content,
            segments: Vec::new(),
            tokens_used,
            truncated,
        };
    }

    let mut selected = Vec::new();
    let mut tracker = TokenBudget::new(budget);
    let mut content_parts = Vec::new();

    // Include headings first (structural anchors), then body content
    let headings: Vec<&Segment> = segments
        .iter()
        .filter(|s| s.seg_type == SegmentType::Heading)
        .take(3)
        .collect();
    let body: Vec<&Segment> = segments
        .iter()
        .filter(|s| s.seg_type != SegmentType::Heading)
        .collect();

    for seg in headings.into_iter().chain(body.into_iter()) {
        if tracker.is_exhausted() {
            break;
        }
        if tracker.try_consume(seg.tokens) {
            content_parts.push(segment_to_text(seg));
            selected.push(seg.clone());
        } else {
            // Try partial inclusion for final segment
            let remaining = tracker.remaining();
            if remaining > 20 {
                let text_val = segment_to_text(seg);
                let truncated_text = truncate_to_budget(&text_val, remaining);
                let t = count_tokens(&truncated_text);
                tracker.consume(t);
                let mut partial = seg.clone();
                partial.tokens = t;
                partial.content = serde_json::Value::String(truncated_text.clone());
                content_parts.push(truncated_text);
                selected.push(partial);
            }
            break;
        }
    }

    let content = content_parts.join("\n\n");
    let tokens_used = tracker.used;
    let truncated = tokens_used < count_tokens(text);

    debug!("PDS summary: {} segments, {} tokens", selected.len(), tokens_used);

    PdsResult {
        tier: PdsTier::Summary,
        content,
        segments: selected,
        tokens_used,
        truncated,
    }
}

/// Tier 3: detailed — full content with context, ~5000 tokens.
///
/// Includes all segments up to the detailed budget. For most pages
/// this captures the entire article.
fn apply_detailed(text: &str, segments: &[Segment]) -> PdsResult {
    let budget = DETAILED_BUDGET;

    if segments.is_empty() {
        let content = truncate_to_budget(text, budget);
        let tokens_used = count_tokens(&content);
        let truncated = tokens_used < count_tokens(text);
        debug!("PDS detailed (fallback): {} tokens", tokens_used);
        return PdsResult {
            tier: PdsTier::Detailed,
            content,
            segments: Vec::new(),
            tokens_used,
            truncated,
        };
    }

    let mut selected = Vec::new();
    let mut tracker = TokenBudget::new(budget);
    let mut content_parts = Vec::new();

    // For detailed, restore original document order (by source_ref position)
    let mut ordered: Vec<&Segment> = segments.iter().collect();
    ordered.sort_by_key(|s| s.source_ref.unwrap_or(u32::MAX));

    for seg in ordered {
        if tracker.is_exhausted() {
            break;
        }
        if tracker.try_consume(seg.tokens) {
            content_parts.push(segment_to_text(seg));
            selected.push(seg.clone());
        } else {
            let remaining = tracker.remaining();
            if remaining > 50 {
                let text_val = segment_to_text(seg);
                let truncated_text = truncate_to_budget(&text_val, remaining);
                let t = count_tokens(&truncated_text);
                tracker.consume(t);
                let mut partial = seg.clone();
                partial.tokens = t;
                partial.content = serde_json::Value::String(truncated_text.clone());
                content_parts.push(truncated_text);
                selected.push(partial);
            }
            break;
        }
    }

    let content = content_parts.join("\n\n");
    let tokens_used = tracker.used;
    let truncated = tokens_used < count_tokens(text);

    debug!("PDS detailed: {} segments, {} tokens", selected.len(), tokens_used);

    PdsResult {
        tier: PdsTier::Detailed,
        content,
        segments: selected,
        tokens_used,
        truncated,
    }
}

/// Tier 4: complete — raw extracted text, no truncation.
///
/// Returns the entire extracted text as-is, plus all segments.
/// Use this when you want everything and handle truncation yourself.
fn apply_complete(text: &str, segments: &[Segment]) -> PdsResult {
    let tokens_used = count_tokens(text);
    debug!("PDS complete: {} tokens", tokens_used);

    PdsResult {
        tier: PdsTier::Complete,
        content: text.to_string(),
        segments: segments.to_vec(),
        tokens_used,
        truncated: false,
    }
}

/// Convert a segment's content to plain text for output.
fn segment_to_text(seg: &Segment) -> String {
    match &seg.content {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
            serde_json::to_string_pretty(&seg.content).unwrap_or_default()
        }
        other => other.to_string(),
    }
}

/// Truncate text to approximately fit within a token budget.
/// Cuts at a word boundary and appends "..." if truncated.
fn truncate_to_budget(text: &str, budget: u32) -> String {
    let current_tokens = count_tokens(text);
    if current_tokens <= budget {
        return text.to_string();
    }

    // Estimate max characters from budget
    let max_chars = (budget as f64 * 4.0) as usize;
    if max_chars >= text.len() {
        return text.to_string();
    }

    let truncated = &text[..max_chars.min(text.len())];
    match truncated.rfind(|c: char| c == ' ' || c == '\n') {
        Some(pos) if pos > max_chars / 2 => {
            format!("{}...", truncated[..pos].trim_end())
        }
        _ => format!("{truncated}..."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SegmentType;

    fn make_segment(text: &str, seg_type: SegmentType, relevance: f64, position: u32) -> Segment {
        Segment {
            seg_type,
            relevance,
            tokens: count_tokens(text),
            content: serde_json::Value::String(text.to_string()),
            source_ref: Some(position),
        }
    }

    #[test]
    fn key_facts_respects_budget() {
        let text = "Long article content with many paragraphs. ".repeat(200);
        let segs = vec![
            make_segment("Key finding one about Rust.", SegmentType::Fact, 0.9, 0),
            make_segment("Key finding two about performance.", SegmentType::Fact, 0.8, 1),
            make_segment("Supporting detail with more context.", SegmentType::Paragraph, 0.5, 2),
        ];
        let result = apply_tier(&text, &segs, PdsTier::KeyFacts);
        assert_eq!(result.tier, PdsTier::KeyFacts);
        assert!(result.tokens_used <= KEY_FACTS_BUDGET + 10); // small slack for estimation
        assert!(!result.content.is_empty());
    }

    #[test]
    fn summary_tier_includes_more_than_key_facts() {
        let text = "Article with lots of content. ".repeat(200);
        let segs: Vec<Segment> = (0..20)
            .map(|i| make_segment(
                &format!("Segment {i} with some substantial content that fills tokens."),
                SegmentType::Paragraph,
                1.0 - (i as f64 * 0.05),
                i as u32,
            ))
            .collect();

        let key_facts = apply_tier(&text, &segs, PdsTier::KeyFacts);
        let summary = apply_tier(&text, &segs, PdsTier::Summary);
        assert!(summary.tokens_used >= key_facts.tokens_used);
        assert!(summary.tokens_used <= SUMMARY_BUDGET + 20);
    }

    #[test]
    fn complete_tier_returns_everything() {
        let text = "Short content.";
        let result = apply_tier(text, &[], PdsTier::Complete);
        assert_eq!(result.tier, PdsTier::Complete);
        assert_eq!(result.content, text);
        assert!(!result.truncated);
    }

    #[test]
    fn truncate_at_word_boundary() {
        let text = "Hello world this is a long piece of text that should be truncated.";
        let truncated = truncate_to_budget(text, 5);
        assert!(truncated.ends_with("..."));
        assert!(count_tokens(&truncated) <= 10); // with some slack
    }

    #[test]
    fn key_facts_fallback_without_segments() {
        let text = "This is the article content. ".repeat(50);
        let result = apply_tier(&text, &[], PdsTier::KeyFacts);
        assert!(result.tokens_used <= KEY_FACTS_BUDGET + 10);
        assert!(!result.content.is_empty());
    }

    #[test]
    fn detailed_preserves_document_order() {
        let text = "Doc content. ".repeat(100);
        let segs = vec![
            make_segment("Third section.", SegmentType::Paragraph, 0.5, 2),
            make_segment("First section.", SegmentType::Paragraph, 0.9, 0),
            make_segment("Second section.", SegmentType::Paragraph, 0.7, 1),
        ];
        let result = apply_tier(&text, &segs, PdsTier::Detailed);
        // Should be reordered by source_ref: first, second, third
        let content = result.content;
        let first_pos = content.find("First").unwrap_or(usize::MAX);
        let second_pos = content.find("Second").unwrap_or(usize::MAX);
        let third_pos = content.find("Third").unwrap_or(usize::MAX);
        assert!(first_pos < second_pos, "First should come before Second");
        assert!(second_pos < third_pos, "Second should come before Third");
    }
}
