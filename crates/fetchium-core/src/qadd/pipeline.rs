//! QADD 5-step pipeline — Query-Aware DOM Distillation (PRD §8.10).
//!
//! Reduces a full HTML page to query-relevant text within a token budget.
//! Target: 50K tokens → ~2.5K tokens (10-20x reduction).
//!
//! Steps:
//! 1. Structural pruning (remove nav/footer/script/ads)
//! 2. BM25 scoring + threshold pruning
//! 3. Semantic word-overlap check
//! 4. Heading context preservation (keep headings near relevant paragraphs)
//! 5. Greedy knapsack packing within token budget

use crate::qadd::pruning::{estimate_tokens, structural_prune, TextNode};
use tracing::{debug, trace};

/// Configuration for the QADD pipeline.
#[derive(Debug, Clone)]
pub struct QaddConfig {
    /// BM25 threshold — nodes below this score are pruned (default: 0.05).
    pub bm25_threshold: f64,
    /// Semantic overlap threshold (default: 0.1).
    pub semantic_threshold: f64,
    /// Target token budget for the distilled output (default: 2000).
    pub token_budget: usize,
    /// Minimum token count to attempt partial node inclusion (default: 50).
    pub min_context_tokens: usize,
}

impl Default for QaddConfig {
    fn default() -> Self {
        Self {
            bm25_threshold: 0.05,
            semantic_threshold: 0.1,
            token_budget: 2000,
            min_context_tokens: 50,
        }
    }
}

/// Output of the QADD distillation pipeline.
#[derive(Debug, Clone)]
pub struct QaddResult {
    /// The distilled, query-relevant content as plain text.
    pub distilled_content: String,
    /// Estimated token count of the original HTML.
    pub tokens_original: usize,
    /// Estimated token count of the distilled output.
    pub tokens_distilled: usize,
    /// Number of text nodes kept.
    pub nodes_kept: usize,
    /// Number of text nodes pruned.
    pub nodes_pruned: usize,
    /// Compression ratio (tokens_distilled / tokens_original).
    pub compression_ratio: f64,
}

/// The QADD 5-step pipeline.
pub struct QaddPipeline {
    config: QaddConfig,
}

impl QaddPipeline {
    pub fn new(config: QaddConfig) -> Self {
        Self { config }
    }

    /// Run the full QADD distillation pipeline.
    ///
    /// # Arguments
    /// * `html` - Raw HTML content of the page
    /// * `query` - The user's search query (used for relevance scoring)
    pub fn distill(&self, html: &str, query: &str) -> QaddResult {
        let tokens_original = estimate_tokens(html);

        // Step 1: Structural pruning
        let mut nodes = structural_prune(html);
        let total_nodes = nodes.len();

        // Build query terms
        let query_terms: Vec<String> = query
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| t.len() >= 2)
            .map(|t| t.to_string())
            .collect();

        if query_terms.is_empty() {
            // No query terms — return top content up to budget
            return self.pack_nodes(nodes, tokens_original, total_nodes);
        }

        // Step 2: BM25 score each node, prune below threshold
        for node in &mut nodes {
            node.relevance_score = bm25_node_score(&node.text, &query_terms);
        }
        let before_step2 = nodes.len();
        nodes.retain(|n| n.is_heading() || n.relevance_score >= self.config.bm25_threshold);
        trace!(
            "QADD step2 (BM25): {} → {} nodes",
            before_step2,
            nodes.len()
        );

        // Step 3: Semantic similarity — embedding cosine (feature) or word-overlap fallback
        let before_step3 = nodes.len();
        #[cfg(feature = "embeddings")]
        {
            use crate::embeddings::{cosine_similarity, embed, embed_batch};

            /// Maximum nodes per `embed_batch` call to cap peak memory (~50 MB per pass).
            const EMBED_BATCH_SIZE: usize = 128;

            let node_texts: Vec<&str> = nodes.iter().map(|n| n.text.as_str()).collect();
            let query_text = query_terms.join(" ");

            // Chunk embed_batch to avoid a single O(N) spike on large pages.
            let all_node_embs: Option<Vec<Vec<f32>>> = {
                let mut combined: Vec<Vec<f32>> = Vec::with_capacity(node_texts.len());
                let mut ok = true;
                for chunk in node_texts.chunks(EMBED_BATCH_SIZE) {
                    match embed_batch(chunk) {
                        Ok(batch) => combined.extend(batch),
                        Err(_) => {
                            ok = false;
                            break;
                        }
                    }
                }
                if ok && combined.len() == node_texts.len() {
                    Some(combined)
                } else {
                    None
                }
            };

            if let (Some(node_embs), Ok(q_emb)) = (all_node_embs, embed(&query_text)) {
                let threshold = self.config.semantic_threshold as f32;
                let scores: Vec<f32> = node_embs
                    .iter()
                    .map(|e| cosine_similarity(&q_emb, e))
                    .collect();
                nodes = nodes
                    .into_iter()
                    .enumerate()
                    .filter(|(i, n)| {
                        n.is_heading() || scores.get(*i).copied().unwrap_or(0.0) >= threshold
                    })
                    .map(|(_, n)| n)
                    .collect();
            } else {
                nodes.retain(|n| {
                    n.is_heading()
                        || semantic_overlap(&n.text, &query_terms) >= self.config.semantic_threshold
                });
            }
        }
        #[cfg(not(feature = "embeddings"))]
        nodes.retain(|n| {
            n.is_heading()
                || semantic_overlap(&n.text, &query_terms) >= self.config.semantic_threshold
        });
        trace!(
            "QADD step3 (semantic): {} → {} nodes",
            before_step3,
            nodes.len()
        );

        // Step 4: Heading context preservation
        let nodes_with_context = add_heading_context(nodes);

        // Step 5: Greedy knapsack packing
        self.pack_nodes(nodes_with_context, tokens_original, total_nodes)
    }

    /// Step 5: Greedy packing — sort by relevance desc, take until budget exhausted.
    fn pack_nodes(
        &self,
        mut nodes: Vec<TextNode>,
        tokens_original: usize,
        total_nodes: usize,
    ) -> QaddResult {
        // Sort: headings first (context), then by relevance descending
        nodes.sort_by(|a, b| {
            if a.is_heading() && !b.is_heading() {
                return std::cmp::Ordering::Less;
            }
            if !a.is_heading() && b.is_heading() {
                return std::cmp::Ordering::Greater;
            }
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut output_parts: Vec<String> = Vec::new();
        let mut tokens_used = 0usize;
        let mut nodes_kept = 0usize;
        let budget = self.config.token_budget;

        for node in &nodes {
            if tokens_used >= budget {
                break;
            }
            let remaining = budget - tokens_used;
            if node.estimated_tokens <= remaining {
                output_parts.push(node.text.clone());
                tokens_used += node.estimated_tokens;
                nodes_kept += 1;
            } else if remaining >= self.config.min_context_tokens {
                // Partial fit: truncate to remaining budget
                let target_chars = remaining * 4;
                let truncated: String = node.text.chars().take(target_chars).collect();
                let truncated = format!("{}...", truncated.trim_end());
                output_parts.push(truncated);
                nodes_kept += 1;
                break;
            }
        }

        let distilled_content = output_parts.join("\n\n");
        let tokens_distilled = estimate_tokens(&distilled_content);
        let nodes_pruned = total_nodes.saturating_sub(nodes_kept);

        let compression_ratio = if tokens_original > 0 {
            tokens_distilled as f64 / tokens_original as f64
        } else {
            1.0
        };

        debug!(
            "QADD: {tokens_original} → {tokens_distilled} tokens ({:.1}x reduction), {nodes_kept}/{total_nodes} nodes",
            if compression_ratio > 0.0 {
                1.0 / compression_ratio
            } else {
                0.0
            }
        );

        QaddResult {
            distilled_content,
            tokens_original,
            tokens_distilled,
            nodes_kept,
            nodes_pruned,
            compression_ratio,
        }
    }
}

/// BM25 relevance score for a single text node against query terms.
/// Simplified (no corpus IDF): IDF = 1.0 for all terms.
fn bm25_node_score(text: &str, query_terms: &[String]) -> f64 {
    const K1: f64 = 1.2;
    const B: f64 = 0.75;
    const AVG_DL: f64 = 50.0; // shorter docs expected in DOM nodes

    if query_terms.is_empty() {
        return 0.0;
    }

    let text_lower = text.to_lowercase();
    let words: Vec<&str> = text_lower.split_whitespace().collect();
    let doc_len = words.len() as f64;

    let mut score = 0.0f64;
    for term in query_terms {
        let tf = words.iter().filter(|w| **w == term.as_str()).count() as f64;
        if tf > 0.0 {
            let num = tf * (K1 + 1.0);
            let den = tf + K1 * (1.0 - B + B * doc_len / AVG_DL);
            score += num / den;
        } else {
            // Partial match (substring)
            let partial = words.iter().filter(|w| w.contains(term.as_str())).count() as f64;
            if partial > 0.0 {
                score += 0.3;
            }
        }
    }

    let max_possible = query_terms.len() as f64 * (K1 + 1.0);
    (score / max_possible).min(1.0)
}

/// Semantic overlap: fraction of query terms present in text.
fn semantic_overlap(text: &str, query_terms: &[String]) -> f64 {
    if query_terms.is_empty() {
        return 1.0;
    }
    let text_lower = text.to_lowercase();
    let matched = query_terms
        .iter()
        .filter(|t| text_lower.contains(t.as_str()))
        .count();
    matched as f64 / query_terms.len() as f64
}

/// Step 4: Ensure headings appear before their related content blocks.
///
/// Emits pending headings only when followed by a content node,
/// preventing orphan headings from consuming the token budget.
fn add_heading_context(retained: Vec<TextNode>) -> Vec<TextNode> {
    let mut result: Vec<TextNode> = Vec::with_capacity(retained.len());
    let mut pending_heading: Option<TextNode> = None;
    let mut last_heading_text = String::new();

    for node in retained {
        if node.is_heading() {
            pending_heading = Some(node);
        } else {
            // Content node: emit pending heading first (dedup by text)
            if let Some(h) = pending_heading.take() {
                if h.text != last_heading_text {
                    last_heading_text = h.text.clone();
                    result.push(h);
                }
            }
            result.push(node);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_HTML: &str = r#"<!DOCTYPE html>
<html>
<head><title>Test Page</title></head>
<body>
<nav>Navigation links skip me please</nav>
<header>Site header banner content</header>
<main>
  <h1>Rust Programming Language</h1>
  <p>Rust is a systems programming language that runs blazingly fast, prevents segfaults,
  and guarantees thread safety. It achieves memory safety without garbage collection
  through its ownership system and borrow checker.</p>
  <h2>Key Features of Rust</h2>
  <p>Memory safety without GC, zero-cost abstractions, and fearless concurrency are
  the key features of Rust. Performance is comparable to C and C++.</p>
  <aside>Related articles about other programming languages</aside>
  <h2>Installation Guide</h2>
  <p>Install Rust using rustup, the official Rust toolchain installer. Run the
  install script to get started with the Rust programming environment.</p>
</main>
<footer>Copyright 2024 footer navigation content</footer>
<script>var tracking = analytics.track(true);</script>
</body>
</html>"#;

    #[test]
    fn distill_within_budget() {
        let pipeline = QaddPipeline::new(QaddConfig {
            token_budget: 200,
            ..Default::default()
        });
        let result = pipeline.distill(SAMPLE_HTML, "Rust programming");
        assert!(
            result.tokens_distilled <= 260,
            "distilled={} (budget=200)",
            result.tokens_distilled
        );
        assert!(
            result.tokens_original > result.tokens_distilled,
            "must compress"
        );
    }

    #[test]
    fn distill_keeps_relevant_content() {
        let pipeline = QaddPipeline::new(QaddConfig::default());
        let result = pipeline.distill(SAMPLE_HTML, "Rust memory safety");
        let content_lower = result.distilled_content.to_lowercase();
        assert!(
            content_lower.contains("rust") || content_lower.contains("memory"),
            "should contain Rust/memory content; got: {}",
            result.distilled_content
        );
    }

    #[test]
    fn distill_removes_nav_script() {
        let pipeline = QaddPipeline::new(QaddConfig::default());
        let result = pipeline.distill(SAMPLE_HTML, "Rust programming");
        assert!(
            !result.distilled_content.contains("Navigation links"),
            "nav should be pruned"
        );
        assert!(
            !result.distilled_content.contains("analytics.track"),
            "script should be pruned"
        );
    }

    #[test]
    fn distill_compression_ratio_less_than_one() {
        let pipeline = QaddPipeline::new(QaddConfig::default());
        let result = pipeline.distill(SAMPLE_HTML, "Rust programming language");
        assert!(
            result.compression_ratio < 1.0,
            "ratio={:.3} (must compress)",
            result.compression_ratio
        );
        assert!(result.compression_ratio > 0.0);
    }

    #[test]
    fn bm25_node_score_relevant_higher() {
        let terms: Vec<String> = vec!["rust".to_string(), "programming".to_string()];
        let relevant = bm25_node_score("Rust is a programming language for systems", &terms);
        let irrelevant = bm25_node_score("Cookie policy notification preferences", &terms);
        assert!(
            relevant > irrelevant,
            "relevant={relevant:.3}, irrelevant={irrelevant:.3}"
        );
    }

    #[test]
    fn semantic_overlap_full_match() {
        let terms: Vec<String> = vec!["rust".to_string(), "safety".to_string()];
        let score = semantic_overlap("Rust provides memory safety guarantees", &terms);
        assert_eq!(score, 1.0);
    }

    #[test]
    fn semantic_overlap_partial() {
        let terms: Vec<String> = vec!["rust".to_string(), "python".to_string()];
        let score = semantic_overlap("Rust is fast and safe", &terms);
        assert_eq!(score, 0.5);
    }

    #[test]
    fn empty_query_returns_something() {
        let config = QaddConfig::default();
        let budget = config.token_budget;
        let pipeline = QaddPipeline::new(config);
        let result = pipeline.distill(SAMPLE_HTML, "");
        // With empty query, no BM25 filtering — returns content up to budget
        assert!(result.tokens_distilled <= budget + 50);
    }
}
