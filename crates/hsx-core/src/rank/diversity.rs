//! Result Diversity Optimizer (RDO) — Maximal Marginal Relevance reranking.
//!
//! MMR ensures users see varied results rather than 10 pages about the same
//! subtopic. At each selection step, it balances relevance to the query against
//! redundancy with already-selected results.
//!
//! The core formula:
//! ```text
//! MMR(r) = λ · relevance(r) − (1−λ) · max_sim(r, selected)
//! ```

use std::collections::HashSet;

use crate::rank::bm25_score;
use crate::types::ResultItem;

/// Configuration for the diversity optimizer.
#[derive(Debug, Clone)]
pub struct DiversityConfig {
    /// Trade-off between relevance and diversity (0.0–1.0).
    /// Higher values favor relevance; lower values favor diversity.
    pub lambda: f64,
    /// Maximum number of results to return.
    pub max_results: usize,
    /// Jaccard similarity above which two results are considered "too similar".
    pub similarity_threshold: f64,
}

impl Default for DiversityConfig {
    fn default() -> Self {
        Self {
            lambda: 0.7,
            max_results: 20,
            similarity_threshold: 0.8,
        }
    }
}

/// Rerank results using Maximal Marginal Relevance.
///
/// At each step, picks the candidate that maximizes:
/// `lambda * relevance(r) - (1 - lambda) * max_similarity(r, selected)`
///
/// Uses the result's existing `score` when present, falling back to BM25
/// scoring against `query` otherwise. Inter-result similarity is measured
/// with Jaccard over term sets extracted from title + snippet.
pub fn mmr_rerank(
    results: &[ResultItem],
    query: &str,
    config: &DiversityConfig,
) -> Vec<ResultItem> {
    if results.is_empty() {
        return vec![];
    }

    let n = results.len();
    let limit = config.max_results.min(n);

    // Pre-compute relevance scores and term sets.
    let relevances: Vec<f64> = results
        .iter()
        .map(|r| {
            r.score.unwrap_or_else(|| {
                let text = format!("{} {}", r.title, r.snippet);
                bm25_score(&text, query)
            })
        })
        .collect();

    let term_sets: Vec<HashSet<String>> = results.iter().map(term_set).collect();

    // Track which indices have been selected vs. remaining.
    let mut selected_indices: Vec<usize> = Vec::with_capacity(limit);
    let mut remaining: Vec<usize> = (0..n).collect();

    // Seed: pick the highest-relevance result.
    let seed = remaining
        .iter()
        .copied()
        .max_by(|&a, &b| {
            relevances[a]
                .partial_cmp(&relevances[b])
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("remaining is non-empty");

    selected_indices.push(seed);
    remaining.retain(|&i| i != seed);

    // Iteratively select the result with the highest MMR score.
    while selected_indices.len() < limit && !remaining.is_empty() {
        let mut best_idx = remaining[0];
        let mut best_mmr = f64::NEG_INFINITY;

        for &candidate in &remaining {
            let rel = relevances[candidate];

            // Maximum similarity to any already-selected result.
            let max_sim = selected_indices
                .iter()
                .map(|&s| jaccard(&term_sets[candidate], &term_sets[s]))
                .fold(0.0_f64, f64::max);

            let mmr = config.lambda * rel - (1.0 - config.lambda) * max_sim;

            if mmr > best_mmr {
                best_mmr = mmr;
                best_idx = candidate;
            }
        }

        selected_indices.push(best_idx);
        remaining.retain(|&i| i != best_idx);
    }

    // Build output with updated ranks.
    selected_indices
        .iter()
        .enumerate()
        .map(|(rank, &orig)| {
            let mut item = results[orig].clone();
            item.rank = (rank + 1) as u32;
            item.score = Some(relevances[orig]);
            item
        })
        .collect()
}

/// Measure the diversity of a result set.
///
/// Returns a value in `[0.0, 1.0]` where 0.0 means all results are
/// identical and 1.0 means every pair is completely unique.
///
/// Computed as `1.0 − mean(pairwise Jaccard similarities)`.
pub fn diversity_score(results: &[ResultItem]) -> f64 {
    if results.len() <= 1 {
        return 1.0;
    }

    let term_sets: Vec<HashSet<String>> = results.iter().map(term_set).collect();
    let n = term_sets.len();
    let mut total_sim = 0.0;
    let mut pair_count = 0u64;

    for i in 0..n {
        for j in (i + 1)..n {
            total_sim += jaccard(&term_sets[i], &term_sets[j]);
            pair_count += 1;
        }
    }

    if pair_count == 0 {
        return 1.0;
    }

    let mean_sim = total_sim / pair_count as f64;
    1.0 - mean_sim
}

// ─── Internal helpers ──────────────────────────────────────────

/// Extract lowercased terms (length >= 3, no stop-words) from title + snippet.
fn term_set(item: &ResultItem) -> HashSet<String> {
    let text = format!("{} {}", item.title, item.snippet).to_lowercase();
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 3)
        .filter(|t| !is_stop_word(t))
        .map(|t| t.to_string())
        .collect()
}

/// Jaccard similarity between two term sets.
fn jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let intersection = a.intersection(b).count();
    let union = a.union(b).count();
    if union == 0 {
        return 0.0;
    }
    intersection as f64 / union as f64
}

/// Common English stop-words excluded from term sets.
fn is_stop_word(word: &str) -> bool {
    const STOP: &[&str] = &[
        "the", "and", "for", "are", "but", "not", "you", "all", "can", "her", "was", "one", "our",
        "out", "has", "had", "how", "its", "may", "new", "now", "old", "see", "way", "who", "did",
        "get", "let", "say", "she", "too", "use", "this", "that", "with", "have", "from", "they",
        "been", "some", "when", "what", "your", "each", "make", "like", "just", "than", "them",
        "very", "will", "more", "also",
    ];
    STOP.contains(&word)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BackendId;

    fn make_item(title: &str, snippet: &str, score: Option<f64>) -> ResultItem {
        ResultItem {
            title: title.into(),
            url: format!(
                "https://{}.example.com",
                title.to_lowercase().replace(' ', "-")
            ),
            snippet: snippet.into(),
            rank: 0,
            backend: BackendId::DuckDuckGo,
            score,
            published_date: None,
        }
    }

    #[test]
    fn empty_input_returns_empty() {
        let out = mmr_rerank(&[], "anything", &DiversityConfig::default());
        assert!(out.is_empty());
    }

    #[test]
    fn single_result_returned_as_is() {
        let items = vec![make_item("Rust Guide", "Learn Rust programming", Some(0.9))];
        let out = mmr_rerank(&items, "rust", &DiversityConfig::default());
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].title, "Rust Guide");
        assert_eq!(out[0].rank, 1);
    }

    #[test]
    fn diverse_results_preserved() {
        let items = vec![
            make_item(
                "Rust Programming",
                "Systems language with safety guarantees",
                Some(0.8),
            ),
            make_item(
                "Python Data Science",
                "Pandas and NumPy for analysis",
                Some(0.7),
            ),
            make_item(
                "Go Concurrency",
                "Goroutines and channels for parallelism",
                Some(0.6),
            ),
        ];
        let out = mmr_rerank(&items, "programming languages", &DiversityConfig::default());
        assert_eq!(out.len(), 3);
        // All three topics should appear since they are diverse.
        let titles: Vec<&str> = out.iter().map(|r| r.title.as_str()).collect();
        assert!(titles.contains(&"Rust Programming"));
        assert!(titles.contains(&"Python Data Science"));
        assert!(titles.contains(&"Go Concurrency"));
    }

    #[test]
    fn duplicate_results_deprioritized_by_mmr() {
        // Two near-identical Rust results and one Python result.
        let items = vec![
            make_item(
                "Rust Safety",
                "Rust memory safety borrow checker",
                Some(0.9),
            ),
            make_item(
                "Rust Safety Guide",
                "Rust memory safety borrow checker explained",
                Some(0.85),
            ),
            make_item("Python Intro", "Python beginner tutorial basics", Some(0.5)),
        ];
        let config = DiversityConfig {
            lambda: 0.5,
            ..DiversityConfig::default()
        };
        let out = mmr_rerank(&items, "programming", &config);
        assert_eq!(out.len(), 3);
        // With balanced lambda, Python should not be last because it provides diversity.
        // The second Rust result (near-duplicate) should be pushed down.
        let last = &out[2];
        assert_eq!(last.title, "Rust Safety Guide");
    }

    #[test]
    fn lambda_one_is_pure_relevance() {
        let items = vec![
            make_item("Low Score", "unrelated content", Some(0.1)),
            make_item("High Score", "very relevant document", Some(0.9)),
            make_item("Mid Score", "somewhat related", Some(0.5)),
        ];
        let config = DiversityConfig {
            lambda: 1.0,
            ..DiversityConfig::default()
        };
        let out = mmr_rerank(&items, "relevant", &config);
        // Pure relevance: should be sorted by score descending.
        assert_eq!(out[0].title, "High Score");
        assert_eq!(out[1].title, "Mid Score");
        assert_eq!(out[2].title, "Low Score");
    }

    #[test]
    fn lambda_zero_is_pure_diversity() {
        // Two identical results and one different result.
        let items = vec![
            make_item(
                "Rust Safety",
                "Rust borrow checker memory safety",
                Some(0.9),
            ),
            make_item(
                "Rust Safety Copy",
                "Rust borrow checker memory safety",
                Some(0.85),
            ),
            make_item("Python Web", "Django Flask web framework", Some(0.3)),
        ];
        let config = DiversityConfig {
            lambda: 0.0,
            ..DiversityConfig::default()
        };
        let out = mmr_rerank(&items, "programming", &config);
        // Seed is still the highest-scoring result. After that, the most
        // diverse candidate (Python) should be chosen before the duplicate.
        assert_eq!(out[0].title, "Rust Safety");
        assert_eq!(out[1].title, "Python Web");
        assert_eq!(out[2].title, "Rust Safety Copy");
    }

    #[test]
    fn diversity_score_unique_results_near_one() {
        let items = vec![
            make_item("Rust Programming", "Systems language safety", None),
            make_item("Python Data Science", "Pandas NumPy analysis", None),
            make_item("Go Networking", "HTTP servers goroutines", None),
        ];
        let score = diversity_score(&items);
        assert!(
            score > 0.8,
            "Expected diversity > 0.8 for unique results, got {score}"
        );
    }

    #[test]
    fn diversity_score_identical_results_near_zero() {
        let items = vec![
            make_item("Rust Safety", "Rust borrow checker memory safety", None),
            make_item("Rust Safety", "Rust borrow checker memory safety", None),
            make_item("Rust Safety", "Rust borrow checker memory safety", None),
        ];
        let score = diversity_score(&items);
        assert!(
            score < 0.01,
            "Expected diversity < 0.01 for identical results, got {score}"
        );
    }

    #[test]
    fn max_results_respected() {
        let items: Vec<ResultItem> = (0..10)
            .map(|i| {
                make_item(
                    &format!("Topic {i}"),
                    &format!("Unique content about subject number {i}"),
                    Some(1.0 - i as f64 * 0.05),
                )
            })
            .collect();
        let config = DiversityConfig {
            max_results: 3,
            ..DiversityConfig::default()
        };
        let out = mmr_rerank(&items, "topic", &config);
        assert_eq!(out.len(), 3);
        // Ranks should be 1, 2, 3.
        for (i, r) in out.iter().enumerate() {
            assert_eq!(r.rank, (i + 1) as u32);
        }
    }
}
