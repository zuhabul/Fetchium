//! V4 Cross-source verification — claim clustering, contradiction detection, consensus scoring.

use crate::validate::types::*;
use sha2::{Digest, Sha256};
use std::collections::HashSet;

/// Input from a single source for cross-source verification.
#[derive(Debug, Clone)]
pub struct SourceContent {
    pub url: String,
    pub index: usize,
    pub title: String,
    pub claims: Vec<String>,
    pub full_text: String,
    pub confidence: f64,
}

/// V4 Cross-Source Verifier.
pub struct CrossSourceVerifier {
    similarity_threshold: f64,
    min_sources: usize,
}

impl Default for CrossSourceVerifier {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.4,
            min_sources: 2,
        }
    }
}

impl CrossSourceVerifier {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_threshold(mut self, t: f64) -> Self {
        self.similarity_threshold = t;
        self
    }

    /// Run V4 cross-source verification.
    pub fn verify(&self, sources: &[SourceContent]) -> LayerResult {
        let start = std::time::Instant::now();

        if sources.len() < self.min_sources {
            return LayerResult {
                layer: ValidationLayerId::V4CrossSource,
                passed: true,
                score: 0.5,
                issues: vec![ValidationIssue {
                    severity: IssueSeverity::Info,
                    code: "V4_INSUFFICIENT_SOURCES".into(),
                    message: format!(
                        "Need {} sources, have {}",
                        self.min_sources,
                        sources.len()
                    ),
                    source_url: None,
                }],
                duration_ms: start.elapsed().as_millis() as u64,
            };
        }

        let all_claims = self.extract_claims(sources);
        let clusters = self.cluster_claims(&all_claims);
        let mut issues = Vec::new();
        let mut scores = Vec::new();

        for cluster in &clusters {
            let consensus = self.score_consensus(cluster, sources.len());
            if consensus.consensus_score <= 0.5 && consensus.contradicting_sources > 0 {
                for c in self.detect_contradictions(cluster) {
                    let mut is_resolved = false;
                    let mut resolution_msg = String::new();

                    // Intelligence layer: Contradiction Resolution Protocol (CRP)
                    let d_a = url::Url::parse(&c.claim_a.source_url)
                        .map(|u| u.host_str().unwrap_or("unknown").to_string())
                        .unwrap_or_else(|_| "unknown".to_string());
                    let d_b = url::Url::parse(&c.claim_b.source_url)
                        .map(|u| u.host_str().unwrap_or("unknown").to_string())
                        .unwrap_or_else(|_| "unknown".to_string());

                    let crp_contradiction = crate::intelligence::crp::CrpContradiction {
                        claim_a: c.claim_a.text.clone(),
                        source_a_domain: d_a.clone(),
                        source_a_trust: c.claim_a.confidence,
                        source_a_date: None,
                        claim_b: c.claim_b.text.clone(),
                        source_b_domain: d_b.clone(),
                        source_b_trust: c.claim_b.confidence,
                        source_b_date: None,
                        severity: match c.severity {
                            ContradictionSeverity::High => crate::intelligence::crp::Severity::High,
                            ContradictionSeverity::Medium => crate::intelligence::crp::Severity::Medium,
                            ContradictionSeverity::Low => crate::intelligence::crp::Severity::Low,
                        },
                    };

                    let pie = crate::intelligence::pie::PersistentIntelligenceEngine::new().ok();
                    if let Ok(res) = crate::intelligence::crp::resolve(&crp_contradiction, |domain| {
                        if let Some(pie_engine) = &pie {
                            pie_engine.stm.get_trust(domain).unwrap_or(0.5)
                        } else {
                            0.5
                        }
                    }) {
                        if res.resolution_type != crate::intelligence::crp::ResolutionType::Unresolved {
                            is_resolved = true;
                            resolution_msg = res.synthesis;
                        }
                    }

                    if is_resolved {
                        issues.push(ValidationIssue {
                            severity: IssueSeverity::Info,
                            code: "V4_CONTRADICTION_RESOLVED".into(),
                            message: format!("{} (CRP Resolved: {})", c.description, resolution_msg),
                            source_url: Some(c.claim_a.source_url.clone()),
                        });
                    } else {
                        issues.push(ValidationIssue {
                            severity: match c.severity {
                                ContradictionSeverity::High => IssueSeverity::Error,
                                ContradictionSeverity::Medium => IssueSeverity::Warning,
                                ContradictionSeverity::Low => IssueSeverity::Info,
                            },
                            code: "V4_CONTRADICTION".into(),
                            message: c.description.clone(),
                            source_url: Some(c.claim_a.source_url.clone()),
                        });
                    }
                }
            }
            scores.push(consensus.consensus_score);
        }

        let avg = if scores.is_empty() {
            1.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };
        let high_ct = issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Error)
            .count();

        LayerResult {
            layer: ValidationLayerId::V4CrossSource,
            passed: high_ct == 0 && avg >= 0.4,
            score: avg,
            issues,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    fn extract_claims(&self, sources: &[SourceContent]) -> Vec<Claim> {
        sources
            .iter()
            .flat_map(|s| {
                s.claims.iter().enumerate().map(move |(i, text)| Claim {
                    id: format!("{}:{}", &Self::hash_short(&s.url), i),
                    text: text.clone(),
                    normalized: Self::normalize(text),
                    source_url: s.url.clone(),
                    source_index: s.index,
                    confidence: s.confidence,
                })
            })
            .collect()
    }

    fn normalize(text: &str) -> String {
        text.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn cluster_claims(&self, claims: &[Claim]) -> Vec<Vec<Claim>> {
        let mut assigned = vec![false; claims.len()];
        let mut clusters = Vec::new();

        for i in 0..claims.len() {
            if assigned[i] {
                continue;
            }
            let mut cluster = vec![claims[i].clone()];
            assigned[i] = true;

            for j in (i + 1)..claims.len() {
                if assigned[j] || claims[i].source_index == claims[j].source_index {
                    continue;
                }
                if Self::jaccard(&claims[i].normalized, &claims[j].normalized)
                    >= self.similarity_threshold
                {
                    cluster.push(claims[j].clone());
                    assigned[j] = true;
                }
            }

            if cluster.len() > 1 {
                clusters.push(cluster);
            }
        }

        clusters
    }

    fn jaccard(a: &str, b: &str) -> f64 {
        let a_words: Vec<&str> = a.split_whitespace().collect();
        let b_words: Vec<&str> = b.split_whitespace().collect();
        let ba: HashSet<String> = a_words
            .windows(2)
            .map(|w| format!("{} {}", w[0], w[1]))
            .collect();
        let bb: HashSet<String> = b_words
            .windows(2)
            .map(|w| format!("{} {}", w[0], w[1]))
            .collect();
        let inter = ba.intersection(&bb).count();
        let union = ba.union(&bb).count();
        if union == 0 {
            // Both inputs produced no bigrams (single words or empty).
            // Treat as identical only if the normalized strings match.
            if a == b { 1.0 } else { 0.0 }
        } else {
            inter as f64 / union as f64
        }
    }

    fn score_consensus(&self, cluster: &[Claim], total: usize) -> ClaimConsensus {
        let anchor = &cluster[0];
        let mut sup = HashSet::new();
        let mut con = HashSet::new();
        sup.insert(anchor.source_index);

        for c in &cluster[1..] {
            if Self::claims_agree(&anchor.normalized, &c.normalized) >= 0.5 {
                sup.insert(c.source_index);
            } else {
                con.insert(c.source_index);
            }
        }

        ClaimConsensus {
            claim: anchor.clone(),
            supporting_sources: sup.len(),
            contradicting_sources: con.len(),
            total_sources: total,
            consensus_score: sup.len() as f64 / total as f64,
        }
    }

    fn claims_agree(a: &str, b: &str) -> f64 {
        // Negation patterns: standalone words and two-word phrases.
        let neg_words = ["not", "no", "never", "neither", "doesnt", "isnt", "cant", "wont",
                         "did not", "has not", "have not", "is not", "are not", "was not",
                         "does not", "will not", "would not", "should not", "could not"];
        let a_neg = neg_words.iter().any(|w| a.contains(w));
        let b_neg = neg_words.iter().any(|w| b.contains(w));
        if a_neg != b_neg {
            return 0.2;
        }
        Self::jaccard(a, b).min(1.0)
    }

    fn detect_contradictions(&self, cluster: &[Claim]) -> Vec<Contradiction> {
        let mut out = Vec::new();
        for i in 0..cluster.len() {
            for j in (i + 1)..cluster.len() {
                if cluster[i].source_index == cluster[j].source_index {
                    continue;
                }
                let agr = Self::claims_agree(&cluster[i].normalized, &cluster[j].normalized);
                if agr < 0.5 {
                    let sev = if agr < 0.2 {
                        ContradictionSeverity::High
                    } else if agr < 0.35 {
                        ContradictionSeverity::Medium
                    } else {
                        ContradictionSeverity::Low
                    };
                    out.push(Contradiction {
                        claim_a: cluster[i].clone(),
                        claim_b: cluster[j].clone(),
                        severity: sev,
                        description: format!(
                            "{sev:?}: \"{}\" vs \"{}\"",
                            cluster[i].text, cluster[j].text
                        ),
                    });
                }
            }
        }
        out
    }

    fn hash_short(input: &str) -> String {
        let mut h = Sha256::new();
        h.update(input.as_bytes());
        let bytes = h.finalize();
        bytes[..4].iter().map(|b| format!("{b:02x}")).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src(i: usize, url: &str, claims: &[&str], conf: f64) -> SourceContent {
        SourceContent {
            url: url.into(),
            index: i,
            title: format!("S{i}"),
            claims: claims.iter().map(|c| c.to_string()).collect(),
            full_text: claims.join(". "),
            confidence: conf,
        }
    }

    #[test]
    fn consistent_sources_pass() {
        let v = CrossSourceVerifier::new();
        let r = v.verify(&[
            src(0, "https://a.com", &["Rust released in 2015"], 0.9),
            src(1, "https://b.com", &["Rust released in 2015"], 0.8),
        ]);
        assert!(r.passed);
    }

    #[test]
    fn contradictions_detected() {
        // Use sentences with high bigram overlap but opposite negation so they
        // cluster (jaccard >= threshold) and then trigger contradiction detection.
        let v = CrossSourceVerifier::new();
        let r = v.verify(&[
            src(0, "https://a.com", &["Rust is a safe language"], 0.9),
            src(1, "https://b.com", &["Rust is not a safe language"], 0.8),
        ]);
        assert!(!r.issues.is_empty());
    }

    #[test]
    fn insufficient_sources() {
        let v = CrossSourceVerifier::new();
        let r = v.verify(&[src(0, "https://a.com", &["claim"], 0.9)]);
        assert!(r.passed);
        assert_eq!(r.score, 0.5);
    }

    #[test]
    fn jaccard_identical() {
        assert!((CrossSourceVerifier::jaccard("the quick brown fox", "the quick brown fox") - 1.0).abs() < 1e-9);
    }

    #[test]
    fn jaccard_disjoint() {
        // single words produce no bigrams
        assert_eq!(CrossSourceVerifier::jaccard("rust", "python"), 0.0);
    }
}
