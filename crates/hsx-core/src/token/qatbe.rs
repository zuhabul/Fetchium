//! Query-Aware Token-Budgeted Extraction (QATBE).
//!
//! PRD SS8.2: The single most important feature for AI agent consumption.
//! 4-stage pipeline: FETCH -> SEGMENT -> RANK -> BUDGET.
//!
//! Phase 1: Uses BM25 scoring for relevance ranking.
//! Phase 5: Adds semantic embeddings (cosine similarity) for hybrid scoring.

use crate::extract::ExtractedContent;
use crate::token::counter::{count_tokens, TokenBudget};
use crate::types::{Segment, SegmentType};
use serde::{Deserialize, Serialize};
use tracing::info;

/// QATBE extraction result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QatbeResult {
    /// Segments packed within the token budget, ordered by relevance.
    pub segments: Vec<Segment>,
    /// Total tokens used.
    pub tokens_used: u32,
    /// Total tokens available from the source.
    pub tokens_total: u32,
    /// Fraction of relevant content captured (0.0-1.0).
    pub relevance_coverage: f64,
    /// Number of segments included.
    pub segments_included: u32,
    /// Number of segments excluded due to budget.
    pub segments_excluded: u32,
}

/// A text block extracted during the SEGMENT stage, before SCS typing.
#[derive(Debug, Clone)]
struct TextBlock {
    text: String,
    tokens: u32,
    relevance: f64,
    position: usize,
    block_type: BlockType,
}

#[derive(Debug, Clone, Copy)]
enum BlockType {
    Heading,
    Paragraph,
    ListItem,
    Code,
    Table,
    Quote,
    Other,
}

/// Run the QATBE pipeline on extracted content.
pub fn extract_with_budget(
    content: &ExtractedContent,
    query: &str,
    budget: u32,
) -> QatbeResult {
    info!(
        "QATBE: query={query:?}, budget={budget}, source_tokens={}",
        content.tokens
    );

    let blocks = segment_content(&content.text);

    let mut scored_blocks: Vec<TextBlock> = blocks
        .into_iter()
        .map(|mut block| {
            block.relevance = bm25_score(&block.text, query);
            block
        })
        .collect();

    scored_blocks.sort_by(|a, b| {
        b.relevance
            .partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut tracker = TokenBudget::new(budget);
    let mut included_segments = Vec::new();
    let mut excluded_count = 0u32;
    let total_relevant_tokens: u32 = scored_blocks
        .iter()
        .filter(|b| b.relevance > 0.01)
        .map(|b| b.tokens)
        .sum();

    for block in &scored_blocks {
        if tracker.is_exhausted() {
            excluded_count += 1;
            continue;
        }

        if block.relevance < 0.01 {
            excluded_count += 1;
            continue;
        }

        if tracker.try_consume(block.tokens) {
            included_segments.push(block_to_segment(block, included_segments.len()));
        } else {
            let remaining = tracker.remaining();
            if remaining > 10 {
                let truncated = truncate_to_tokens(&block.text, remaining);
                let truncated_tokens = count_tokens(&truncated);
                tracker.consume(truncated_tokens);

                let mut seg = block_to_segment(block, included_segments.len());
                seg.tokens = truncated_tokens;
                seg.content = serde_json::Value::String(truncated);
                included_segments.push(seg);
            }
            excluded_count += 1;
        }
    }

    let included_relevant_tokens: u32 = included_segments
        .iter()
        .filter(|s| s.relevance > 0.01)
        .map(|s| s.tokens)
        .sum();
    let coverage = if total_relevant_tokens > 0 {
        included_relevant_tokens as f64 / total_relevant_tokens as f64
    } else {
        0.0
    };

    info!(
        "QATBE: packed {} segments ({} tokens, {:.0}% coverage)",
        included_segments.len(),
        tracker.used,
        coverage * 100.0,
    );

    let segments_included = included_segments.len() as u32;
    QatbeResult {
        segments: included_segments,
        tokens_used: tracker.used,
        tokens_total: content.tokens,
        relevance_coverage: coverage,
        segments_included,
        segments_excluded: excluded_count,
    }
}

/// Stage 2: Split extracted text into typed blocks.
fn segment_content(text: &str) -> Vec<TextBlock> {
    let mut blocks = Vec::new();
    let mut position = 0;

    for chunk in text.split("\n\n") {
        let trimmed = chunk.trim();
        if trimmed.is_empty() {
            continue;
        }

        let block_type = classify_block(trimmed);
        let tokens = count_tokens(trimmed);

        if tokens < 3 {
            continue;
        }

        blocks.push(TextBlock {
            text: trimmed.to_string(),
            tokens,
            relevance: 0.0,
            position,
            block_type,
        });

        position += 1;
    }

    if blocks.is_empty() {
        for (i, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let tokens = count_tokens(trimmed);
            if tokens < 3 {
                continue;
            }
            blocks.push(TextBlock {
                text: trimmed.to_string(),
                tokens,
                relevance: 0.0,
                position: i,
                block_type: classify_block(trimmed),
            });
        }
    }

    blocks
}

/// Classify a text block by its likely type.
fn classify_block(text: &str) -> BlockType {
    let trimmed = text.trim();

    if trimmed.len() < 100
        && !trimmed.contains('.')
        && trimmed
            .chars()
            .next()
            .map_or(false, |c| c.is_uppercase())
    {
        return BlockType::Heading;
    }

    if trimmed.starts_with("```")
        || trimmed.starts_with("    ")
        || trimmed.starts_with('\t')
        || (trimmed.contains('{') && trimmed.contains('}'))
        || (trimmed.contains("fn ") && trimmed.contains("("))
        || (trimmed.contains("def ") && trimmed.contains(":"))
    {
        return BlockType::Code;
    }

    if trimmed.starts_with("- ")
        || trimmed.starts_with("* ")
        || trimmed.starts_with("+ ")
        || (trimmed.len() > 2
            && trimmed.chars().next().unwrap_or(' ').is_ascii_digit()
            && trimmed.chars().nth(1) == Some('.'))
    {
        return BlockType::ListItem;
    }

    if trimmed.starts_with('>') || trimmed.starts_with('"') {
        return BlockType::Quote;
    }

    if trimmed.contains('|') && trimmed.matches('|').count() >= 2 {
        return BlockType::Table;
    }

    BlockType::Paragraph
}

/// BM25 scoring for a text block against a query.
fn bm25_score(text: &str, query: &str) -> f64 {
    let k1: f64 = 1.2;
    let b: f64 = 0.75;

    let text_lower = text.to_lowercase();
    let text_words: Vec<&str> = text_lower.split_whitespace().collect();
    let doc_len = text_words.len() as f64;

    let avg_dl: f64 = 100.0;

    let query_terms: Vec<String> = query
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    let mut score = 0.0;

    for term in &query_terms {
        let tf = text_words
            .iter()
            .filter(|w| w.contains(term.as_str()))
            .count() as f64;

        if tf == 0.0 {
            continue;
        }

        let idf = 1.0;

        let numerator = tf * (k1 + 1.0);
        let denominator = tf + k1 * (1.0 - b + b * doc_len / avg_dl);

        score += idf * numerator / denominator;
    }

    let max_possible = query_terms.len() as f64 * 2.0;
    if max_possible > 0.0 {
        (score / max_possible).min(1.0)
    } else {
        0.0
    }
}

/// Convert a TextBlock to a Segment.
fn block_to_segment(block: &TextBlock, index: usize) -> Segment {
    let seg_type = match block.block_type {
        BlockType::Heading => SegmentType::Heading,
        BlockType::Paragraph => SegmentType::Paragraph,
        BlockType::ListItem => SegmentType::List,
        BlockType::Code => SegmentType::Code,
        BlockType::Table => SegmentType::Table,
        BlockType::Quote => SegmentType::Quote,
        BlockType::Other => SegmentType::Paragraph,
    };

    Segment {
        seg_type,
        relevance: block.relevance,
        tokens: block.tokens,
        content: serde_json::Value::String(block.text.clone()),
        source_ref: Some(index as u32),
    }
}

/// Truncate text to fit within an approximate token count.
fn truncate_to_tokens(text: &str, max_tokens: u32) -> String {
    let max_chars = (max_tokens as f64 * 4.0) as usize;
    if text.len() <= max_chars {
        return text.to_string();
    }

    let truncated = &text[..max_chars];
    match truncated.rfind(' ') {
        Some(pos) => format!("{}...", &truncated[..pos]),
        None => format!("{truncated}..."),
    }
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
    fn qatbe_basic_extraction() {
        let text = "Rust is a systems programming language focused on safety.\n\n\
                    Python is an interpreted language for general purpose coding.\n\n\
                    JavaScript runs in the browser and on the server with Node.js.\n\n\
                    Go is designed for cloud infrastructure and microservices.";

        let content = make_content(text);
        let result = extract_with_budget(&content, "Rust systems programming", 100);

        assert!(result.tokens_used <= 100);
        assert!(!result.segments.is_empty());
        let first = &result.segments[0];
        assert!(
            first
                .content
                .as_str()
                .unwrap_or("")
                .contains("Rust"),
            "First segment should be about Rust"
        );
    }

    #[test]
    fn qatbe_respects_budget() {
        let text = (0..50)
            .map(|i| format!("Paragraph {i} with some content about various topics."))
            .collect::<Vec<_>>()
            .join("\n\n");

        let content = make_content(&text);
        let result = extract_with_budget(&content, "topic", 200);

        assert!(result.tokens_used <= 200);
        assert!(result.segments_excluded > 0);
    }

    #[test]
    fn bm25_relevance_ordering() {
        let high = bm25_score(
            "Rust is a systems programming language focused on safety",
            "Rust programming",
        );
        let low = bm25_score(
            "The weather today is sunny and warm with clear skies",
            "Rust programming",
        );
        assert!(
            high > low,
            "Relevant text ({high}) should score higher than irrelevant ({low})"
        );
    }

    #[test]
    fn classify_block_types() {
        assert!(matches!(classify_block("Main Title"), BlockType::Heading));
        assert!(matches!(
            classify_block("- item one\n- item two"),
            BlockType::ListItem
        ));
        assert!(matches!(
            classify_block("```rust\nfn main() {}\n```"),
            BlockType::Code
        ));
        assert!(matches!(
            classify_block("> This is a quote from someone important."),
            BlockType::Quote
        ));
        assert!(matches!(
            classify_block("This is a regular paragraph with multiple sentences. It contains details."),
            BlockType::Paragraph
        ));
    }

    #[test]
    fn truncate_at_word_boundary() {
        let text = "Hello world this is a test of truncation at word boundaries";
        let truncated = truncate_to_tokens(text, 5);
        assert!(truncated.ends_with("..."));
        assert!(truncated.len() < text.len());
    }
}
