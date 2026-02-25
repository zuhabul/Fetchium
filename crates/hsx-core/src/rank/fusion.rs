//! HyperFusion 8-signal score normalization and fusion (PRD §8.1).
//!
//! Pipeline:
//! 1. Compute raw scores for all 8 signals per result
//! 2. Min-max normalize each signal across the result set to [0, 1]
//! 3. Apply intent-weighted linear combination
//! 4. Sort results by fusion score descending

use crate::rank::signals::{
    authority_score, bm25_score, consensus_score, depth_score, diversity_score, evidence_score,
    semantic_score, temporal_score, ScoringContext,
};
use crate::types::ResultItem;
use std::collections::HashMap;

/// Query intent category for weight selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryIntent {
    #[default]
    Factual,
    HowTo,
    Comparison,
    Verification,
    CurrentEvents,
    DeepAnalysis,
    Code,
    Academic,
    Opinion,
    Data,
}

/// Weights for the 8 HyperFusion signals, tuned per query intent.
#[derive(Debug, Clone)]
pub struct IntentWeights {
    pub bm25: f64,
    pub semantic: f64,
    pub temporal: f64,
    pub authority: f64,
    pub evidence: f64,
    pub diversity: f64,
    pub depth: f64,
    pub consensus: f64,
}

impl IntentWeights {
    /// Return default weights for a given intent.
    pub fn for_intent(intent: QueryIntent) -> Self {
        match intent {
            QueryIntent::Factual => Self {
                bm25: 0.20,
                semantic: 0.15,
                temporal: 0.05,
                authority: 0.20,
                evidence: 0.15,
                diversity: 0.05,
                depth: 0.05,
                consensus: 0.15,
            },
            QueryIntent::CurrentEvents => Self {
                bm25: 0.15,
                semantic: 0.10,
                temporal: 0.30,
                authority: 0.10,
                evidence: 0.10,
                diversity: 0.05,
                depth: 0.05,
                consensus: 0.15,
            },
            QueryIntent::Academic => Self {
                bm25: 0.15,
                semantic: 0.20,
                temporal: 0.05,
                authority: 0.25,
                evidence: 0.15,
                diversity: 0.05,
                depth: 0.10,
                consensus: 0.05,
            },
            QueryIntent::DeepAnalysis => Self {
                bm25: 0.10,
                semantic: 0.15,
                temporal: 0.05,
                authority: 0.10,
                evidence: 0.10,
                diversity: 0.10,
                depth: 0.30,
                consensus: 0.10,
            },
            QueryIntent::Code => Self {
                bm25: 0.25,
                semantic: 0.15,
                temporal: 0.05,
                authority: 0.15,
                evidence: 0.05,
                diversity: 0.10,
                depth: 0.15,
                consensus: 0.10,
            },
            QueryIntent::HowTo => Self {
                bm25: 0.20,
                semantic: 0.15,
                temporal: 0.10,
                authority: 0.10,
                evidence: 0.10,
                diversity: 0.10,
                depth: 0.15,
                consensus: 0.10,
            },
            QueryIntent::Comparison => Self {
                bm25: 0.15,
                semantic: 0.20,
                temporal: 0.05,
                authority: 0.15,
                evidence: 0.15,
                diversity: 0.15,
                depth: 0.10,
                consensus: 0.05,
            },
            // Balanced fallback for Verification, Opinion, Data, and any new variants
            _ => Self {
                bm25: 0.15,
                semantic: 0.15,
                temporal: 0.10,
                authority: 0.15,
                evidence: 0.10,
                diversity: 0.10,
                depth: 0.10,
                consensus: 0.15,
            },
        }
    }

    /// Apply per-signal weight overrides from config.
    pub fn with_overrides(mut self, overrides: &HashMap<String, f64>) -> Self {
        for (signal, &val) in overrides {
            let val = val.clamp(0.0, 1.0);
            match signal.as_str() {
                "bm25" => self.bm25 = val,
                "semantic" => self.semantic = val,
                "temporal" => self.temporal = val,
                "authority" => self.authority = val,
                "evidence" => self.evidence = val,
                "diversity" => self.diversity = val,
                "depth" => self.depth = val,
                "consensus" => self.consensus = val,
                _ => {}
            }
        }
        self
    }

    /// Sum of all weights (should be ~1.0 for calibrated weights).
    pub fn sum(&self) -> f64 {
        self.bm25
            + self.semantic
            + self.temporal
            + self.authority
            + self.evidence
            + self.diversity
            + self.depth
            + self.consensus
    }

    /// Normalize weights so they sum to 1.0.
    pub fn normalize(mut self) -> Self {
        let s = self.sum();
        if s > 0.0 {
            self.bm25 /= s;
            self.semantic /= s;
            self.temporal /= s;
            self.authority /= s;
            self.evidence /= s;
            self.diversity /= s;
            self.depth /= s;
            self.consensus /= s;
        }
        self
    }
}

/// Raw 8-signal scores for a single result (before normalization).
#[derive(Debug, Clone, Default)]
struct RawScores {
    bm25: f64,
    semantic: f64,
    temporal: f64,
    authority: f64,
    evidence: f64,
    diversity: f64,
    depth: f64,
    consensus: f64,
}

/// Apply HyperFusion ranking to a mutable slice of results.
///
/// Mutates `results` in-place: populates `score` field and re-sorts by fusion score.
///
/// # Arguments
/// * `results` - The result set to rank
/// * `query` - The user's query string
/// * `intent` - Detected query intent for weight selection
/// * `freshness_need` - How much recency matters (0=none, 1=high)
/// * `weight_overrides` - Optional per-signal overrides from config
pub fn hyperfusion_rank(
    results: &mut [ResultItem],
    query: &str,
    intent: QueryIntent,
    freshness_need: f64,
    weight_overrides: &HashMap<String, f64>,
) {
    if results.is_empty() {
        return;
    }

    let weights = IntentWeights::for_intent(intent)
        .with_overrides(weight_overrides)
        .normalize();

    // Build scoring context (immutable view of all results)
    let mut ctx = ScoringContext::new(query, results);

    // Phase 1: compute raw scores
    let mut raw: Vec<RawScores> = results
        .iter()
        .map(|r| {
            RawScores {
                bm25: bm25_score(r, &ctx),
                semantic: semantic_score(r, &ctx),
                temporal: temporal_score(r, freshness_need),
                authority: authority_score(r),
                evidence: evidence_score(r),
                // diversity and consensus are computed below with mutable ctx
                diversity: 0.0,
                depth: depth_score(r),
                consensus: consensus_score(r, &ctx),
            }
        })
        .collect();

    // Diversity requires sequential access (tracks seen domains)
    for (i, result) in results.iter().enumerate() {
        raw[i].diversity = diversity_score(result, &mut ctx);
    }

    // Phase 2: min-max normalize each signal across result set
    let normalized = min_max_normalize(raw);

    // Phase 3: weighted fusion
    let fusion_scores: Vec<f64> = normalized
        .iter()
        .map(|s| {
            s.bm25 * weights.bm25
                + s.semantic * weights.semantic
                + s.temporal * weights.temporal
                + s.authority * weights.authority
                + s.evidence * weights.evidence
                + s.diversity * weights.diversity
                + s.depth * weights.depth
                + s.consensus * weights.consensus
        })
        .collect();

    // Apply fusion scores to result items
    for (i, result) in results.iter_mut().enumerate() {
        result.score = Some(fusion_scores[i]);
    }

    // Phase 4: sort by fusion score descending, rank ties broken by original rank
    results.sort_by(|a, b| {
        let sa = a.score.unwrap_or(0.0);
        let sb = b.score.unwrap_or(0.0);
        sb.partial_cmp(&sa)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.rank.cmp(&b.rank))
    });

    // Re-assign ranks
    for (i, r) in results.iter_mut().enumerate() {
        r.rank = (i + 1) as u32;
    }
}

/// Min-max normalize each signal across all results.
///
/// If all values are equal (range ≈ 0), returns 0.5 for all.
fn min_max_normalize(scores: Vec<RawScores>) -> Vec<RawScores> {
    const EPSILON: f64 = 1e-9;

    macro_rules! normalize_field {
        ($scores:expr, $field:ident) => {{
            let min = $scores
                .iter()
                .map(|s| s.$field)
                .fold(f64::INFINITY, f64::min);
            let max = $scores
                .iter()
                .map(|s| s.$field)
                .fold(f64::NEG_INFINITY, f64::max);
            let range = max - min;
            if range < EPSILON {
                for s in $scores.iter_mut() {
                    s.$field = 0.5;
                }
            } else {
                for s in $scores.iter_mut() {
                    s.$field = (s.$field - min) / range;
                }
            }
        }};
    }

    let mut scores = scores;
    normalize_field!(scores, bm25);
    normalize_field!(scores, semantic);
    normalize_field!(scores, temporal);
    normalize_field!(scores, authority);
    normalize_field!(scores, evidence);
    normalize_field!(scores, diversity);
    normalize_field!(scores, depth);
    normalize_field!(scores, consensus);
    scores
}

/// Simple heuristic to detect query intent from the query string.
///
/// Used when the caller doesn't specify intent explicitly.
pub fn detect_intent(query: &str) -> QueryIntent {
    let q = query.to_lowercase();

    if q.contains("how to") || q.contains("tutorial") || q.contains("guide") || q.contains("setup")
    {
        return QueryIntent::HowTo;
    }
    if q.contains("vs ") || q.contains(" vs") || q.contains("compare") || q.contains("difference") {
        return QueryIntent::Comparison;
    }
    if q.contains("arxiv") || q.contains("paper") || q.contains("research") || q.contains("study") {
        return QueryIntent::Academic;
    }
    if q.contains("code") || q.contains("github") || q.contains("function") || q.contains("crate") {
        return QueryIntent::Code;
    }
    if q.contains("today")
        || q.contains("2025")
        || q.contains("2026")
        || q.contains("latest")
        || q.contains("news")
    {
        return QueryIntent::CurrentEvents;
    }
    if q.contains("deep") || q.contains("analysis") || q.contains("comprehensive") {
        return QueryIntent::DeepAnalysis;
    }

    QueryIntent::Factual
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BackendId;

    fn make_result(title: &str, url: &str, snippet: &str) -> ResultItem {
        ResultItem {
            title: title.into(),
            url: url.into(),
            snippet: snippet.into(),
            rank: 1,
            backend: BackendId::DuckDuckGo,
            score: None,
            published_date: None,
        }
    }

    #[test]
    fn weights_sum_to_one_after_normalize() {
        for intent in [
            QueryIntent::Factual,
            QueryIntent::CurrentEvents,
            QueryIntent::Academic,
            QueryIntent::Code,
            QueryIntent::DeepAnalysis,
        ] {
            let w = IntentWeights::for_intent(intent).normalize();
            let sum = w.sum();
            assert!((sum - 1.0).abs() < 1e-6, "intent={intent:?} sum={sum}");
        }
    }

    #[test]
    fn hyperfusion_ranks_relevant_higher() {
        let mut results = vec![
            make_result("Random Blog", "https://random-blog.xyz/post", "Some unrelated content about gardening"),
            make_result("Rust Lang", "https://rust-lang.org", "Rust is a systems programming language focused on safety, performance, and concurrency."),
            make_result("Python Docs", "https://python.org", "Python is a dynamic scripting language"),
        ];
        // Set ranks
        for (i, r) in results.iter_mut().enumerate() {
            r.rank = (i + 1) as u32;
        }

        hyperfusion_rank(
            &mut results,
            "Rust programming language",
            QueryIntent::Factual,
            0.5,
            &HashMap::new(),
        );

        // Rust result should be ranked first
        assert_eq!(
            results[0].url, "https://rust-lang.org",
            "Rust should rank first"
        );
        assert!(results[0].score.unwrap() > results[2].score.unwrap_or(0.0));
    }

    #[test]
    fn hyperfusion_empty_input() {
        let mut results = vec![];
        hyperfusion_rank(
            &mut results,
            "test",
            QueryIntent::Factual,
            0.5,
            &HashMap::new(),
        );
        assert!(results.is_empty());
    }

    #[test]
    fn all_identical_scores_use_half() {
        let scores = vec![
            RawScores {
                bm25: 0.5,
                semantic: 0.5,
                temporal: 0.5,
                authority: 0.5,
                evidence: 0.5,
                diversity: 0.5,
                depth: 0.5,
                consensus: 0.5,
            },
            RawScores {
                bm25: 0.5,
                semantic: 0.5,
                temporal: 0.5,
                authority: 0.5,
                evidence: 0.5,
                diversity: 0.5,
                depth: 0.5,
                consensus: 0.5,
            },
        ];
        let normalized = min_max_normalize(scores);
        assert!((normalized[0].bm25 - 0.5).abs() < 1e-9);
    }

    #[test]
    fn min_max_normalize_correct() {
        let scores = vec![
            RawScores {
                bm25: 0.0,
                semantic: 0.0,
                temporal: 0.0,
                authority: 0.0,
                evidence: 0.0,
                diversity: 0.0,
                depth: 0.0,
                consensus: 0.0,
            },
            RawScores {
                bm25: 1.0,
                semantic: 1.0,
                temporal: 1.0,
                authority: 1.0,
                evidence: 1.0,
                diversity: 1.0,
                depth: 1.0,
                consensus: 1.0,
            },
        ];
        let normalized = min_max_normalize(scores);
        assert!((normalized[0].bm25 - 0.0).abs() < 1e-9);
        assert!((normalized[1].bm25 - 1.0).abs() < 1e-9);
    }

    #[test]
    fn detect_intent_works() {
        assert_eq!(detect_intent("how to install Rust"), QueryIntent::HowTo);
        assert_eq!(
            detect_intent("Rust vs Go performance"),
            QueryIntent::Comparison
        );
        assert_eq!(
            detect_intent("arxiv paper transformer attention"),
            QueryIntent::Academic
        );
        assert_eq!(detect_intent("github rust async code"), QueryIntent::Code);
        assert_eq!(
            detect_intent("Rust latest news 2026"),
            QueryIntent::CurrentEvents
        );
    }

    #[test]
    fn weight_overrides_applied() {
        let weights = IntentWeights::for_intent(QueryIntent::Factual)
            .with_overrides(&[("temporal".to_string(), 0.9f64)].into_iter().collect());
        assert!((weights.temporal - 0.9).abs() < 1e-9);
    }

    #[test]
    fn hyperfusion_populates_scores() {
        let mut results = vec![
            make_result("A", "https://a.com", "content A about systems programming"),
            make_result("B", "https://b.com", "content B about databases"),
        ];
        results[0].rank = 1;
        results[1].rank = 2;
        hyperfusion_rank(
            &mut results,
            "systems",
            QueryIntent::Factual,
            0.5,
            &HashMap::new(),
        );
        assert!(results[0].score.is_some());
        assert!(results[1].score.is_some());
    }
}
