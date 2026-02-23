//! V6 Output integrity validation — citation reachability, content hash (PRD §19).

use crate::validate::types::*;

/// Citation reachability check input.
#[derive(Debug, Clone)]
pub struct CitationCheck {
    pub citation_id: String,
    pub url: String,
    pub url_reachable: bool,
    pub content_hash_matches: bool,
}

/// V6 Output Integrity Validator.
pub struct OutputValidator;

impl OutputValidator {
    /// V6: Verify citation links are valid and format is well-formed.
    pub fn validate(citations: &[CitationCheck], format_valid: bool) -> LayerResult {
        let start = std::time::Instant::now();
        let mut issues = Vec::new();
        let mut score = 1.0;
        let mut broken = 0usize;

        for c in citations {
            if !c.url_reachable {
                broken += 1;
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning,
                    code: "V6_BROKEN_CITATION".into(),
                    message: format!(
                        "Citation [{}] URL unreachable: {}",
                        c.citation_id, c.url
                    ),
                    source_url: Some(c.url.clone()),
                });
            }
            if !c.content_hash_matches {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning,
                    code: "V6_CONTENT_CHANGED".into(),
                    message: format!(
                        "Citation [{}] content changed since fetch",
                        c.citation_id
                    ),
                    source_url: Some(c.url.clone()),
                });
            }
        }

        if !citations.is_empty() {
            let broken_ratio = broken as f64 / citations.len() as f64;
            score *= 1.0 - broken_ratio;
        }
        if !format_valid {
            score *= 0.5;
        }

        LayerResult {
            layer: ValidationLayerId::V6OutputIntegrity,
            passed: score >= 0.5,
            score,
            issues,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_reachable_passes() {
        let r = OutputValidator::validate(
            &[CitationCheck {
                citation_id: "1".into(),
                url: "https://example.com".into(),
                url_reachable: true,
                content_hash_matches: true,
            }],
            true,
        );
        assert!(r.passed);
        assert_eq!(r.score, 1.0);
    }

    #[test]
    fn broken_citation_penalizes() {
        let r = OutputValidator::validate(
            &[CitationCheck {
                citation_id: "1".into(),
                url: "https://broken.example.com".into(),
                url_reachable: false,
                content_hash_matches: true,
            }],
            true,
        );
        assert!(r.issues.iter().any(|i| i.code == "V6_BROKEN_CITATION"));
        assert!(!r.passed);
    }

    #[test]
    fn content_changed_flagged() {
        let r = OutputValidator::validate(
            &[CitationCheck {
                citation_id: "1".into(),
                url: "https://example.com".into(),
                url_reachable: true,
                content_hash_matches: false,
            }],
            true,
        );
        assert!(r.issues.iter().any(|i| i.code == "V6_CONTENT_CHANGED"));
    }
}
