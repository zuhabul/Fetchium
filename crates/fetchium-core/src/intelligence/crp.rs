//! Contradiction Resolution Protocol (CRP) — PRD §8.13.
//!
//! 5-step protocol for resolving contradictions between sources:
//! 1. Date check — newer source supersedes older
//! 2. Authority check — higher-trust domain wins
//! 3. Context check — different scopes may not truly contradict
//! 4. Investigation — spawn additional search to find more evidence
//! 5. Weighted synthesis — produce nuanced conclusion citing both sources
//!
//! Short-circuits at any step if the resolution is conclusive.

use crate::error::HsxError;

// ─── Types ───────────────────────────────────────────────────────────────────

/// Severity of a contradiction between two sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// A contradiction between two source claims that needs resolution.
#[derive(Debug, Clone)]
pub struct CrpContradiction {
    pub claim_a: String,
    pub source_a_domain: String,
    pub source_a_trust: f64,
    /// ISO date string or None if unknown.
    pub source_a_date: Option<String>,
    pub claim_b: String,
    pub source_b_domain: String,
    pub source_b_trust: f64,
    pub source_b_date: Option<String>,
    pub severity: Severity,
}

/// How a contradiction was resolved.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionType {
    /// Newer source supersedes.
    TemporalPrecedence,
    /// More trusted domain wins.
    AuthorityPrecedence,
    /// Claims refer to different scopes — not a real contradiction.
    ScopeDependent,
    /// Additional investigation found decisive evidence.
    InvestigationResolvedIt,
    /// Genuine disagreement — both sides presented.
    Unresolved,
}

/// A single step in the resolution pipeline.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ResolutionStep {
    pub name: String,
    pub conclusion: String,
    pub confidence: f64,
    pub conclusive: bool,
}

/// The final resolution of a contradiction.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Resolution {
    pub claim_a: String,
    pub claim_b: String,
    pub steps_taken: Vec<ResolutionStep>,
    pub synthesis: String,
    pub confidence: f64,
    pub resolution_type: ResolutionType,
}

// ─── Pipeline ────────────────────────────────────────────────────────────────

/// Resolve a contradiction using the 5-step CRP.
///
/// The `stm_trust_fn` callback retrieves the PIE trust score for a domain.
/// Pass `|_| 0.5` when PIE is unavailable.
pub fn resolve<F>(contradiction: &CrpContradiction, stm_trust_fn: F) -> Result<Resolution, HsxError>
where
    F: Fn(&str) -> f64,
{
    let mut steps = Vec::new();

    // Step 1: Date check
    let date_step = step_date_check(contradiction);
    let conclusive = date_step.conclusive;
    steps.push(date_step);
    if conclusive {
        return Ok(finalize(
            contradiction,
            steps,
            ResolutionType::TemporalPrecedence,
        ));
    }

    // Step 2: Authority check (uses PIE STM trust score)
    let auth_step = step_authority_check(contradiction, &stm_trust_fn);
    let conclusive = auth_step.conclusive;
    steps.push(auth_step);
    if conclusive {
        return Ok(finalize(
            contradiction,
            steps,
            ResolutionType::AuthorityPrecedence,
        ));
    }

    // Step 3: Context check
    let ctx_step = step_context_check(contradiction);
    let conclusive = ctx_step.conclusive;
    steps.push(ctx_step);
    if conclusive {
        return Ok(finalize(
            contradiction,
            steps,
            ResolutionType::ScopeDependent,
        ));
    }

    // Step 4: Investigation (keyword-based additional queries)
    let inv_step = step_investigation(contradiction);
    let inv_conclusive = inv_step.conclusive;
    steps.push(inv_step);

    // Step 5: Weighted synthesis (always runs)
    let synth_step = step_weighted_synthesis(contradiction, &steps);
    steps.push(synth_step);

    let resolution_type = if inv_conclusive {
        ResolutionType::InvestigationResolvedIt
    } else {
        ResolutionType::Unresolved
    };

    Ok(finalize(contradiction, steps, resolution_type))
}

/// Build the final `Resolution` from accumulated steps.
fn finalize(
    c: &CrpContradiction,
    steps: Vec<ResolutionStep>,
    resolution_type: ResolutionType,
) -> Resolution {
    let synthesis = steps
        .last()
        .map(|s| s.conclusion.clone())
        .unwrap_or_default();
    let confidence = if steps.is_empty() {
        0.5
    } else {
        steps.iter().map(|s| s.confidence).sum::<f64>() / steps.len() as f64
    };
    Resolution {
        claim_a: c.claim_a.clone(),
        claim_b: c.claim_b.clone(),
        steps_taken: steps,
        synthesis,
        confidence,
        resolution_type,
    }
}

// ─── Step implementations ─────────────────────────────────────────────────────

fn step_date_check(c: &CrpContradiction) -> ResolutionStep {
    match (&c.source_a_date, &c.source_b_date) {
        (Some(da), Some(db)) => {
            // Compare dates lexicographically (ISO 8601 is lexicographically ordered).
            if da == db {
                return ResolutionStep {
                    name: "Date Check".into(),
                    conclusion: "Sources are from the same date; date alone is inconclusive."
                        .into(),
                    confidence: 0.3,
                    conclusive: false,
                };
            }
            let (newer_domain, newer_claim) = if db > da {
                (&c.source_b_domain, &c.claim_b)
            } else {
                (&c.source_a_domain, &c.claim_a)
            };
            ResolutionStep {
                name: "Date Check".into(),
                conclusion: format!(
                    "Newer source ({newer_domain}) supersedes older source. \
                     Adopting: \"{newer_claim}\""
                ),
                confidence: 0.7,
                conclusive: true,
            }
        }
        _ => ResolutionStep {
            name: "Date Check".into(),
            conclusion: "Publication dates unavailable; cannot determine temporal precedence."
                .into(),
            confidence: 0.2,
            conclusive: false,
        },
    }
}

fn step_authority_check<F>(c: &CrpContradiction, trust_fn: &F) -> ResolutionStep
where
    F: Fn(&str) -> f64,
{
    let trust_a = c.source_a_trust.max(trust_fn(&c.source_a_domain));
    let trust_b = c.source_b_trust.max(trust_fn(&c.source_b_domain));

    let diff = (trust_a - trust_b).abs();
    if diff < 0.2 {
        return ResolutionStep {
            name: "Authority Check".into(),
            conclusion: format!(
                "Sources have similar trust levels ({:.2} vs {:.2}); authority inconclusive.",
                trust_a, trust_b
            ),
            confidence: 0.3,
            conclusive: false,
        };
    }

    let (winner_domain, winner_claim, winner_trust) = if trust_a > trust_b {
        (&c.source_a_domain, &c.claim_a, trust_a)
    } else {
        (&c.source_b_domain, &c.claim_b, trust_b)
    };

    ResolutionStep {
        name: "Authority Check".into(),
        conclusion: format!(
            "Higher-trust source ({winner_domain}, trust={winner_trust:.2}) preferred. \
             Adopting: \"{winner_claim}\""
        ),
        confidence: 0.65 + diff * 0.2,
        conclusive: true,
    }
}

fn step_context_check(c: &CrpContradiction) -> ResolutionStep {
    // Heuristic: if both claims share fewer than 20% of significant words, they may
    // be discussing different populations/scopes.
    let shared = shared_word_ratio(&c.claim_a, &c.claim_b);
    if shared < 0.20 {
        ResolutionStep {
            name: "Context Check".into(),
            conclusion: format!(
                "Claims share only {:.0}% vocabulary — they may refer to different contexts. \
                 Both claims can coexist without contradiction.",
                shared * 100.0
            ),
            confidence: 0.55,
            conclusive: true,
        }
    } else {
        ResolutionStep {
            name: "Context Check".into(),
            conclusion: format!(
                "Claims share {:.0}% vocabulary — they appear to discuss the same topic. \
                 The contradiction is genuine.",
                shared * 100.0
            ),
            confidence: 0.35,
            conclusive: false,
        }
    }
}

fn step_investigation(c: &CrpContradiction) -> ResolutionStep {
    // Without an async runtime here, we do a keyword-based analysis.
    // In a full async context, this could spawn additional search queries.
    let severity_hint = match c.severity {
        Severity::Critical | Severity::High => {
            "⚠ High-severity contradiction — manual verification strongly recommended."
        }
        Severity::Medium => {
            "Consider running `fetchium research` with both claims to gather more evidence."
        }
        Severity::Low => "Low-severity contradiction — likely a minor phrasing difference.",
    };

    ResolutionStep {
        name: "Investigation".into(),
        conclusion: format!(
            "Suggested additional queries: \"{} evidence\", \"{} evidence\". {severity_hint}",
            keywords_from_claim(&c.claim_a),
            keywords_from_claim(&c.claim_b),
        ),
        confidence: 0.4,
        conclusive: false,
    }
}

fn step_weighted_synthesis(c: &CrpContradiction, prior_steps: &[ResolutionStep]) -> ResolutionStep {
    let avg_conf: f64 = if prior_steps.is_empty() {
        0.5
    } else {
        prior_steps.iter().map(|s| s.confidence).sum::<f64>() / prior_steps.len() as f64
    };

    let synthesis = format!(
        "Sources disagree: \"{}\" ({}, trust={:.2}) vs \"{}\" ({}, trust={:.2}). \
         Both perspectives are presented; final determination requires domain expertise. \
         Overall confidence: {:.0}%.",
        c.claim_a,
        c.source_a_domain,
        c.source_a_trust,
        c.claim_b,
        c.source_b_domain,
        c.source_b_trust,
        avg_conf * 100.0,
    );

    ResolutionStep {
        name: "Weighted Synthesis".into(),
        conclusion: synthesis,
        confidence: avg_conf,
        conclusive: false,
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn shared_word_ratio(a: &str, b: &str) -> f64 {
    let stop_words: std::collections::HashSet<&str> = [
        "the", "a", "an", "is", "was", "are", "were", "of", "in", "to", "and", "or",
    ]
    .iter()
    .cloned()
    .collect();

    let words_a: std::collections::HashSet<String> = a
        .split_whitespace()
        .map(|w| {
            w.to_lowercase()
                .trim_matches(|c: char| !c.is_alphabetic())
                .to_string()
        })
        .filter(|w| w.len() >= 3 && !stop_words.contains(w.as_str()))
        .collect();

    let words_b: std::collections::HashSet<String> = b
        .split_whitespace()
        .map(|w| {
            w.to_lowercase()
                .trim_matches(|c: char| !c.is_alphabetic())
                .to_string()
        })
        .filter(|w| w.len() >= 3 && !stop_words.contains(w.as_str()))
        .collect();

    if words_a.is_empty() || words_b.is_empty() {
        return 0.0;
    }

    let intersection: usize = words_a.intersection(&words_b).count();
    let union: usize = words_a.union(&words_b).count();
    intersection as f64 / union.max(1) as f64
}

fn keywords_from_claim(claim: &str) -> String {
    claim
        .split_whitespace()
        .filter(|w| w.len() >= 4)
        .take(4)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_contradiction(
        claim_a: &str,
        claim_b: &str,
        date_a: Option<&str>,
        date_b: Option<&str>,
        trust_a: f64,
        trust_b: f64,
    ) -> CrpContradiction {
        CrpContradiction {
            claim_a: claim_a.into(),
            source_a_domain: "source-a.com".into(),
            source_a_trust: trust_a,
            source_a_date: date_a.map(String::from),
            claim_b: claim_b.into(),
            source_b_domain: "source-b.com".into(),
            source_b_trust: trust_b,
            source_b_date: date_b.map(String::from),
            severity: Severity::Medium,
        }
    }

    #[test]
    fn newer_date_wins() {
        let c = make_contradiction(
            "X is true",
            "X is false",
            Some("2023-01-01"),
            Some("2024-06-01"),
            0.5,
            0.5,
        );
        let res = resolve(&c, |_| 0.5).unwrap();
        assert_eq!(res.resolution_type, ResolutionType::TemporalPrecedence);
    }

    #[test]
    fn higher_trust_wins() {
        let c = make_contradiction("Y is true", "Y is false", None, None, 0.9, 0.3);
        let res = resolve(&c, |_| 0.5).unwrap();
        assert_eq!(res.resolution_type, ResolutionType::AuthorityPrecedence);
    }

    #[test]
    fn unrelated_claims_are_scope_dependent() {
        let c = make_contradiction(
            "quantum computing breaks RSA encryption",
            "coffee is the most consumed beverage in the world",
            None,
            None,
            0.6,
            0.6,
        );
        let res = resolve(&c, |_| 0.5).unwrap();
        assert_eq!(res.resolution_type, ResolutionType::ScopeDependent);
    }

    #[test]
    fn resolution_has_all_steps_taken() {
        let c = make_contradiction("Rust is fast", "Rust is slow", None, None, 0.5, 0.5);
        let res = resolve(&c, |_| 0.5).unwrap();
        assert!(!res.steps_taken.is_empty());
        assert!(!res.synthesis.is_empty());
    }
}
