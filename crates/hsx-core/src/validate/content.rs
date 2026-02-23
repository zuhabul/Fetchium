//! V2 Content validation — relevance, paywall detection, minimum length (PRD §19).

use crate::validate::types::*;

/// Input for content validation.
#[derive(Debug, Clone)]
pub struct ContentInput {
    pub url: String,
    pub text: String,
}

/// V2 Content Validator.
pub struct ContentValidator {
    pub min_relevance_score: f64,
    pub min_content_length: usize,
}

impl Default for ContentValidator {
    fn default() -> Self {
        Self {
            min_relevance_score: 0.2,
            min_content_length: 100,
        }
    }
}

impl ContentValidator {
    /// V2: Validate content quality of extracted text against the query.
    pub fn validate(&self, sources: &[ContentInput], query: &str) -> LayerResult {
        let start = std::time::Instant::now();
        let mut issues = Vec::new();
        let mut scores = Vec::new();

        for src in sources {
            let mut score = 1.0;

            if src.text.len() < self.min_content_length {
                score *= 0.3;
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning,
                    code: "V2_TOO_SHORT".into(),
                    message: format!("Content too short: {} chars", src.text.len()),
                    source_url: Some(src.url.clone()),
                });
            }

            if Self::is_error_page(&src.text) {
                score *= 0.1;
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Error,
                    code: "V2_ERROR_PAGE".into(),
                    message: "Detected error or paywall page".into(),
                    source_url: Some(src.url.clone()),
                });
            }

            let query_terms: Vec<&str> = query
                .split_whitespace()
                .filter(|w| w.len() > 2)
                .collect();
            let lower = src.text.to_lowercase();
            let term_hits = query_terms
                .iter()
                .filter(|t| lower.contains(&t.to_lowercase()))
                .count();
            let relevance = if query_terms.is_empty() {
                0.5
            } else {
                term_hits as f64 / query_terms.len() as f64
            };

            if relevance < self.min_relevance_score {
                score *= 0.4;
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning,
                    code: "V2_LOW_RELEVANCE".into(),
                    message: format!("Low relevance score: {relevance:.2}"),
                    source_url: Some(src.url.clone()),
                });
            }

            scores.push(score * relevance.max(0.1));
        }

        let avg = if scores.is_empty() {
            0.5
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };

        LayerResult {
            layer: ValidationLayerId::V2Content,
            passed: avg >= 0.2,
            score: avg,
            issues,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    fn is_error_page(text: &str) -> bool {
        let lower = text.to_lowercase();
        let patterns = [
            "404 not found",
            "403 forbidden",
            "access denied",
            "subscribe to continue",
            "sign in to view",
            "paywall",
            "enable javascript",
            "captcha",
            "too many requests",
        ];
        patterns.iter().any(|p| lower.contains(p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relevant_content_passes() {
        let v = ContentValidator::default();
        let src = ContentInput {
            url: "https://rust-lang.org".into(),
            text: "Rust is a systems programming language focused on safety, speed, and concurrency. It provides memory safety without a garbage collector.".into(),
        };
        let r = v.validate(&[src], "Rust programming language");
        assert!(r.passed);
    }

    #[test]
    fn paywall_detected() {
        let v = ContentValidator::default();
        let src = ContentInput {
            url: "https://example.com".into(),
            text: "Subscribe to continue reading this article about Rust programming.".into(),
        };
        let r = v.validate(&[src], "Rust");
        assert!(r.issues.iter().any(|i| i.code == "V2_ERROR_PAGE"));
    }

    #[test]
    fn short_content_penalized() {
        let v = ContentValidator::default();
        let src = ContentInput {
            url: "https://example.com".into(),
            text: "Short text about Rust.".into(),
        };
        let r = v.validate(&[src], "Rust");
        assert!(r.issues.iter().any(|i| i.code == "V2_TOO_SHORT"));
    }
}
