//! Source Trust Propagation (STP) — domain-level trust scoring.
//!
//! Maintains a trust database of domains with scores from 0.0–1.0 based on
//! known reliability tiers. Scores are applied as multiplicative adjustments
//! to search result relevance scores during the ranking pipeline.

use std::collections::HashMap;

use crate::types::ResultItem;

/// Tuning knobs for how trust scores modify result relevance.
#[derive(Debug, Clone)]
pub struct TrustConfig {
    /// Score assigned to completely unknown domains (0.0–1.0).
    pub default_trust: f64,
    /// Multiplicative boost for [`DomainCategory::Authoritative`] results.
    pub authoritative_boost: f64,
    /// Multiplicative penalty for low-quality / spam domains.
    pub low_quality_penalty: f64,
    /// Factor applied when a domain is not in the database at all.
    pub unknown_domain_factor: f64,
}

impl Default for TrustConfig {
    fn default() -> Self {
        Self {
            default_trust: 0.5,
            authoritative_boost: 1.3,
            low_quality_penalty: 0.5,
            unknown_domain_factor: 0.8,
        }
    }
}

/// Reliability category for a domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DomainCategory {
    /// Government, education, and established technical references.
    Authoritative,
    /// Community-curated knowledge bases (Wikipedia, package registries).
    Curated,
    /// Major news outlets.
    News,
    /// Forums, blogs, social platforms.
    UserGenerated,
    /// Domain not recognised.
    Unknown,
}

/// Trust record for a single domain.
#[derive(Debug, Clone)]
pub struct DomainTrust {
    /// The root domain (e.g. `github.com`).
    pub domain: String,
    /// Trust score in the range 0.0–1.0.
    pub trust_score: f64,
    /// Broad reliability category.
    pub category: DomainCategory,
    /// Number of observations that informed this score (0 = hardcoded default).
    pub sample_count: u32,
}

/// In-memory trust database seeded with well-known domain scores.
#[derive(Debug, Clone)]
pub struct TrustDatabase {
    domains: HashMap<String, DomainTrust>,
    default_trust: f64,
}

impl TrustDatabase {
    /// Create a new database pre-populated with ~30 well-known domains.
    pub fn new() -> Self {
        use DomainCategory::*;
        let seed: &[(&str, f64, DomainCategory)] = &[
            ("github.com", 0.95, Authoritative),
            ("stackoverflow.com", 0.95, Authoritative),
            ("docs.python.org", 0.95, Authoritative),
            ("doc.rust-lang.org", 0.95, Authoritative),
            ("developer.mozilla.org", 0.95, Authoritative),
            ("arxiv.org", 0.95, Authoritative),
            ("learn.microsoft.com", 0.95, Authoritative),
            ("docs.oracle.com", 0.95, Authoritative),
            ("wikipedia.org", 0.85, Curated),
            ("en.wikipedia.org", 0.85, Curated),
            ("crates.io", 0.85, Curated),
            ("npmjs.com", 0.85, Curated),
            ("pypi.org", 0.85, Curated),
            ("docs.rs", 0.85, Curated),
            ("bbc.com", 0.70, News),
            ("reuters.com", 0.70, News),
            ("nytimes.com", 0.70, News),
            ("theguardian.com", 0.70, News),
            ("arstechnica.com", 0.70, News),
            ("techcrunch.com", 0.70, News),
            ("reddit.com", 0.50, UserGenerated),
            ("quora.com", 0.50, UserGenerated),
            ("medium.com", 0.50, UserGenerated),
            ("dev.to", 0.50, UserGenerated),
            ("news.ycombinator.com", 0.50, UserGenerated),
            ("hashnode.dev", 0.50, UserGenerated),
            ("pinterest.com", 0.10, UserGenerated),
            ("w3schools.com", 0.10, UserGenerated),
        ];

        let mut domains = HashMap::with_capacity(seed.len());
        for &(domain, score, category) in seed {
            domains.insert(
                domain.to_owned(),
                DomainTrust {
                    domain: domain.to_owned(),
                    trust_score: score,
                    category,
                    sample_count: 0,
                },
            );
        }

        Self {
            domains,
            default_trust: TrustConfig::default().default_trust,
        }
    }

    /// Look up trust information for a URL.
    ///
    /// Extracts the domain, checks for an exact match, then falls back to
    /// parent-domain lookup and TLD heuristics (`.edu`/`.gov` = Authoritative).
    pub fn domain_trust(&self, url: &str) -> DomainTrust {
        let domain = extract_domain(url);
        if domain.is_empty() {
            return self.unknown_trust(String::new());
        }

        // Exact match.
        if let Some(entry) = self.domains.get(&domain) {
            return entry.clone();
        }

        // Try parent domain (e.g. `old.reddit.com` -> `reddit.com`).
        let parts: Vec<&str> = domain.split('.').collect();
        if parts.len() > 2 {
            let parent = parts[parts.len() - 2..].join(".");
            if let Some(entry) = self.domains.get(&parent) {
                return DomainTrust {
                    domain: domain.clone(),
                    trust_score: entry.trust_score,
                    category: entry.category,
                    sample_count: 0,
                };
            }
        }

        // TLD heuristics.
        if let Some(&tld) = parts.last() {
            if tld == "edu" || tld == "gov" {
                return DomainTrust {
                    domain,
                    trust_score: 0.95,
                    category: DomainCategory::Authoritative,
                    sample_count: 0,
                };
            }
        }

        self.unknown_trust(domain)
    }

    /// Modify each result's `score` based on domain trust.
    ///
    /// Results without an existing score use a 0.5 baseline. The modifier is:
    /// Authoritative -> `authoritative_boost`, Unknown -> `unknown_domain_factor`,
    /// trust < 0.2 -> `low_quality_penalty`, else linear interpolation.
    pub fn apply_trust_scores(results: &mut [ResultItem], config: &TrustConfig) {
        let db = Self::new();
        for item in results.iter_mut() {
            let trust = db.domain_trust(&item.url);
            let base = item.score.unwrap_or(0.5);
            let modifier = match trust.category {
                DomainCategory::Authoritative => config.authoritative_boost,
                DomainCategory::Unknown => config.unknown_domain_factor,
                _ if trust.trust_score < 0.2 => config.low_quality_penalty,
                _ => 0.7 + trust.trust_score * 0.5,
            };
            item.score = Some(base * modifier);
        }
    }

    fn unknown_trust(&self, domain: String) -> DomainTrust {
        DomainTrust {
            domain,
            trust_score: self.default_trust,
            category: DomainCategory::Unknown,
            sample_count: 0,
        }
    }
}

impl Default for TrustDatabase {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract the root domain from a URL string.
///
/// Returns an empty string if the URL cannot be parsed.
fn extract_domain(url: &str) -> String {
    if url.is_empty() {
        return String::new();
    }
    match url::Url::parse(url) {
        Ok(parsed) => parsed.host_str().unwrap_or("").to_lowercase(),
        Err(_) => {
            let stripped = url
                .trim_start_matches("https://")
                .trim_start_matches("http://");
            stripped
                .split(['/', ':', '?', '#'])
                .next()
                .unwrap_or("")
                .to_lowercase()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BackendId;

    fn item(url: &str, score: Option<f64>) -> ResultItem {
        ResultItem {
            title: "Test".into(),
            url: url.into(),
            snippet: String::new(),
            rank: 0,
            backend: BackendId::DuckDuckGo,
            score,
            published_date: None,
        }
    }

    #[test]
    fn known_domain_returns_correct_trust() {
        let db = TrustDatabase::new();
        let trust = db.domain_trust("https://github.com/rust-lang/rust");
        assert_eq!(trust.trust_score, 0.95);
        assert_eq!(trust.category, DomainCategory::Authoritative);
    }

    #[test]
    fn unknown_domain_returns_default() {
        let db = TrustDatabase::new();
        let trust = db.domain_trust("https://some-random-blog.xyz/article");
        assert_eq!(trust.trust_score, 0.5);
        assert_eq!(trust.category, DomainCategory::Unknown);
    }

    #[test]
    fn tld_edu_scored_as_authoritative() {
        let db = TrustDatabase::new();
        let trust = db.domain_trust("https://cs.stanford.edu/courses/");
        assert_eq!(trust.trust_score, 0.95);
        assert_eq!(trust.category, DomainCategory::Authoritative);
    }

    #[test]
    fn tld_gov_scored_as_authoritative() {
        let db = TrustDatabase::new();
        let trust = db.domain_trust("https://data.gov/dataset/population");
        assert_eq!(trust.trust_score, 0.95);
        assert_eq!(trust.category, DomainCategory::Authoritative);
    }

    #[test]
    fn extract_domain_parses_correctly() {
        assert_eq!(
            extract_domain("https://en.wikipedia.org/wiki/Rust"),
            "en.wikipedia.org"
        );
        assert_eq!(extract_domain("http://localhost:8080/path"), "localhost");
        assert_eq!(extract_domain(""), "");
    }

    #[test]
    fn trust_modifies_result_scores() {
        let config = TrustConfig::default();
        let mut results = vec![
            item("https://github.com/repo", Some(0.8)),
            item("https://some-unknown-site.com/page", Some(0.8)),
        ];
        TrustDatabase::apply_trust_scores(&mut results, &config);
        let github_score = results[0].score.unwrap();
        let unknown_score = results[1].score.unwrap();
        assert!(
            github_score > unknown_score,
            "github={github_score}, unknown={unknown_score}"
        );
    }

    #[test]
    fn authoritative_results_boosted() {
        let config = TrustConfig::default();
        let mut results = vec![item("https://doc.rust-lang.org/std/", Some(1.0))];
        TrustDatabase::apply_trust_scores(&mut results, &config);
        let score = results[0].score.unwrap();
        assert!(
            score > 1.0,
            "authoritative score should exceed base 1.0, got {score}"
        );
    }

    #[test]
    fn low_quality_penalized() {
        let config = TrustConfig::default();
        let mut results = vec![item("https://pinterest.com/pin/12345", Some(1.0))];
        TrustDatabase::apply_trust_scores(&mut results, &config);
        let score = results[0].score.unwrap();
        assert!(
            score < 1.0,
            "low-quality score should be below base 1.0, got {score}"
        );
    }

    #[test]
    fn empty_url_handled_gracefully() {
        let db = TrustDatabase::new();
        let trust = db.domain_trust("");
        assert_eq!(trust.category, DomainCategory::Unknown);
        assert_eq!(trust.trust_score, 0.5);
        assert!(trust.domain.is_empty());
    }

    #[test]
    fn subdomain_inherits_parent_trust() {
        let db = TrustDatabase::new();
        let trust = db.domain_trust("https://old.reddit.com/r/rust");
        assert_eq!(trust.trust_score, 0.50);
        assert_eq!(trust.category, DomainCategory::UserGenerated);
    }

    #[test]
    fn curated_domain_scores_correctly() {
        let db = TrustDatabase::new();
        let trust = db.domain_trust("https://crates.io/crates/serde");
        assert_eq!(trust.trust_score, 0.85);
        assert_eq!(trust.category, DomainCategory::Curated);
    }

    #[test]
    fn news_domain_scores_correctly() {
        let db = TrustDatabase::new();
        let trust = db.domain_trust("https://arstechnica.com/tech-policy/");
        assert_eq!(trust.trust_score, 0.70);
        assert_eq!(trust.category, DomainCategory::News);
    }

    #[test]
    fn apply_trust_with_no_existing_score() {
        let config = TrustConfig::default();
        let mut results = vec![item("https://github.com/foo", None)];
        TrustDatabase::apply_trust_scores(&mut results, &config);
        // baseline 0.5 * authoritative_boost 1.3 = 0.65
        let score = results[0].score.unwrap();
        assert!((score - 0.65).abs() < 1e-9, "expected 0.65, got {score}");
    }
}
