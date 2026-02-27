//! Query Complexity Estimator (QCE) — estimates query difficulty to adjust search strategy.
//!
//! Simple factual queries ("capital of France") get fast single-backend lookups.
//! Complex queries ("compare Rust async frameworks performance benchmarks 2024")
//! trigger multi-backend, expansion, and deeper ranking.
//!
//! Scoring signals:
//! 1. Term count — more terms = more complex
//! 2. Operator presence — quotes, site:, filetype:, boolean operators
//! 3. Question words — "how", "why", "compare" indicate depth
//! 4. Entity count — named entities increase complexity
//! 5. Temporal markers — "latest", "2024", "recent" add complexity

use crate::rank::fusion::QueryIntent;

/// Configuration for the complexity estimator.
#[derive(Debug, Clone)]
pub struct ComplexityConfig {
    /// Below this threshold: simple query (single backend, no expansion).
    pub simple_threshold: f64,
    /// Above this threshold: complex query (multi-backend, expansion, deep ranking).
    pub complex_threshold: f64,
}

impl Default for ComplexityConfig {
    fn default() -> Self {
        Self {
            simple_threshold: 0.3,
            complex_threshold: 0.7,
        }
    }
}

/// Complexity assessment result.
#[derive(Debug, Clone)]
pub struct ComplexityAssessment {
    /// Overall complexity score in [0.0, 1.0].
    pub score: f64,
    /// Human-readable complexity level.
    pub level: ComplexityLevel,
    /// Recommended number of backends to query.
    pub recommended_backends: usize,
    /// Whether query expansion should be applied.
    pub should_expand: bool,
    /// Whether deep ranking (HyperFusion) is worthwhile.
    pub needs_deep_ranking: bool,
    /// Individual signal scores for debugging.
    pub signals: ComplexitySignals,
}

/// Complexity level categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplexityLevel {
    /// Quick factual lookup — 1 backend, no expansion.
    Simple,
    /// Moderate query — 2-3 backends, optional expansion.
    Moderate,
    /// Deep analysis — all backends, expansion, full ranking.
    Complex,
}

/// Individual signal contributions.
#[derive(Debug, Clone)]
pub struct ComplexitySignals {
    pub term_count_signal: f64,
    pub operator_signal: f64,
    pub question_depth_signal: f64,
    pub entity_signal: f64,
    pub temporal_signal: f64,
}

/// Estimate the complexity of a search query.
pub fn estimate_complexity(query: &str, config: &ComplexityConfig) -> ComplexityAssessment {
    let signals = compute_signals(query);

    // Weighted combination
    let score = (signals.term_count_signal * 0.25
        + signals.operator_signal * 0.25
        + signals.question_depth_signal * 0.25
        + signals.entity_signal * 0.15
        + signals.temporal_signal * 0.10)
        .clamp(0.0, 1.0);

    let level = if score < config.simple_threshold {
        ComplexityLevel::Simple
    } else if score < config.complex_threshold {
        ComplexityLevel::Moderate
    } else {
        ComplexityLevel::Complex
    };

    let recommended_backends = match level {
        ComplexityLevel::Simple => 1,
        ComplexityLevel::Moderate => 3,
        ComplexityLevel::Complex => 5,
    };

    ComplexityAssessment {
        score,
        level,
        recommended_backends,
        should_expand: score >= config.simple_threshold,
        needs_deep_ranking: score >= config.complex_threshold,
        signals,
    }
}

/// Suggest a query intent based on complexity signals.
pub fn suggest_intent(query: &str) -> QueryIntent {
    let lower = query.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();

    if words.iter().any(|w| COMPARISON_WORDS.contains(w)) || lower.contains(" vs ") {
        return QueryIntent::Comparison;
    }
    if words.iter().any(|w| TEMPORAL_WORDS.contains(w)) || has_year(&lower) {
        return QueryIntent::CurrentEvents;
    }
    if words.first().is_some_and(|w| HOW_WORDS.contains(w)) {
        return QueryIntent::HowTo;
    }
    if words
        .first()
        .is_some_and(|w| *w == "is" || *w == "are" || *w == "does" || *w == "do")
    {
        return QueryIntent::Verification;
    }
    if words.iter().any(|w| CODE_WORDS.contains(w)) {
        return QueryIntent::Code;
    }
    if words.iter().any(|w| ACADEMIC_WORDS.contains(w)) {
        return QueryIntent::Academic;
    }
    if words.iter().any(|w| OPINION_WORDS.contains(w)) {
        return QueryIntent::Opinion;
    }
    if words.iter().any(|w| DATA_WORDS.contains(w)) {
        return QueryIntent::Data;
    }
    if words.len() > 6 {
        return QueryIntent::DeepAnalysis;
    }
    QueryIntent::Factual
}

// ─── Signal computation ──────────────────────────────────────

fn compute_signals(query: &str) -> ComplexitySignals {
    let lower = query.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();

    ComplexitySignals {
        term_count_signal: term_count_signal(words.len()),
        operator_signal: operator_signal(&lower),
        question_depth_signal: question_depth_signal(&words),
        entity_signal: entity_signal(&words),
        temporal_signal: temporal_signal(&lower),
    }
}

/// More terms → higher complexity. Sigmoid-like curve: 1 word = 0.0, 8+ words = 1.0.
fn term_count_signal(count: usize) -> f64 {
    match count {
        0..=1 => 0.0,
        2 => 0.15,
        3 => 0.3,
        4 => 0.45,
        5 => 0.6,
        6 => 0.75,
        7 => 0.9,
        _ => 1.0,
    }
}

/// Detect advanced search operators: quotes, site:, filetype:, boolean.
fn operator_signal(query: &str) -> f64 {
    let mut score: f64 = 0.0;
    if query.contains('"') {
        score += 0.3;
    }
    if query.contains("site:") {
        score += 0.3;
    }
    if query.contains("filetype:") || query.contains("type:") {
        score += 0.2;
    }
    if query.contains(" and ") || query.contains(" or ") || query.contains(" not ") {
        score += 0.2;
    }
    if query.contains('(') && query.contains(')') {
        score += 0.1;
    }
    score.min(1.0)
}

/// Question words signal depth. "how to" < "why does" < "compare X and Y".
fn question_depth_signal(words: &[&str]) -> f64 {
    if words.is_empty() {
        return 0.0;
    }

    let mut score: f64 = 0.0;

    // Deep question words
    if words.iter().any(|w| COMPARISON_WORDS.contains(w)) {
        score += 0.5;
    }
    if words.iter().any(|w| DEEP_WORDS.contains(w)) {
        score += 0.4;
    }
    if words.first().is_some_and(|w| HOW_WORDS.contains(w)) {
        score += 0.2;
    }
    if words.first().is_some_and(|w| BASIC_QUESTION.contains(w)) {
        score += 0.1;
    }

    score.min(1.0)
}

/// Capitalized words (potential named entities) increase complexity.
fn entity_signal(words: &[&str]) -> f64 {
    // Count words that look like they could be entities
    // Since we lowercased, check for multi-word terms and proper-noun patterns
    let entity_indicators: usize = words
        .iter()
        .filter(|w| w.len() > 1 && !is_common_word(w))
        .count();

    match entity_indicators {
        0..=1 => 0.0,
        2 => 0.2,
        3 => 0.4,
        4 => 0.6,
        5 => 0.8,
        _ => 1.0,
    }
}

/// Temporal markers increase complexity (need fresh results).
fn temporal_signal(query: &str) -> f64 {
    let mut score: f64 = 0.0;
    if has_year(query) {
        score += 0.5;
    }
    let words: Vec<&str> = query.split_whitespace().collect();
    if words.iter().any(|w| TEMPORAL_WORDS.contains(w)) {
        score += 0.5;
    }
    score.min(1.0)
}

fn has_year(query: &str) -> bool {
    query.split(|c: char| !c.is_ascii_digit()).any(|seg| {
        seg.len() == 4
            && seg.starts_with("20")
            && seg.parse::<u32>().is_ok_and(|y| (2000..=2030).contains(&y))
    })
}

fn is_common_word(word: &str) -> bool {
    const COMMON: &[&str] = &[
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
        "do", "does", "did", "will", "would", "could", "should", "may", "might", "can", "shall",
        "must", "need", "to", "of", "in", "for", "on", "with", "at", "by", "from", "as", "into",
        "about", "like", "through", "after", "over", "between", "out", "up", "not", "no", "but",
        "or", "and", "if", "then", "so", "than", "too", "very", "just", "how", "what", "when",
        "where", "why", "who", "which", "that", "this", "it", "its", "my", "your", "his", "her",
        "their", "our",
    ];
    COMMON.contains(&word)
}

const COMPARISON_WORDS: &[&str] = &[
    "compare",
    "comparing",
    "versus",
    "vs",
    "difference",
    "differences",
    "better",
];
const DEEP_WORDS: &[&str] = &[
    "why",
    "explain",
    "analysis",
    "analyze",
    "evaluate",
    "pros",
    "cons",
    "tradeoff",
    "tradeoffs",
];
const HOW_WORDS: &[&str] = &["how"];
const BASIC_QUESTION: &[&str] = &["what", "who", "when", "where"];
const TEMPORAL_WORDS: &[&str] = &[
    "latest", "recent", "new", "newest", "current", "today", "updated",
];
const CODE_WORDS: &[&str] = &[
    "code",
    "function",
    "error",
    "bug",
    "api",
    "implementation",
    "library",
    "framework",
    "syntax",
];
const ACADEMIC_WORDS: &[&str] = &[
    "paper",
    "research",
    "study",
    "journal",
    "thesis",
    "hypothesis",
    "methodology",
];
const OPINION_WORDS: &[&str] = &[
    "best",
    "worst",
    "opinion",
    "review",
    "recommend",
    "recommendation",
    "favorite",
];
const DATA_WORDS: &[&str] = &[
    "data",
    "dataset",
    "statistics",
    "benchmark",
    "metrics",
    "numbers",
    "chart",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_factual_query() {
        let config = ComplexityConfig::default();
        let result = estimate_complexity("capital of France", &config);
        assert_eq!(result.level, ComplexityLevel::Simple);
        assert_eq!(result.recommended_backends, 1);
        assert!(!result.should_expand);
    }

    #[test]
    fn moderate_query() {
        let config = ComplexityConfig::default();
        let result = estimate_complexity("how to implement binary search in Rust", &config);
        assert!(result.score >= config.simple_threshold);
        assert!(result.recommended_backends >= 2);
    }

    #[test]
    fn complex_query_with_operators() {
        let config = ComplexityConfig::default();
        let result = estimate_complexity(
            "compare \"Rust async\" vs \"Go goroutines\" performance benchmarks site:github.com 2024",
            &config,
        );
        assert_eq!(result.level, ComplexityLevel::Complex);
        assert!(result.needs_deep_ranking);
        assert_eq!(result.recommended_backends, 5);
    }

    #[test]
    fn empty_query_is_simple() {
        let config = ComplexityConfig::default();
        let result = estimate_complexity("", &config);
        assert_eq!(result.level, ComplexityLevel::Simple);
        assert_eq!(result.score, 0.0);
    }

    #[test]
    fn single_word_is_simple() {
        let config = ComplexityConfig::default();
        let result = estimate_complexity("rust", &config);
        assert_eq!(result.level, ComplexityLevel::Simple);
    }

    #[test]
    fn temporal_markers_increase_complexity() {
        let config = ComplexityConfig::default();
        let base = estimate_complexity("rust frameworks", &config);
        let temporal = estimate_complexity("latest rust frameworks 2024", &config);
        assert!(temporal.score > base.score);
    }

    #[test]
    fn comparison_query_detected() {
        let result = suggest_intent("compare React vs Vue");
        assert_eq!(result, QueryIntent::Comparison);
    }

    #[test]
    fn how_to_query_detected() {
        let result = suggest_intent("how to deploy kubernetes");
        assert_eq!(result, QueryIntent::HowTo);
    }

    #[test]
    fn code_query_detected() {
        let result = suggest_intent("python function syntax error");
        assert_eq!(result, QueryIntent::Code);
    }

    #[test]
    fn academic_query_detected() {
        let result = suggest_intent("research paper on transformer models");
        assert_eq!(result, QueryIntent::Academic);
    }

    #[test]
    fn verification_query_detected() {
        let result = suggest_intent("is rust memory safe");
        assert_eq!(result, QueryIntent::Verification);
    }

    #[test]
    fn signals_are_bounded() {
        let config = ComplexityConfig::default();
        let result = estimate_complexity(
            "compare analyze why explain latest 2024 new site:x.com \"quoted\" and or not data benchmark metrics review best code api",
            &config,
        );
        assert!(result.score <= 1.0);
        assert!(result.signals.term_count_signal <= 1.0);
        assert!(result.signals.operator_signal <= 1.0);
        assert!(result.signals.question_depth_signal <= 1.0);
        assert!(result.signals.entity_signal <= 1.0);
        assert!(result.signals.temporal_signal <= 1.0);
    }
}
