//! V3 Temporal/freshness validation — exponential decay scoring (PRD §19, §21).

use crate::validate::types::*;
use chrono::{DateTime, Datelike, TimeZone, Utc};

/// Query temporal intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalIntent {
    Recent,
    Historical,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TemporalQueryProfile {
    intent: TemporalIntent,
    explicit_year: Option<i32>,
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
        Self::analyze_query(query).intent
    }

    fn analyze_query(query: &str) -> TemporalQueryProfile {
        let q = query.to_lowercase();
        if [
            "latest",
            "newest",
            "recent",
            "current",
            "today",
            "yesterday",
            "tonight",
            "this week",
            "this month",
            "this quarter",
            "this year",
            "as of",
            "breaking",
            "just announced",
            "just released",
        ]
        .iter()
        .any(|s| q.contains(s))
        {
            return TemporalQueryProfile {
                intent: TemporalIntent::Recent,
                explicit_year: extract_explicit_year(&q),
            };
        }

        let explicit_year = extract_explicit_year(&q);
        if let Some(year) = explicit_year {
            let current_year = Utc::now().year();
            let intent = if year >= current_year - 1 {
                TemporalIntent::Recent
            } else {
                TemporalIntent::Historical
            };
            return TemporalQueryProfile {
                intent,
                explicit_year: Some(year),
            };
        }

        if [
            "history of",
            "origin of",
            "when was",
            "first",
            "originally",
            "founded",
        ]
        .iter()
        .any(|s| q.contains(s))
        {
            return TemporalQueryProfile {
                intent: TemporalIntent::Historical,
                explicit_year: None,
            };
        }

        TemporalQueryProfile {
            intent: TemporalIntent::Default,
            explicit_year: None,
        }
    }

    fn max_age_days(&self, profile: TemporalQueryProfile, now: DateTime<Utc>) -> u64 {
        if profile.intent == TemporalIntent::Recent {
            return self.recent_max_age_days;
        }

        if let Some(year) = profile.explicit_year {
            if year > now.year() {
                return 30;
            }
            if let Some(start_of_year) = Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0).single() {
                let days_since_year_start = (now - start_of_year).num_days().max(0) as u64;
                let floor = match profile.intent {
                    TemporalIntent::Historical => 365,
                    TemporalIntent::Default => self.default_max_age_days,
                    TemporalIntent::Recent => self.recent_max_age_days,
                };
                return (days_since_year_start + 366).max(floor);
            }
        }

        match profile.intent {
            TemporalIntent::Recent => self.recent_max_age_days,
            TemporalIntent::Historical => self.historical_max_age_days,
            TemporalIntent::Default => self.default_max_age_days,
        }
    }

    fn source_matches_explicit_year(date: DateTime<Utc>, year: i32) -> bool {
        date.year() == year
    }

    /// Validate freshness of sources against query intent.
    pub fn validate(&self, sources: &[SourceFreshness], query: &str) -> LayerResult {
        let start = std::time::Instant::now();
        let now = Utc::now();
        let profile = Self::analyze_query(query);
        let max_age = self.max_age_days(profile, now);

        let mut issues = Vec::new();
        let mut scores = Vec::new();

        for s in sources {
            let best = s.published_date.or(s.last_modified);
            match best {
                Some(date) => {
                    if date > now {
                        scores.push(0.0);
                        issues.push(ValidationIssue {
                            severity: IssueSeverity::Error,
                            code: "V3_FUTURE_DATE".into(),
                            message: format!(
                                "Source timestamp is in the future: {}",
                                date.date_naive()
                            ),
                            source_url: Some(s.url.clone()),
                        });
                        continue;
                    }

                    let age = (now - date).num_days().max(0) as u64;
                    let mut score = (-2.0 * age as f64 / max_age as f64).exp();

                    if let Some(year) = profile.explicit_year {
                        if !Self::source_matches_explicit_year(date, year) {
                            score *= 0.25;
                            issues.push(ValidationIssue {
                                severity: if profile.intent == TemporalIntent::Recent {
                                    IssueSeverity::Error
                                } else {
                                    IssueSeverity::Warning
                                },
                                code: "V3_YEAR_MISMATCH".into(),
                                message: format!(
                                    "Source dated {} but query targets year {}",
                                    date.year(),
                                    year
                                ),
                                source_url: Some(s.url.clone()),
                            });
                        }
                    }

                    let edf = crate::intelligence::edf::EvidenceDecayFunction::new();
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
                                "{} days old (max {} for {:?})",
                                age, max_age, profile.intent
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
            passed: avg >= 0.3
                && !issues
                    .iter()
                    .any(|issue| issue.severity == IssueSeverity::Error),
            score: avg,
            issues,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }
}

fn extract_explicit_year(query: &str) -> Option<i32> {
    query.split(|c: char| !c.is_ascii_digit()).find_map(|part| {
        if part.len() == 4 {
            part.parse::<i32>()
                .ok()
                .filter(|year| (1900..=2100).contains(year))
        } else {
            None
        }
    })
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
        assert_eq!(
            TemporalValidator::classify_intent("latest news"),
            TemporalIntent::Recent
        );
        assert_eq!(
            TemporalValidator::classify_intent("history of Rust"),
            TemporalIntent::Historical
        );
        assert_eq!(
            TemporalValidator::classify_intent("what is Rust"),
            TemporalIntent::Default
        );
    }

    #[test]
    fn detects_relative_date_queries_as_recent() {
        assert_eq!(
            TemporalValidator::classify_intent("current interest rates today"),
            TemporalIntent::Recent
        );
        assert_eq!(
            TemporalValidator::classify_intent("Q1 2026 earnings"),
            TemporalIntent::Recent
        );
    }

    #[test]
    fn future_dated_source_fails_closed() {
        let v = TemporalValidator::default();
        let r = v.validate(
            &[SourceFreshness {
                url: "https://x.com".into(),
                published_date: Some(Utc::now() + Duration::days(2)),
                last_modified: None,
            }],
            "latest security advisory",
        );
        assert!(!r.passed);
        assert!(r.issues.iter().any(|i| i.code == "V3_FUTURE_DATE"));
    }

    #[test]
    fn explicit_year_mismatch_is_flagged() {
        let v = TemporalValidator::default();
        let r = v.validate(
            &[SourceFreshness {
                url: "https://x.com".into(),
                published_date: Some(Utc.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).single().unwrap()),
                last_modified: None,
            }],
            "best ETFs for 2024",
        );
        assert!(r.issues.iter().any(|i| i.code == "V3_YEAR_MISMATCH"));
    }
}
