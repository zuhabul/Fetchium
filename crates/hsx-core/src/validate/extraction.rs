//! V5 Extraction quality validation — completeness, truncation, segments (PRD §19).

use crate::validate::types::*;

/// Input for extraction quality validation.
#[derive(Debug, Clone)]
pub struct ExtractionInput {
    pub url: String,
    pub truncated: bool,
    pub segment_count: usize,
    pub encoding_errors: usize,
}

/// V5 Extraction Quality Validator.
pub struct ExtractionValidator;

impl ExtractionValidator {
    /// V5: Validate extraction quality metrics.
    pub fn validate(sources: &[ExtractionInput]) -> LayerResult {
        let start = std::time::Instant::now();
        let mut issues = Vec::new();
        let mut scores = Vec::new();

        for src in sources {
            let mut score = 1.0;

            if src.truncated {
                score *= 0.7;
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning,
                    code: "V5_TRUNCATED".into(),
                    message: "Content was truncated during extraction".into(),
                    source_url: Some(src.url.clone()),
                });
            }

            if src.segment_count == 0 {
                score *= 0.2;
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Error,
                    code: "V5_NO_SEGMENTS".into(),
                    message: "Extraction produced zero segments".into(),
                    source_url: Some(src.url.clone()),
                });
            }

            if src.encoding_errors > 0 {
                score *= 0.8;
            }

            scores.push(score);
        }

        let avg = if scores.is_empty() {
            0.5
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };

        LayerResult {
            layer: ValidationLayerId::V5ExtractionQuality,
            passed: avg >= 0.3,
            score: avg,
            issues,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn good_extraction_passes() {
        let r = ExtractionValidator::validate(&[ExtractionInput {
            url: "https://example.com".into(),
            truncated: false,
            segment_count: 10,
            encoding_errors: 0,
        }]);
        assert!(r.passed);
        assert_eq!(r.score, 1.0);
    }

    #[test]
    fn truncated_penalized() {
        let r = ExtractionValidator::validate(&[ExtractionInput {
            url: "https://example.com".into(),
            truncated: true,
            segment_count: 5,
            encoding_errors: 0,
        }]);
        assert!(r.issues.iter().any(|i| i.code == "V5_TRUNCATED"));
        assert!(r.score < 1.0);
    }

    #[test]
    fn no_segments_fails() {
        let r = ExtractionValidator::validate(&[ExtractionInput {
            url: "https://example.com".into(),
            truncated: false,
            segment_count: 0,
            encoding_errors: 0,
        }]);
        assert!(r.issues.iter().any(|i| i.code == "V5_NO_SEGMENTS"));
        assert!(!r.passed);
    }
}
