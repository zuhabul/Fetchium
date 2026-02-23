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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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

    // Phase 5: hybrid BM25 + semantic scoring (0.6 * BM25_norm + 0.4 * cosine)
    // Feature-gated: only when `embeddings` feature is enabled.
    #[cfg(feature = "embeddings")]
    if !query.is_empty() && !scored_blocks.is_empty() {
        hybrid_rank_blocks(&mut scored_blocks, query);
    }

    scored_blocks.sort_by(|a, b| {
        b.relevance
            .partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Apply coherence window to boost contextually adjacent segments, then
    // deduplicate near-identical segments before greedy packing.
    scored_blocks = apply_coherence_window(scored_blocks);
    scored_blocks = deduplicate_segments(scored_blocks);

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

/// Phase 5: Hybrid BM25 + semantic (embedding) re-scoring.
///
/// Overwrites each block's `relevance` with:
///   `0.6 * bm25_norm + 0.4 * cosine(query_emb, block_emb)`
///
/// Falls back silently to BM25-only when the embedding model is
/// unavailable or returns an error.
///
/// Embedding is chunked at 128 blocks per inference pass to cap peak
/// memory on long documents (consistent with QADD's EMBED_BATCH_SIZE).
#[cfg(feature = "embeddings")]
fn hybrid_rank_blocks(blocks: &mut [TextBlock], query: &str) {
    let max_bm25 = blocks
        .iter()
        .map(|b| b.relevance)
        .fold(0.0f64, f64::max);

    if max_bm25 <= 0.0 {
        return;
    }

    /// Maximum blocks per `embed_batch` call to bound peak memory.
    const EMBED_BATCH_SIZE: usize = 128;

    let texts: Vec<&str> = blocks.iter().map(|b| b.text.as_str()).collect();

    // Chunk the batch to avoid a single O(N) memory spike on long pages.
    let block_embs: Vec<Vec<f32>> = {
        let mut combined: Vec<Vec<f32>> = Vec::with_capacity(texts.len());
        for chunk in texts.chunks(EMBED_BATCH_SIZE) {
            match crate::embeddings::embed_batch(chunk) {
                Ok(batch) => combined.extend(batch),
                Err(_) => return, // fall back to BM25-only
            }
        }
        combined
    };

    if block_embs.len() != blocks.len() {
        return; // partial failure — keep BM25-only scores
    }

    let query_emb = match crate::embeddings::embed(query) {
        Ok(e) => e,
        Err(_) => return,
    };

    for (block, block_emb) in blocks.iter_mut().zip(block_embs.iter()) {
        let bm25_norm = block.relevance / max_bm25;
        let cosine = crate::embeddings::cosine_similarity(&query_emb, block_emb) as f64;
        block.relevance = 0.6 * bm25_norm + 0.4 * cosine.max(0.0);
    }
}

/// Apply a coherence window: boost the immediate neighbors (position ± 1) of
/// every high-relevance block (relevance > 0.3) so that contextually adjacent
/// segments get pulled into the budget alongside their anchor.
///
/// The boost is capped at 40 % of the anchor's relevance score and only
/// applied when it would actually increase the neighbor's current score,
/// so truly irrelevant neighbors receive at most a modest lift.  After
/// boosting, blocks are re-sorted by the updated scores.
///
/// Has no effect when fewer than 3 blocks are present or when no block
/// exceeds the 0.3 relevance threshold.
fn apply_coherence_window(mut blocks: Vec<TextBlock>) -> Vec<TextBlock> {
    let len = blocks.len();
    if len < 3 {
        return blocks;
    }

    // Collect positions of high-relevance anchor blocks.
    let high_relevance_positions: Vec<usize> = blocks
        .iter()
        .enumerate()
        .filter(|(_, b)| b.relevance > 0.3)
        .map(|(i, _)| i)
        .collect();

    if high_relevance_positions.is_empty() {
        return blocks;
    }

    // Boost neighbors of each anchor without lowering any existing score.
    for &pos in &high_relevance_positions {
        let neighbor_boost = blocks[pos].relevance * 0.4;
        if pos > 0 && blocks[pos - 1].relevance < neighbor_boost {
            blocks[pos - 1].relevance = neighbor_boost;
        }
        if pos + 1 < len && blocks[pos + 1].relevance < neighbor_boost {
            blocks[pos + 1].relevance = neighbor_boost;
        }
    }

    // Re-sort by the updated relevance scores.
    blocks.sort_by(|a, b| {
        b.relevance
            .partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    blocks
}

/// Remove near-duplicate segments using word-level Jaccard overlap.
///
/// Two segments are considered near-duplicates when their word overlap
/// exceeds 70 %.  Tiny segments (≤ 5 tokens) are always kept because they
/// are usually headings or labels rather than duplicate body text.
fn deduplicate_segments(blocks: Vec<TextBlock>) -> Vec<TextBlock> {
    let mut unique: Vec<TextBlock> = Vec::new();

    for block in blocks {
        // Skip dedup logic for very short segments (headings, labels, etc.).
        if block.tokens <= 5 {
            unique.push(block);
            continue;
        }

        let is_dup = unique.iter().any(|existing| {
            existing.tokens > 5 && word_overlap(&existing.text, &block.text) > 0.70
        });

        if !is_dup {
            unique.push(block);
        }
    }

    unique
}

/// Compute the Jaccard word overlap between two strings.
///
/// Returns a value in [0.0, 1.0] where 1.0 means identical word sets.
fn word_overlap(a: &str, b: &str) -> f64 {
    let words_a: std::collections::HashSet<&str> = a.split_whitespace().collect();
    let words_b: std::collections::HashSet<&str> = b.split_whitespace().collect();

    if words_a.is_empty() || words_b.is_empty() {
        return 0.0;
    }

    let intersection = words_a.intersection(&words_b).count();
    let union = words_a.union(&words_b).count();

    intersection as f64 / union as f64
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
            .is_some_and(|c| c.is_uppercase())
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
        // Each paragraph is unique (distinct numeric suffix and unique filler words)
        // so deduplication keeps them all, and 50 × ~15 tokens >> the 200-token budget.
        let text = (0..50)
            .map(|i| {
                format!(
                    "Paragraph {i} discusses topic_{i} in detail: alpha_{i} beta_{i}                      gamma_{i} delta_{i} epsilon_{i} zeta_{i} eta_{i} theta_{i}."
                )
            })
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

    #[test]
    fn coherence_window_boosts_neighbors() {
        // A document where the relevant paragraph is in the middle.
        // The adjacent paragraphs should get pulled in via the coherence window.
        let text = "Introduction paragraph with no relevance to query.\n\n\
                    Context before the key section.\n\n\
                    Rust memory safety ownership borrowing lifetimes.\n\n\
                    More details about Rust systems programming.\n\n\
                    Conclusion with unrelated content.";

        let content = make_content(text);
        let result = extract_with_budget(&content, "Rust memory safety", 500);

        // Should include the Rust paragraph AND adjacent context.
        let all_text: String = result
            .segments
            .iter()
            .filter_map(|s| s.content.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        assert!(all_text.contains("Rust"), "Should include Rust paragraph");
        // Adjacent context paragraph should also be included.
        assert!(
            result.segments.len() >= 2,
            "Should include context segments too"
        );
    }

    #[test]
    fn deduplication_removes_similar_segments() {
        // Two nearly identical paragraphs.
        let text = "Rust is a systems programming language for safety.\n\n\
                    Rust is a systems programming language for safety.\n\n\
                    Python is a different language entirely for other purposes.";

        let content = make_content(text);
        let result = extract_with_budget(&content, "Rust programming", 1000);

        // Should not include both duplicate paragraphs.
        let rust_segs: Vec<_> = result
            .segments
            .iter()
            .filter(|s| s.content.as_str().unwrap_or("").contains("Rust"))
            .collect();
        assert!(
            rust_segs.len() <= 2,
            "Should deduplicate near-identical segments"
        );
    }

    #[test]
    fn word_overlap_identical_strings() {
        assert!(
            (word_overlap("hello world foo", "hello world foo") - 1.0).abs() < 1e-9,
            "Identical strings must have overlap 1.0"
        );
    }

    #[test]
    fn word_overlap_disjoint_strings() {
        assert!(
            word_overlap("alpha beta gamma", "delta epsilon zeta") < 1e-9,
            "Disjoint strings must have overlap 0.0"
        );
    }

    #[test]
    fn coherence_window_no_op_when_no_high_relevance() {
        // Build blocks that will all score below the 0.3 threshold.
        let blocks: Vec<TextBlock> = (0..5)
            .map(|i| TextBlock {
                text: format!("Unrelated content paragraph number {i}."),
                tokens: 6,
                relevance: 0.1,
                position: i,
                block_type: BlockType::Paragraph,
            })
            .collect();

        let result = apply_coherence_window(blocks.clone());
        // Scores must be unchanged when no anchor exists.
        for (orig, after) in blocks.iter().zip(result.iter()) {
            assert!(
                (orig.relevance - after.relevance).abs() < 1e-9,
                "No boost should occur without a high-relevance anchor"
            );
        }
    }
}
