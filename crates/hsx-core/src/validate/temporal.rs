//! V3 Temporal/freshness validation — exponential decay scoring (PRD §19, §21).

use crate::validate::types::*;
use chrono::{DateTime, Utc};

/// Query temporal intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalIntent {
    Recent,
    Historical,
    Default,
}

/// Freshness metadata for a single source.
#[derive(Debug, Clone)]
pub struct SourceFreshness {
    pub url: String,
    pub published_date: Option<DateTime<Utc>>,
    pub last_modified: Option<DateTime<Utc>>,
}

/// V3 Temporal Validator.
pub struct TemporalValidator {
    pub default_max_age_days: u64,
    pub recent_max_age_days: u64,
    pub historical_max_age_days: u64,
}

impl Default for TemporalValidator {
    fn default() -> Self {
        Self {
            default_max_age_days: 365,
            recent_max_age_days: 30,
            historical_max_age_days: 3650,
        }
    }
}

impl TemporalValidator {
    /// Classify query temporal intent from keywords.
    pub fn classify_intent(query: &str) -> TemporalIntent {
        let q = query.to_lowercase();
        if ["latest", "newest", "recent", "2026", "2025", "this year", "just released"]
            .iter()
            .any(|s| q.contains(s))
        {
            TemporalIntent::Recent
        } else if ["history of", "origin of", "when was", "first", "originally", "founded"]
            .iter()
            .any(|s| q.contains(s))
        {
            TemporalIntent::Historical
        } else {
            TemporalIntent::Default
        }
    }

    /// Validate freshness of sources against query intent.
    pub fn validate(&self, sources: &[SourceFreshness], query: &str) -> LayerResult {
        let start = std::time::Instant::now();
        let intent = Self::classify_intent(query);
        let max_age = match intent {
            TemporalIntent::Recent => self.recent_max_age_days,
            TemporalIntent::Historical => self.historical_max_age_days,
            TemporalIntent::Default => self.default_max_age_days,
        };

        let now = Utc::now();
        let mut issues = Vec::new();
        let mut scores = Vec::new();

        for s in sources {
            let best = s.published_date.or(s.last_modified);
            match best {
                Some(date) => {
                    let age = (now - date).num_days().unsigned_abs();
                    let mut score = (-2.0 * age as f64 / max_age as f64).exp();
                    
                    // Intelligence layer: Evidence Decay Function (EDF)
                    let edf = crate::intelligence::edf::EvidenceDecayFunction::new();
                    // Just use a generic "tech" domain for broad calibration, or try to guess.
                    let edf_result = edf.decay(score, "technology", age as f64);
                    score = edf_result.decayed_confidence;

                    scores.push(score);
                    if age > max_age {
                        issues.push(ValidationIssue {
                            severity: if age > max_age * 2 {
                                IssueSeverity::Error
                            } else {
                                IssueSeverity::Warning
                            },
                            code: "V3_STALE_CONTENT".into(),
                            message: format!(
                                "{} days old (max {} for {intent:?})",
                                age, max_age
                            ),
                            source_url: Some(s.url.clone()),
                        });
                    }
                }
                None => {
                    scores.push(0.3);
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Warning,
                        code: "V3_NO_DATE".into(),
                        message: "No published date found".into(),
                        source_url: Some(s.url.clone()),
                    });
                }
            }
        }

        let avg = if scores.is_empty() {
            0.5
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };

        LayerResult {
            layer: ValidationLayerId::V3Freshness,
            passed: avg >= 0.3,
            score: avg,
            issues,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn fresh_content_passes() {
        let v = TemporalValidator::default();
        let r = v.validate(
            &[SourceFreshness {
                url: "https://x.com".into(),
                published_date: Some(Utc::now() - Duration::days(5)),
                last_modified: None,
            }],
            "what is Rust",
        );
        assert!(r.passed);
        assert!(r.score > 0.9);
    }

    #[test]
    fn stale_for_recent_query() {
        let v = TemporalValidator::default();
        let r = v.validate(
            &[SourceFreshness {
                url: "https://x.com".into(),
                published_date: Some(Utc::now() - Duration::days(90)),
                last_modified: None,
            }],
            "latest Rust news 2026",
        );
        assert!(r.issues.iter().any(|i| i.code == "V3_STALE_CONTENT"));
    }

    #[test]
    fn undated_penalized() {
        let v = TemporalValidator::default();
        let r = v.validate(
            &[SourceFreshness {
                url: "https://x.com".into(),
                published_date: None,
                last_modified: None,
            }],
            "what is Rust",
        );
        assert!(r.issues.iter().any(|i| i.code == "V3_NO_DATE"));
        assert!((r.score - 0.3).abs() < 1e-9);
    }

    #[test]
    fn intent_classification() {
        assert_eq!(TemporalValidator::classify_intent("latest news"), TemporalIntent::Recent);
        assert_eq!(TemporalValidator::classify_intent("history of Rust"), TemporalIntent::Historical);
        assert_eq!(TemporalValidator::classify_intent("what is Rust"), TemporalIntent::Default);
    }
}
