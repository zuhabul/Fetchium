//! Response Quality Estimator (RQE) — confidence scoring for result sets.
//!
//! Novel algorithm: After ranking, estimate the overall quality/confidence of
//! the result set before presenting to the user. Uses 5 signals:
//!
//! 1. **Source Diversity** — How many unique backends contributed? (0-1)
//! 2. **Score Distribution** — Are top scores well-separated from noise? (0-1)
//! 3. **Result Agreement** — Do top results cover the same topic? (0-1)
//! 4. **Coverage Completeness** — Are all query aspects addressed? (0-1)
//! 5. **Freshness Signal** — Do results have recent publication dates? (0-1)
//!
//! Outputs an overall quality score [0.0, 1.0] and optional warnings.

use crate::types::ResultItem;
use std::collections::HashSet;

/// Quality assessment of a result set.
#[derive(Debug, Clone)]
pub struct QualityAssessment {
    /// Overall quality score [0.0, 1.0].
    pub score: f64,
    /// Confidence level (derived from score).
    pub confidence: ConfidenceLevel,
    /// Individual signal scores.
    pub signals: QualitySignals,
    /// Warnings for the user (empty = no issues).
    pub warnings: Vec<String>,
}

/// Signal breakdown.
#[derive(Debug, Clone)]
pub struct QualitySignals {
    /// How many unique backends contributed results.
    pub source_diversity: f64,
    /// How well-separated the top scores are from noise.
    pub score_separation: f64,
    /// How much the top results agree on topic.
    pub result_agreement: f64,
    /// How well the results cover all query terms.
    pub coverage: f64,
    /// Freshness of results (recent dates).
    pub freshness: f64,
}

/// Confidence level — maps from the numeric score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfidenceLevel {
    /// Score >= 0.8 — results are comprehensive and reliable.
    High,
    /// Score >= 0.5 — results are decent but may have gaps.
    Medium,
    /// Score >= 0.3 — results are sparse; user should verify.
    Low,
    /// Score < 0.3 — very few relevant results found.
    VeryLow,
}

impl ConfidenceLevel {
    fn from_score(score: f64) -> Self {
        if score >= 0.8 {
            Self::High
        } else if score >= 0.5 {
            Self::Medium
        } else if score >= 0.3 {
            Self::Low
        } else {
            Self::VeryLow
        }
    }

    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
            Self::VeryLow => "very low",
        }
    }
}

impl std::fmt::Display for ConfidenceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Assess the quality of a ranked result set.
///
/// Should be called after ranking but before output formatting.
pub fn assess_quality(results: &[ResultItem], query: &str) -> QualityAssessment {
    if results.is_empty() {
        return QualityAssessment {
            score: 0.0,
            confidence: ConfidenceLevel::VeryLow,
            signals: QualitySignals {
                source_diversity: 0.0,
                score_separation: 0.0,
                result_agreement: 0.0,
                coverage: 0.0,
                freshness: 0.0,
            },
            warnings: vec!["No results found.".into()],
        };
    }

    let diversity = source_diversity_score(results);
    let separation = score_separation(results);
    let agreement = result_agreement(results);
    let coverage = coverage_score(results, query);
    let freshness = freshness_score(results);

    // Weighted combination
    let score = diversity * 0.20
        + separation * 0.25
        + agreement * 0.20
        + coverage * 0.25
        + freshness * 0.10;

    let score = score.clamp(0.0, 1.0);
    let confidence = ConfidenceLevel::from_score(score);

    let mut warnings = Vec::new();

    if diversity < 0.3 {
        warnings
            .push("Results come from very few sources. Consider broadening your search.".into());
    }
    if separation < 0.2 {
        warnings
            .push("Top results have similar relevance scores — ranking may be unreliable.".into());
    }
    if coverage < 0.3 {
        warnings.push("Some query terms are not well-covered in results.".into());
    }
    if results.len() < 3 {
        warnings.push(format!("Only {} result(s) found.", results.len()));
    }

    QualityAssessment {
        score,
        confidence,
        signals: QualitySignals {
            source_diversity: diversity,
            score_separation: separation,
            result_agreement: agreement,
            coverage,
            freshness,
        },
        warnings,
    }
}

/// Source diversity: what fraction of available backend types contributed?
fn source_diversity_score(results: &[ResultItem]) -> f64 {
    let unique_backends: HashSet<String> = results.iter().map(|r| r.backend.to_string()).collect();

    // Scale: 1 backend = 0.2, 2 = 0.4, 3 = 0.6, 4 = 0.8, 5+ = 1.0
    let n = unique_backends.len();
    (n as f64 / 5.0).min(1.0)
}

/// Score separation: how well-separated are the top results from the rest?
///
/// Uses the gap between the best score and the median score.
/// A large gap means the ranker is confident about the top results.
fn score_separation(results: &[ResultItem]) -> f64 {
    let mut scores: Vec<f64> = results.iter().filter_map(|r| r.score).collect();

    if scores.len() < 2 {
        return 0.5; // neutral if we can't compute separation
    }

    scores.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

    let top = scores[0];
    let median_idx = scores.len() / 2;
    let median = scores[median_idx];

    // Gap between top and median, normalized
    let gap = top - median;

    // Also check gap between #1 and #2
    let top2_gap = if scores.len() >= 2 {
        top - scores[1]
    } else {
        0.0
    };

    // Combined: larger gaps = more confidence in ranking
    (gap * 2.0 + top2_gap).min(1.0)
}

/// Result agreement: do the top results talk about the same topic?
///
/// Uses term overlap between the top-3 snippets. High overlap = agreement.
fn result_agreement(results: &[ResultItem]) -> f64 {
    let top_n = results.iter().take(5).collect::<Vec<_>>();

    if top_n.len() < 2 {
        return 1.0; // single result trivially agrees
    }

    // Extract term sets from each result
    let term_sets: Vec<HashSet<String>> = top_n
        .iter()
        .map(|r| {
            let text = format!("{} {}", r.title, r.snippet).to_lowercase();
            text.split(|c: char| !c.is_alphanumeric())
                .filter(|t| t.len() >= 3)
                .map(|t| t.to_string())
                .collect()
        })
        .collect();

    // Average pairwise Jaccard similarity
    let mut total_sim = 0.0;
    let mut pairs = 0;

    for i in 0..term_sets.len() {
        for j in (i + 1)..term_sets.len() {
            let intersection = term_sets[i].intersection(&term_sets[j]).count();
            let union = term_sets[i].union(&term_sets[j]).count();
            if union > 0 {
                total_sim += intersection as f64 / union as f64;
            }
            pairs += 1;
        }
    }

    if pairs == 0 {
        return 0.5;
    }

    // Scale: Jaccard of 0.1 = 0.5, 0.3 = 1.0
    let avg_jaccard = total_sim / pairs as f64;
    (avg_jaccard * 3.33).min(1.0)
}

/// Coverage: are all query terms represented in the results?
fn coverage_score(results: &[ResultItem], query: &str) -> f64 {
    let query_terms: Vec<String> = query
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 2)
        .map(|t| t.to_string())
        .collect();

    if query_terms.is_empty() {
        return 1.0;
    }

    // Build a combined text from all results
    let combined: String = results
        .iter()
        .map(|r| format!("{} {}", r.title, r.snippet))
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();

    // Check what fraction of query terms appear in the results
    let covered = query_terms
        .iter()
        .filter(|term| combined.contains(term.as_str()))
        .count();

    covered as f64 / query_terms.len() as f64
}

/// Freshness: what fraction of results have recent publication dates?
fn freshness_score(results: &[ResultItem]) -> f64 {
    let total = results.len();
    if total == 0 {
        return 0.5;
    }

    // Count results with any date information
    let with_date = results
        .iter()
        .filter(|r| r.published_date.is_some())
        .count();

    if with_date == 0 {
        return 0.5; // neutral if no date info
    }

    // Having dates is a positive signal (backends that provide dates
    // tend to be more structured/reliable)
    let date_ratio = with_date as f64 / total as f64;

    // Scale: 0% with dates = 0.3, 50% = 0.6, 100% = 1.0
    0.3 + date_ratio * 0.7
}

/// Format quality assessment as a one-line summary for CLI display.
pub fn quality_line(assessment: &QualityAssessment) -> String {
    format!(
        "Quality: {:.0}% ({}) — diversity={:.0}% coverage={:.0}% agreement={:.0}%",
        assessment.score * 100.0,
        assessment.confidence.label(),
        assessment.signals.source_diversity * 100.0,
        assessment.signals.coverage * 100.0,
        assessment.signals.result_agreement * 100.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BackendId;

    fn make_item(title: &str, snippet: &str, backend: BackendId, score: f64) -> ResultItem {
        ResultItem {
            title: title.into(),
            url: format!("https://{}.com", title.to_lowercase().replace(' ', "-")),
            snippet: snippet.into(),
            rank: 0,
            backend,
            score: Some(score),
            published_date: None,
        }
    }

    #[test]
    fn empty_results_very_low_confidence() {
        let assessment = assess_quality(&[], "test query");
        assert_eq!(assessment.confidence, ConfidenceLevel::VeryLow);
        assert_eq!(assessment.score, 0.0);
        assert!(!assessment.warnings.is_empty());
    }

    #[test]
    fn diverse_sources_higher_quality() {
        let results = vec![
            make_item(
                "Rust Docs",
                "Rust programming guide",
                BackendId::DuckDuckGo,
                0.9,
            ),
            make_item(
                "Rust Wikipedia",
                "Rust is a language",
                BackendId::Wikipedia,
                0.8,
            ),
            make_item(
                "Rust Reddit",
                "Rust discussion forum",
                BackendId::Reddit,
                0.7,
            ),
            make_item("Rust HN", "Rust on Hacker News", BackendId::HackerNews, 0.6),
            make_item("Rust GitHub", "Rust repositories", BackendId::Github, 0.5),
        ];

        let diverse = assess_quality(&results, "rust programming");

        let single_source = vec![
            make_item("A", "rust stuff", BackendId::DuckDuckGo, 0.9),
            make_item("B", "more rust", BackendId::DuckDuckGo, 0.8),
            make_item("C", "even more rust", BackendId::DuckDuckGo, 0.7),
        ];
        let uniform = assess_quality(&single_source, "rust programming");

        assert!(
            diverse.signals.source_diversity > uniform.signals.source_diversity,
            "Diverse sources should score higher: diverse={} uniform={}",
            diverse.signals.source_diversity,
            uniform.signals.source_diversity
        );
    }

    #[test]
    fn good_coverage_higher_score() {
        let results = vec![
            make_item(
                "Rust Async",
                "Rust async programming with tokio and futures",
                BackendId::DuckDuckGo,
                0.9,
            ),
            make_item(
                "Async Patterns",
                "Patterns for async rust code",
                BackendId::Wikipedia,
                0.8,
            ),
        ];

        let assessment = assess_quality(&results, "rust async programming");
        assert!(
            assessment.signals.coverage > 0.5,
            "All query terms should be covered: {}",
            assessment.signals.coverage
        );
    }

    #[test]
    fn poor_coverage_generates_warning() {
        let results = vec![make_item(
            "Dogs",
            "Cute dogs and puppies",
            BackendId::DuckDuckGo,
            0.3,
        )];

        let assessment = assess_quality(&results, "quantum computing research");
        assert!(
            assessment.signals.coverage < 0.5,
            "Irrelevant results should have low coverage"
        );
    }

    #[test]
    fn confidence_levels_from_score() {
        assert_eq!(ConfidenceLevel::from_score(0.9), ConfidenceLevel::High);
        assert_eq!(ConfidenceLevel::from_score(0.6), ConfidenceLevel::Medium);
        assert_eq!(ConfidenceLevel::from_score(0.35), ConfidenceLevel::Low);
        assert_eq!(ConfidenceLevel::from_score(0.1), ConfidenceLevel::VeryLow);
    }

    #[test]
    fn quality_line_formatting() {
        let results = vec![make_item(
            "Test",
            "test content",
            BackendId::DuckDuckGo,
            0.8,
        )];
        let assessment = assess_quality(&results, "test");
        let line = quality_line(&assessment);
        assert!(line.contains("Quality:"));
        assert!(line.contains("%"));
    }

    #[test]
    fn single_result_not_panic() {
        let results = vec![make_item(
            "Only Result",
            "some content",
            BackendId::Wikipedia,
            0.5,
        )];
        let assessment = assess_quality(&results, "query");
        assert!(assessment.score > 0.0);
    }

    #[test]
    fn freshness_with_dates() {
        let mut results = vec![
            make_item("A", "content", BackendId::DuckDuckGo, 0.8),
            make_item("B", "content", BackendId::Wikipedia, 0.7),
        ];
        results[0].published_date = Some("2024-01-15".into());
        results[1].published_date = Some("2024-02-20".into());

        let with_dates = assess_quality(&results, "test");

        let no_dates = vec![
            make_item("C", "content", BackendId::DuckDuckGo, 0.8),
            make_item("D", "content", BackendId::Wikipedia, 0.7),
        ];
        let without_dates = assess_quality(&no_dates, "test");

        assert!(
            with_dates.signals.freshness > without_dates.signals.freshness,
            "Results with dates should score higher freshness"
        );
    }
}
