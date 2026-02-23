//! V1 Authority scoring — domain reputation, SSL, redirect analysis (PRD §19, §21).

use crate::validate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Domain reputation tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainTier {
    /// Tier 1: Highly authoritative (.gov, .edu, major outlets).
    Authoritative,
    /// Tier 2: Generally reliable (established tech blogs, official docs).
    Reliable,
    /// Tier 3: Mixed quality (forums, personal blogs).
    Mixed,
    /// Tier 4: Low trust (unknown, new domains).
    Unknown,
    /// Tier 5: Blocklisted (known spam, SEO farms).
    Blocked,
}

/// V1 Authority Scorer.
pub struct AuthorityScorer {
    domain_tiers: HashMap<String, DomainTier>,
}

impl Default for AuthorityScorer {
    fn default() -> Self {
        let mut tiers = HashMap::new();
        // Tier 1: Authoritative
        for d in &[
            "github.com", "docs.rs", "rust-lang.org", "wikipedia.org", "arxiv.org",
            "nature.com", "science.org", "ieee.org", "acm.org", "nih.gov", "cdc.gov",
        ] {
            tiers.insert(d.to_string(), DomainTier::Authoritative);
        }
        // Tier 2: Reliable
        for d in &[
            "stackoverflow.com", "developer.mozilla.org", "medium.com",
            "dev.to", "hacker-news.firebaseio.com", "reddit.com",
        ] {
            tiers.insert(d.to_string(), DomainTier::Reliable);
        }
        Self { domain_tiers: tiers }
    }
}

impl AuthorityScorer {
    /// Score a single source URL. Returns (score 0.0-1.0, domain tier).
    pub fn score(&self, url: &str, has_ssl: bool, redirect_count: u32) -> (f64, DomainTier) {
        let domain = Self::extract_domain(url);
        let tier = self.domain_tiers.get(&domain).copied().unwrap_or_else(|| {
            if domain.ends_with(".gov") || domain.ends_with(".edu") {
                DomainTier::Authoritative
            } else if domain.ends_with(".org") {
                DomainTier::Reliable
            } else {
                DomainTier::Unknown
            }
        });

        let mut score: f64 = match tier {
            DomainTier::Authoritative => 0.95,
            DomainTier::Reliable => 0.75,
            DomainTier::Mixed => 0.50,
            DomainTier::Unknown => 0.40,
            DomainTier::Blocked => 0.05,
        };

        if !has_ssl { score *= 0.7; }
        if redirect_count > 2 { score *= 0.9; }
        if redirect_count > 5 { score *= 0.8; }

        (score.clamp(0.0, 1.0), tier)
    }

    /// Run as V1 validation layer.
    /// Each tuple: (url, has_ssl, redirect_count).
    pub fn validate_sources(&self, sources: &[(String, bool, u32)]) -> LayerResult {
        let start = std::time::Instant::now();
        let mut issues = Vec::new();
        let mut scores = Vec::new();

        for (url, has_ssl, redirects) in sources {
            let (score, tier) = self.score(url, *has_ssl, *redirects);
            scores.push(score);

            if tier == DomainTier::Blocked {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Error,
                    code: "V1_BLOCKED_DOMAIN".into(),
                    message: format!("Domain is blocklisted: {url}"),
                    source_url: Some(url.clone()),
                });
            }
            if !has_ssl {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning,
                    code: "V1_NO_SSL".into(),
                    message: format!("No SSL/TLS: {url}"),
                    source_url: Some(url.clone()),
                });
            }
        }

        let avg = if scores.is_empty() {
            0.5
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };

        LayerResult {
            layer: ValidationLayerId::V1Source,
            passed: avg >= 0.3,
            score: avg,
            issues,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    fn extract_domain(url: &str) -> String {
        url::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gov_domain_high_score() {
        let s = AuthorityScorer::default();
        let (score, tier) = s.score("https://nih.gov/article", true, 0);
        assert!(score >= 0.9);
        assert_eq!(tier, DomainTier::Authoritative);
    }

    #[test]
    fn no_ssl_penalty() {
        let s = AuthorityScorer::default();
        let (with, _) = s.score("https://example.com/page", true, 0);
        let (without, _) = s.score("http://example.com/page", false, 0);
        assert!(without < with);
    }

    #[test]
    fn redirect_penalty() {
        let s = AuthorityScorer::default();
        let (no_redirect, _) = s.score("https://github.com/repo", true, 0);
        let (many_redirects, _) = s.score("https://github.com/repo", true, 6);
        assert!(many_redirects < no_redirect);
    }

    #[test]
    fn validate_layer_result() {
        let s = AuthorityScorer::default();
        let sources = vec![
            ("https://rust-lang.org/page".to_string(), true, 0u32),
            ("https://docs.rs/crate".to_string(), true, 0u32),
        ];
        let r = s.validate_sources(&sources);
        assert!(r.passed);
        assert!(r.score > 0.8);
    }
}
