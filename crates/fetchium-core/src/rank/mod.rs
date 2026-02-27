//! Ranking system — BM25, HyperFusion 8-signal, semantic (PRD §21).
//!
//! Phase 1: BM25-based result reranking and URL deduplication.
//! Phase 2: HyperFusion 8-signal ranking.
//! Phase 5: Semantic/embedding-based ranking.

pub mod answer;
pub mod bm25;
pub mod cluster;
pub mod diversity;
pub mod evidence;
pub mod fusion;
pub mod quality;
pub mod signals;
pub mod snippet;
pub mod spre;
pub mod temporal;
pub mod trust;

pub use answer::{extract_answers, is_answerable_query, AnswerConfig, ExtractedAnswer};
pub use cluster::{cluster_results, ClusterConfig, ResultCluster};
pub use diversity::{diversity_score, mmr_rerank, DiversityConfig};
pub use evidence::{apply_evidence_boost, build_evidence_graph, EvidenceConfig, EvidenceGraph};
pub use quality::{assess_quality, ConfidenceLevel, QualityAssessment};
pub use snippet::{extract_best_snippets, highlight_terms, ScoredSnippet, SnippetConfig};
pub use spre::{SpreConfig, SpreRanker, SpreSnapshot};
pub use temporal::{apply_temporal_decay, decay_factor, TemporalConfig};
pub use trust::{DomainCategory, DomainTrust, TrustConfig, TrustDatabase};

use crate::types::ResultItem;
pub use bm25::{Bm25Scorer, ScoredResult, ScoringDocument};
pub use fusion::{detect_intent, hyperfusion_rank, IntentWeights, QueryIntent};
use std::collections::HashMap;
use tracing::debug;

/// Rerank a list of search results using BM25 relevance scoring.
///
/// Scores each result by its title + snippet against the query,
/// then sorts by score descending while preserving original rank
/// as a tiebreaker.
///
/// # Arguments
/// * `results` - Flat list of ResultItems from all backends
/// * `query` - The user's search query
///
/// # Returns
/// The results sorted by BM25 score (best first), with `score` field populated.
///
/// # Examples
///
/// ```rust
/// use fetchium_core::rank::rerank;
/// use fetchium_core::types::{BackendId, ResultItem};
///
/// let results = vec![
///     ResultItem {
///         title: "Rust async programming".into(),
///         url: "https://example.com/rust-async".into(),
///         snippet: "Tokio and async/await in Rust".into(),
///         rank: 0,
///         backend: BackendId::DuckDuckGo,
///         score: None,
///         published_date: None,
///     },
///     ResultItem {
///         title: "Python tutorial".into(),
///         url: "https://example.com/python".into(),
///         snippet: "Python basics and data structures".into(),
///         rank: 1,
///         backend: BackendId::DuckDuckGo,
///         score: None,
///         published_date: None,
///     },
/// ];
///
/// let ranked = rerank(results, "rust async");
/// // The Rust result should rank first
/// assert_eq!(ranked[0].url, "https://example.com/rust-async");
/// assert!(ranked[0].score.unwrap_or(0.0) > ranked[1].score.unwrap_or(0.0));
/// ```
pub fn rerank(mut results: Vec<ResultItem>, query: &str) -> Vec<ResultItem> {
    if query.is_empty() || results.is_empty() {
        return results;
    }

    for item in &mut results {
        let text = format!("{} {}", item.title, item.snippet);
        let score = bm25_score(&text, query);
        item.score = Some(score);
    }

    results.sort_by(|a, b| {
        let sa = a.score.unwrap_or(0.0);
        let sb = b.score.unwrap_or(0.0);
        sb.partial_cmp(&sa)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.rank.cmp(&b.rank))
    });

    // Re-assign ranks after sorting
    for (i, item) in results.iter_mut().enumerate() {
        item.rank = (i + 1) as u32;
    }

    debug!("Reranked {} results for query {:?}", results.len(), query);
    results
}

/// Deduplicate results by canonical URL.
///
/// Removes results whose URL (after normalization) has already been seen.
/// When duplicates exist, keeps the one with the highest BM25 score.
/// Modifies ranks to be contiguous after dedup.
pub fn deduplicate(results: Vec<ResultItem>) -> Vec<ResultItem> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut deduped: Vec<ResultItem> = Vec::with_capacity(results.len());

    for item in results {
        let key = canonical_url(&item.url);
        match seen.get(&key) {
            Some(&existing_idx) => {
                // Keep the one with higher score
                let existing_score = deduped[existing_idx].score.unwrap_or(0.0);
                let new_score = item.score.unwrap_or(0.0);
                if new_score > existing_score {
                    deduped[existing_idx] = item;
                }
            }
            None => {
                seen.insert(key, deduped.len());
                deduped.push(item);
            }
        }
    }

    // Re-assign contiguous ranks
    for (i, item) in deduped.iter_mut().enumerate() {
        item.rank = (i + 1) as u32;
    }

    deduped
}

/// BM25 relevance score for a document against a query.
///
/// Uses standard BM25 parameters: k1=1.2, b=0.75.
/// Since we're scoring individual snippets (not a full corpus),
/// IDF defaults to 1.0 for all terms.
pub fn bm25_score(text: &str, query: &str) -> f64 {
    const K1: f64 = 1.2;
    const B: f64 = 0.75;
    const AVG_DL: f64 = 80.0; // average doc length in words for web snippets

    if text.is_empty() || query.is_empty() {
        return 0.0;
    }

    let text_lower = text.to_lowercase();
    let words: Vec<&str> = text_lower.split_whitespace().collect();
    let doc_len = words.len() as f64;

    let query_terms: Vec<String> = query
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 2)
        .map(|t| t.to_string())
        .collect();

    if query_terms.is_empty() {
        return 0.0;
    }

    let mut score = 0.0;
    for term in &query_terms {
        let tf = words.iter().filter(|w| **w == term.as_str()).count() as f64;
        if tf == 0.0 {
            // Try substring match for partial term matching
            let partial_tf = words.iter().filter(|w| w.contains(term.as_str())).count() as f64;
            if partial_tf > 0.0 {
                // Partial matches get half weight
                let denom = partial_tf * 0.5 + K1 * (1.0 - B + B * doc_len / AVG_DL);
                score += 0.5 * (partial_tf * 0.5 * (K1 + 1.0)) / denom;
            }
            continue;
        }
        let numerator = tf * (K1 + 1.0);
        let denominator = tf + K1 * (1.0 - B + B * doc_len / AVG_DL);
        score += numerator / denominator; // IDF=1.0
    }

    // Normalize to [0.0, 1.0]
    let max_possible = query_terms.len() as f64 * (K1 + 1.0);
    if max_possible > 0.0 {
        (score / max_possible).min(1.0)
    } else {
        0.0
    }
}

/// Normalize a URL for deduplication purposes.
///
/// Strips: fragment, common tracking params (utm_*, fbclid, gclid),
/// trailing slash, and lowercases the host.
pub fn canonical_url(url: &str) -> String {
    // Fast path for empty/invalid URLs
    if url.is_empty() {
        return String::new();
    }

    match url::Url::parse(url) {
        Ok(mut parsed) => {
            parsed.set_fragment(None);

            // Filter out tracking query params
            const TRACKING: &[&str] = &[
                "utm_source",
                "utm_medium",
                "utm_campaign",
                "utm_term",
                "utm_content",
                "utm_id",
                "fbclid",
                "gclid",
                "msclkid",
                "ref",
                "source",
                "_ga",
                "mc_cid",
                "mc_eid",
            ];

            let filtered: Vec<(String, String)> = parsed
                .query_pairs()
                .filter(|(k, _)| !TRACKING.contains(&k.as_ref()))
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect();

            if filtered.is_empty() {
                parsed.set_query(None);
            } else {
                let qs = filtered
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<_>>()
                    .join("&");
                parsed.set_query(Some(&qs));
            }

            let mut s = parsed.to_string();
            // Strip trailing slash (but not for root domain)
            if s.ends_with('/') {
                let path = parsed.path();
                if path != "/" {
                    s.pop();
                }
            }
            s
        }
        Err(_) => url.to_lowercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BackendId;

    fn make_item(title: &str, url: &str, snippet: &str, rank: u32) -> ResultItem {
        ResultItem {
            title: title.into(),
            url: url.into(),
            snippet: snippet.into(),
            rank,
            backend: BackendId::DuckDuckGo,
            score: None,
            published_date: None,
        }
    }

    #[test]
    fn bm25_scores_relevant_higher() {
        let relevant = bm25_score(
            "Rust programming language safety systems",
            "Rust programming",
        );
        let irrelevant = bm25_score("Python web framework django flask", "Rust programming");
        assert!(
            relevant > irrelevant,
            "relevant={relevant}, irrelevant={irrelevant}"
        );
    }

    #[test]
    fn bm25_empty_inputs() {
        assert_eq!(bm25_score("", "query"), 0.0);
        assert_eq!(bm25_score("text", ""), 0.0);
        assert_eq!(bm25_score("", ""), 0.0);
    }

    #[test]
    fn bm25_score_range() {
        let score = bm25_score(
            "Rust is a great systems programming language",
            "Rust systems",
        );
        assert!((0.0..=1.0).contains(&score), "score out of range: {score}");
    }

    #[test]
    fn rerank_sorts_by_relevance() {
        let items = vec![
            make_item(
                "Python Tutorial",
                "https://python.org",
                "Learn Python programming",
                1,
            ),
            make_item(
                "Rust Book",
                "https://doc.rust-lang.org/book",
                "The Rust programming language",
                2,
            ),
            make_item(
                "Go Tour",
                "https://go.dev/tour",
                "A tour of the Go language",
                3,
            ),
        ];
        let reranked = rerank(items, "Rust programming language");
        assert_eq!(reranked[0].title, "Rust Book");
        assert!(reranked[0].score.unwrap() > reranked[1].score.unwrap_or(0.0));
    }

    #[test]
    fn rerank_updates_rank_field() {
        let items = vec![
            make_item("A", "https://a.com", "about rust", 1),
            make_item("B", "https://b.com", "rust programming", 2),
        ];
        let reranked = rerank(items, "rust");
        for (i, item) in reranked.iter().enumerate() {
            assert_eq!(item.rank, (i + 1) as u32);
        }
    }

    #[test]
    fn deduplicate_removes_tracking_params() {
        let mut items = vec![
            make_item("Page A", "https://example.com/page", "content", 1),
            make_item(
                "Page A Dup",
                "https://example.com/page?utm_source=google",
                "content",
                2,
            ),
            make_item("Page B", "https://other.com/page", "different", 3),
        ];
        items[0].score = Some(0.5);
        items[1].score = Some(0.3);
        items[2].score = Some(0.4);
        let deduped = deduplicate(items);
        assert_eq!(deduped.len(), 2);
    }

    #[test]
    fn canonical_url_strips_fragment() {
        assert!(!canonical_url("https://example.com/page#section").contains('#'));
    }

    #[test]
    fn canonical_url_strips_utm() {
        let url = "https://example.com/article?id=42&utm_source=twitter&utm_medium=social";
        let canonical = canonical_url(url);
        assert!(canonical.contains("id=42"));
        assert!(!canonical.contains("utm_source"));
        assert!(!canonical.contains("utm_medium"));
    }

    #[test]
    fn canonical_url_handles_invalid() {
        let result = canonical_url("not-a-url");
        assert!(!result.is_empty());
    }
}
