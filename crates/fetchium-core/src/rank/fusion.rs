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
    /// Definitional / explanatory queries ("what is X", "explain Y", "how does Z work").
    ///
    /// Prioritizes BM25 text relevance and Wikipedia/educational sources over
    /// GitHub repositories, which are almost never useful for definitions.
    Informational,
    HowTo,
    Comparison,
    Verification,
    CurrentEvents,
    DeepAnalysis,
    Code,
    Academic,
    Opinion,
    Data,
    /// Everyday casual queries: "best pizza near me", "coffee stains shirt",
    /// "cheap flights to paris", "signs of burnout". These should skip all
    /// specialist backends (ArXiv, Scholar, GitHub, StackOverflow) and rely
    /// on general web search + Reddit for real-world answers.
    Casual,
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
                bm25: 0.30,
                semantic: 0.15,
                temporal: 0.05,
                authority: 0.10,
                evidence: 0.15,
                diversity: 0.05,
                depth: 0.05,
                consensus: 0.15,
            },
            QueryIntent::Informational => Self {
                bm25: 0.40,
                semantic: 0.20,
                temporal: 0.05,
                authority: 0.03,
                evidence: 0.10,
                diversity: 0.07,
                depth: 0.10,
                consensus: 0.05,
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
                bm25: 0.35,
                semantic: 0.15,
                temporal: 0.05,
                authority: 0.05,
                evidence: 0.05,
                diversity: 0.10,
                depth: 0.15,
                consensus: 0.10,
            },
            QueryIntent::HowTo => Self {
                bm25: 0.35,
                semantic: 0.15,
                temporal: 0.05,
                authority: 0.05,
                evidence: 0.10,
                diversity: 0.10,
                depth: 0.15,
                consensus: 0.05,
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
            QueryIntent::Opinion => Self {
                bm25: 0.20,
                semantic: 0.10,
                temporal: 0.05,
                authority: 0.10,
                evidence: 0.10,
                diversity: 0.15,
                depth: 0.10,
                consensus: 0.20,
            },
            QueryIntent::Casual => Self {
                bm25: 0.35,
                semantic: 0.15,
                temporal: 0.05,
                authority: 0.05,
                evidence: 0.05,
                diversity: 0.15,
                depth: 0.05,
                consensus: 0.15,
            },
            // Balanced fallback for Verification, Data, and any new variants
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

/// Adjust raw authority score based on query intent to fix over-ranking biases.
///
/// GitHub.com has domain authority 0.90 (same tier as Wikipedia), which is correct
/// for code queries but causes irrelevant repositories to rank above educational pages
/// for informational/factual queries.
///
/// Wikipedia has high authority which is correct for factual/informational queries
/// but causes generic articles to dominate temporal/current-events queries where
/// news sources should rank higher.
fn adjust_authority_for_intent(url: &str, base: f64, intent: QueryIntent) -> f64 {
    // GitHub bias: repos only relevant for code queries
    if url.contains("github.com/") {
        return match intent {
            QueryIntent::Code => base,               // repos are the right result
            QueryIntent::Academic => base * 0.6,     // some papers reference repos
            QueryIntent::DeepAnalysis => base * 0.7, // deep dives sometimes cite repos
            QueryIntent::Casual => base * 0.2,       // repos never useful for casual queries
            _ => base * 0.45,                        // informational/factual: heavily downweight
        };
    }

    // Wikipedia bias: great for definitions, poor for "breakthroughs 2025" type queries
    if url.contains("wikipedia.org") {
        // Extra penalty for "List of..." and biography pages that are usually off-topic
        let path_lower = url.to_lowercase();
        let is_list_or_bio = path_lower.contains("list_of_")
            || path_lower.contains("(academic)")
            || path_lower.contains("(politician)")
            || path_lower.contains("(footballer)")
            || path_lower.contains("(singer)")
            || path_lower.contains("(actor)");
        let list_penalty = if is_list_or_bio { 0.5 } else { 1.0 };

        return match intent {
            QueryIntent::CurrentEvents => base * 0.35 * list_penalty,
            QueryIntent::Casual => base * 0.50 * list_penalty, // Wikipedia generic articles drown casual results
            QueryIntent::Factual | QueryIntent::Informational | QueryIntent::Verification => {
                base * list_penalty
            }
            _ => base * 0.75 * list_penalty,
        };
    }

    // Reddit/forum bias: great for opinions, but comparison/technical queries need
    // structured articles over discussion threads.
    if url.contains("reddit.com") || url.contains("news.ycombinator.com") {
        return match intent {
            QueryIntent::Opinion => base,         // discussions add perspective for opinion queries
            QueryIntent::Casual => base * 0.55,    // prefer authoritative consumer sites over Reddit threads
            QueryIntent::Comparison => base * 0.55,             // prefer articles over threads
            QueryIntent::Code | QueryIntent::HowTo => base * 0.60, // prefer docs/tutorials
            QueryIntent::Academic => base * 0.40,               // not a good source
            _ => base * 0.70,
        };
    }

    // YouTube/video results: deprioritize for comparison/technical where text is better
    if url.contains("youtube.com") || url.contains("youtu.be") {
        return match intent {
            QueryIntent::HowTo => base * 0.80, // tutorials on YouTube can be good
            QueryIntent::Opinion => base * 0.85,
            QueryIntent::Comparison => base * 0.50, // articles much better than videos
            QueryIntent::Code => base * 0.45,       // text tutorials > video
            QueryIntent::Academic => base * 0.35,
            QueryIntent::Casual => base * 0.90,
            _ => base * 0.65,
        };
    }

    // ArXiv bias: excellent for academic/verification, often off-topic for
    // opinion/comparison "best X" queries where community benchmarks matter more.
    if url.contains("arxiv.org/") {
        return match intent {
            QueryIntent::Academic | QueryIntent::Verification | QueryIntent::Data => base,
            QueryIntent::Opinion | QueryIntent::Comparison => base * 0.45,
            QueryIntent::Casual => base * 0.15, // ArXiv never relevant for casual queries
            _ => base * 0.75,
        };
    }

    base
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
            let base_authority = authority_score(r);
            RawScores {
                bm25: bm25_score(r, &ctx),
                semantic: semantic_score(r, &ctx),
                temporal: temporal_score(r, freshness_need),
                // Apply intent-based adjustment: downweight GitHub repos for non-code queries
                authority: adjust_authority_for_intent(&r.url, base_authority, intent),
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

    // Apply power-law spread to widen the score distribution.
    // Raw fusion scores cluster in [0.3, 0.7] due to min-max normalization.
    // Spread them across [0.0, 1.0] for more discriminating ranking.
    let fmin = fusion_scores.iter().cloned().fold(f64::INFINITY, f64::min);
    let fmax = fusion_scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let frange = fmax - fmin;
    let spread_scores: Vec<f64> = if frange > 1e-9 {
        fusion_scores
            .iter()
            .map(|&s| {
                let normalized = (s - fmin) / frange; // [0, 1]
                // Apply power curve: pow(x, 0.6) spreads the top end
                normalized.powf(0.6)
            })
            .collect()
    } else {
        fusion_scores.iter().map(|_| 0.5).collect()
    };

    // Apply spread fusion scores to result items
    for (i, result) in results.iter_mut().enumerate() {
        result.score = Some(spread_scores[i]);
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
    let has_recent_year = q.contains("2025") || q.contains("2026");
    let has_news_signal = q.contains("today")
        || q.contains("latest")
        || q.contains("news")
        || q.contains("recent")
        || q.contains("this year")
        || q.contains("this month")
        || q.contains("this week")
        || q.contains("breaking")
        || q.contains("update")
        || q.contains("announcement")
        || q.contains("released")
        || q.contains("launched")
        || q.contains("new in ");
    let is_technical_eval = q.contains("benchmark")
        || q.contains("performance")
        || q.contains("vs ")
        || q.contains(" vs")
        || q.contains("compare")
        || q.contains("runtime")
        || q.contains("framework")
        || q.contains("latency")
        || q.contains("throughput");
    let is_preference_query = q.contains("best ")
        || q.starts_with("best ")
        || q.contains("top ")
        || q.contains("recommended")
        || q.contains("which is better")
        || q.contains("should i use")
        || q.contains("worth it")
        || q.contains("pros and cons");
    let has_technical_signal = q.contains("protocol")
        || q.contains("architecture")
        || q.contains("theorem")
        || q.contains("equation")
        || q.contains("compiler")
        || q.contains("kernel")
        || q.contains("database")
        || q.contains("encryption")
        || q.contains("neural")
        || q.contains("quantum")
        || q.contains("molecular")
        || q.contains("genome")
        || q.contains("proof")
        || q.contains("axiom")
        || q.contains("cryptograph");

    // Code intent: explicit code/repo signals
    // Note: "tutorial" and "example" with programming languages → HowTo (not Code)
    // because users want articles/docs, not GitHub repos
    let is_programming_lang = q.contains("python")
        || q.contains("rust")
        || q.contains("javascript")
        || q.contains("typescript")
        || q.contains("java ")
        || q.contains("golang")
        || q.contains("go ")
        || q.contains("ruby")
        || q.contains("swift")
        || q.contains("kotlin")
        || q.contains("c++")
        || q.contains("c#");
    let has_tutorial_signal = q.contains("tutorial")
        || q.contains("guide")
        || q.contains("how to")
        || q.contains("example");
    if q.contains("code")
        || q.contains("github")
        || q.contains("function")
        || q.contains("crate")
        || q.contains("snippet")
        || q.contains("implement")
        || q.contains("algorithm")
        || (is_programming_lang
            && !has_tutorial_signal
            && (q.contains("library")
                || q.contains("package")
                || q.contains("crate")))
    {
        return QueryIntent::Code;
    }

    // CurrentEvents: temporal signals — checked early so "breakthroughs 2025" doesn't
    // fall through to Factual. Year patterns, recency keywords, and news signals.
    if has_news_signal
        || (has_recent_year && !is_technical_eval && !is_preference_query)
        || (q.contains("breakthrough") && !is_technical_eval)
    {
        return QueryIntent::CurrentEvents;
    }

    // HowTo: procedural instructions
    if q.contains("how to")
        || q.contains("tutorial")
        || q.contains("guide")
        || q.contains("setup")
        || q.contains("install")
        || q.contains("configure")
    {
        return QueryIntent::HowTo;
    }

    // Informational: definitional / explanatory — detected BEFORE generic Factual fallback
    if q.starts_with("what is")
        || q.starts_with("what are")
        || q.starts_with("what does")
        || q.starts_with("who is")
        || q.starts_with("who was")
        || q.starts_with("where is")
        || q.starts_with("when was")
        || q.starts_with("when did")
        || q.contains("how does")
        || q.contains("how do ")
        || q.contains("define ")
        || q.contains("definition of")
        || q.contains("definition:")
        || q.contains("explain ")
        || q.contains("meaning of")
        || q.contains(" is a ")
        || q.contains(" is an ")
    {
        return QueryIntent::Informational;
    }

    // Casual consumer queries: "best pizza in new york", "cheap flights to paris",
    // "best coffee shop near me". These look like Opinion ("best X") but are everyday
    // consumer searches that should prefer review sites, travel sites, etc. over Reddit.
    let is_consumer_query = q.contains("pizza")
        || q.contains("restaurant")
        || q.contains("hotel")
        || q.contains("flight")
        || q.contains("coffee")
        || q.contains("recipe")
        || q.contains("movie")
        || q.contains("near me")
        || q.contains("in new ")
        || q.contains("in los ")
        || q.contains("in san ")
        || q.contains("in chicago")
        || q.contains("in london")
        || q.contains("in paris")
        || q.contains("in tokyo")
        || q.contains("shop")
        || q.contains("store")
        || q.contains("gym")
        || q.contains("barber")
        || q.contains("salon")
        || q.contains("dentist")
        || q.contains("plumber");
    if is_preference_query && is_consumer_query && !is_programming_lang && !has_technical_signal {
        return QueryIntent::Casual;
    }

    // Opinion: "best X", "top X", "recommended", "should I use"
    if is_preference_query {
        return QueryIntent::Opinion;
    }

    if q.contains("vs ") || q.contains(" vs") || q.contains("compare") || q.contains("difference") {
        return QueryIntent::Comparison;
    }
    if q.contains("arxiv") || q.contains("paper") || q.contains("research") || q.contains("study") {
        return QueryIntent::Academic;
    }
    // DeepAnalysis: explicit analytical intent (but NOT "deep dive" which is just emphasis)
    if q.contains("analysis") || q.contains("comprehensive") {
        return QueryIntent::DeepAnalysis;
    }
    // "deep dive" is a common phrase meaning "thorough article", not DeepAnalysis.
    // Route to HowTo for "X deep dive" patterns, which performs better than DeepAnalysis
    // for technical explainers like "kubernetes pod networking deep dive".
    if q.contains("deep dive") || q.contains("deep-dive") {
        return QueryIntent::HowTo;
    }
    if q.contains("deep") && !q.contains("deep learning") {
        return QueryIntent::DeepAnalysis;
    }

    // Casual: everyday queries that don't match any specialist intent.
    // Short queries (≤6 words) without technical/academic signals are likely casual.
    // Examples: "best pizza near me", "coffee stains shirt", "signs of burnout"
    let word_count = q.split_whitespace().count();
    if word_count <= 6 && !has_technical_signal {
        return QueryIntent::Casual;
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
            QueryIntent::Informational,
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
    fn detect_intent_temporal_keywords() {
        assert_eq!(
            detect_intent("quantum computing breakthroughs 2025"),
            QueryIntent::CurrentEvents
        );
        assert_eq!(
            detect_intent("recent advances in AI"),
            QueryIntent::CurrentEvents
        );
        assert_eq!(
            detect_intent("breaking news AI regulation"),
            QueryIntent::CurrentEvents
        );
        assert_eq!(
            detect_intent("new in rust this year"),
            QueryIntent::CurrentEvents
        );
        assert_eq!(
            detect_intent("GPT-5 announcement"),
            QueryIntent::CurrentEvents
        );
    }

    #[test]
    fn detect_intent_technical_year_query_not_forced_to_current_events() {
        let intent =
            detect_intent("best rust async runtime for high-throughput web api in 2025 benchmark");
        assert_ne!(intent, QueryIntent::CurrentEvents);
    }

    #[test]
    fn detect_intent_consumer_best_year_query_prefers_opinion() {
        let intent = detect_intent("best noise cancelling headphones for office calls 2026");
        assert_eq!(intent, QueryIntent::Opinion);
    }

    #[test]
    fn wikipedia_authority_capped_for_temporal() {
        let wiki_result = make_result(
            "Quantum computing",
            "https://en.wikipedia.org/wiki/Quantum_computing",
            "Quantum computing is the exploitation of collective properties...",
        );
        let base_auth = 0.9_f64;
        let events_auth =
            adjust_authority_for_intent(&wiki_result.url, base_auth, QueryIntent::CurrentEvents);
        let factual_auth =
            adjust_authority_for_intent(&wiki_result.url, base_auth, QueryIntent::Factual);
        assert!(
            events_auth < 0.4,
            "Wikipedia authority for CurrentEvents should be < 0.4, got {events_auth}"
        );
        assert!(
            (factual_auth - base_auth).abs() < 1e-9,
            "Wikipedia authority for Factual should be unchanged"
        );
    }

    #[test]
    fn detect_intent_informational() {
        assert_eq!(
            detect_intent("what is artificial intelligence"),
            QueryIntent::Informational
        );
        assert_eq!(detect_intent("what is ai"), QueryIntent::Informational);
        assert_eq!(
            detect_intent("what are neural networks"),
            QueryIntent::Informational
        );
        assert_eq!(
            detect_intent("who is Alan Turing"),
            QueryIntent::Informational
        );
        assert_eq!(
            detect_intent("explain machine learning"),
            QueryIntent::Informational
        );
        assert_eq!(
            detect_intent("define deep learning"),
            QueryIntent::Informational
        );
        assert_eq!(
            detect_intent("how does TCP work"),
            QueryIntent::Informational
        );
        assert_eq!(
            detect_intent("meaning of recursion"),
            QueryIntent::Informational
        );
    }

    #[test]
    fn informational_authority_lower_than_code_for_github() {
        let github_result = make_result(
            "hand_detection",
            "https://github.com/user/hand_detection",
            "A popular ML repository for hand detection",
        );
        let weights_info = IntentWeights::for_intent(QueryIntent::Informational);
        let weights_code = IntentWeights::for_intent(QueryIntent::Code);
        // Informational gives much less weight to authority than Code
        assert!(
            weights_info.authority < weights_code.authority,
            "Informational authority weight ({}) should be < Code ({})",
            weights_info.authority,
            weights_code.authority
        );
        // GitHub authority should be downscaled for Informational
        let base_auth = 0.9_f64; // GitHub domain tier
        let info_auth =
            adjust_authority_for_intent(&github_result.url, base_auth, QueryIntent::Informational);
        let code_auth =
            adjust_authority_for_intent(&github_result.url, base_auth, QueryIntent::Code);
        assert!(
            info_auth < code_auth,
            "GitHub info_auth ({info_auth}) should be < code_auth ({code_auth})"
        );
        assert!(
            info_auth < 0.5,
            "GitHub authority for informational queries should be < 0.5, got {info_auth}"
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
