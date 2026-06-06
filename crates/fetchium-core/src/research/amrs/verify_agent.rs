//! Verify Agent — cross-source contradiction detection and confidence scoring (PRD §8.8).

use crate::error::FetchiumError;
use crate::research::amrs::agent::Agent;
use crate::research::amrs::channel::{
    AgentMessage, AgentReceiver, AgentSender, AgentType, AmrsContradiction, AmrsFinding, AmrsSource,
};
use async_trait::async_trait;

/// Detects contradictions between sources and assigns confidence scores.
pub struct VerifyAgent;

impl VerifyAgent {
    pub fn new() -> Self {
        Self
    }

    /// Extract key claims from source content (first N sentences).
    fn extract_claims(source: &AmrsSource, max_claims: usize) -> Vec<String> {
        source
            .content
            .split(". ")
            .map(|s| s.trim().to_string())
            .filter(|s| s.len() > 20)
            .take(max_claims)
            .collect()
    }

    /// Check if two text passages contradict each other.
    ///
    /// Uses heuristic negation and keyword conflict detection.
    fn detect_contradiction(
        text_a: &str,
        text_b: &str,
        url_a: &str,
        url_b: &str,
    ) -> Option<AmrsContradiction> {
        let lower_a = text_a.to_lowercase();
        let lower_b = text_b.to_lowercase();

        // Negation patterns in B relative to A's keywords
        let negation_phrases = ["not ", "never ", "incorrect ", "false ", "wrong "];
        let words_a: std::collections::HashSet<&str> = lower_a.split_whitespace().collect();

        let mut conflict_score = 0.0f64;
        for phrase in &negation_phrases {
            if lower_b.contains(phrase) {
                // Check if negation is about same topic
                let words_b: Vec<&str> = lower_b.split_whitespace().collect();
                let overlap = words_b.iter().filter(|w| words_a.contains(**w)).count();
                if overlap > 3 {
                    conflict_score += 0.3;
                }
            }
        }

        // Number conflict: look for different numeric values on same topic
        let numbers_a: Vec<&str> = lower_a
            .split_whitespace()
            .filter(|w| w.chars().all(|c| c.is_ascii_digit() || c == '.'))
            .collect();
        let numbers_b: Vec<&str> = lower_b
            .split_whitespace()
            .filter(|w| w.chars().all(|c| c.is_ascii_digit() || c == '.'))
            .collect();
        if !numbers_a.is_empty() && !numbers_b.is_empty() && numbers_a != numbers_b {
            conflict_score += 0.2;
        }

        if conflict_score >= 0.3 {
            let claim = text_a.chars().take(100).collect::<String>();
            Some(AmrsContradiction {
                claim,
                source_a: url_a.to_string(),
                source_b: url_b.to_string(),
                source_a_says: text_a.chars().take(150).collect(),
                source_b_says: text_b.chars().take(150).collect(),
                severity: conflict_score.min(1.0),
            })
        } else {
            None
        }
    }

    /// Verify sources and return findings + contradictions.
    fn verify_sources(
        sources: &[AmrsSource],
        query: &str,
    ) -> (Vec<AmrsFinding>, Vec<AmrsContradiction>) {
        let mut findings = Vec::new();
        let mut contradictions = Vec::new();

        let query_words: std::collections::HashSet<&str> = query.split_whitespace().collect();

        for (idx, source) in sources.iter().enumerate() {
            let claims = Self::extract_claims(source, 5);

            for claim in &claims {
                // Score relevance: word overlap with query
                let claim_words: std::collections::HashSet<&str> =
                    claim.split_whitespace().collect();
                let overlap = claim_words.intersection(&query_words).count();
                let relevance = (overlap as f64 / query_words.len().max(1) as f64).min(1.0);

                if relevance > 0.1 || !findings.is_empty() {
                    findings.push(AmrsFinding {
                        claim: claim.clone(),
                        confidence: 0.5 + relevance * 0.4,
                        source_indices: vec![idx],
                        evidence_type: "direct".into(),
                    });
                }
            }

            // Check for contradictions with other sources
            for other in sources.iter().skip(idx + 1) {
                let claims_a = Self::extract_claims(source, 3);
                let claims_b = Self::extract_claims(other, 3);

                for ca in &claims_a {
                    for cb in &claims_b {
                        if let Some(contradiction) =
                            Self::detect_contradiction(ca, cb, &source.url, &other.url)
                        {
                            contradictions.push(contradiction);
                        }
                    }
                }
            }
        }

        (findings, contradictions)
    }
}

impl Default for VerifyAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for VerifyAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Verify
    }

    async fn run(&self, mut rx: AgentReceiver, tx: AgentSender) -> Result<(), FetchiumError> {
        while let Some(msg) = rx.recv().await {
            match msg {
                AgentMessage::SpawnVerify { sources, query } => {
                    let _ = tx
                        .send(AgentMessage::ProgressUpdate {
                            agent_type: AgentType::Verify,
                            message: format!("Verifying {} sources...", sources.len()),
                            progress: 0.5,
                        })
                        .await;

                    let (findings, contradictions) = Self::verify_sources(&sources, &query);

                    let _ = tx
                        .send(AgentMessage::VerifyComplete {
                            findings,
                            contradictions,
                        })
                        .await;
                }
                AgentMessage::Shutdown => break,
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_source(url: &str, content: &str) -> AmrsSource {
        AmrsSource {
            url: url.into(),
            title: "Test".into(),
            content: content.into(),
            content_hash: "abc".into(),
        }
    }

    #[test]
    fn extract_claims_returns_sentences() {
        let s = make_source("https://a.com", "Rust is fast. Go is simple. Python is slow. Short. This is a longer sentence that qualifies.");
        let claims = VerifyAgent::extract_claims(&s, 5);
        assert!(!claims.is_empty());
        assert!(claims.iter().all(|c| c.len() > 20));
    }

    #[test]
    fn no_contradiction_for_agreeing_sources() {
        let result = VerifyAgent::detect_contradiction(
            "Rust is memory safe",
            "Rust provides memory safety guarantees",
            "https://a.com",
            "https://b.com",
        );
        assert!(result.is_none());
    }
}
