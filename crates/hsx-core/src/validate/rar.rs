//! RAR (Reflection-Augmented Research) self-correction loop (PRD §8.6).
//!
//! 5 reflection checkpoints: R1 NeedMore, R2 Relevant, R3 Sufficient,
//! R4 Supported, R5 Consistent.

use crate::validate::types::Contradiction;
use serde::{Deserialize, Serialize};

/// The 5 RAR reflection checkpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReflectionCheckpoint {
    /// R1: Are there enough relevant results?
    NeedMore,
    /// R2: Are the results relevant to the query?
    Relevant,
    /// R3: Does extracted content actually answer the query?
    Sufficient,
    /// R4: Does synthesis contain only source-supported claims?
    Supported,
    /// R5: Do sources agree with each other?
    Consistent,
}

/// Action the RAR loop decides to take.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RarAction {
    /// Retrieval is good; proceed.
    Proceed,
    /// Expand query and re-search (R1 action).
    ExpandQuery { new_query: String },
    /// Discard irrelevant results and reformulate (R2 action).
    ReformulateQuery { reason: String, new_query: String },
    /// Fetch additional pages for more evidence (R3 action).
    FetchMore { urls: Vec<String> },
    /// Remove unsupported claims from synthesis (R4 action).
    RemoveUnsupported { claim_ids: Vec<String> },
    /// Flag contradictions for the user (R5 action).
    FlagContradictions { contradictions: Vec<Contradiction> },
}

/// Result of one RAR loop iteration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RarIterationResult {
    pub iteration: usize,
    pub checkpoint: ReflectionCheckpoint,
    pub action: RarAction,
    pub quality_before: f64,
    pub quality_after: f64,
}

/// Configuration for the RAR loop.
#[derive(Debug, Clone)]
pub struct RarConfig {
    pub max_loops: usize,
    pub min_relevant_ratio: f64,
    pub min_sufficiency_score: f64,
    pub min_consistency_score: f64,
    pub min_results: usize,
}

impl Default for RarConfig {
    fn default() -> Self {
        Self {
            max_loops: 3,
            min_relevant_ratio: 0.5,
            min_sufficiency_score: 0.4,
            min_consistency_score: 0.5,
            min_results: 3,
        }
    }
}

/// Snapshot of retrieval quality passed to the RAR engine.
#[derive(Debug, Clone)]
pub struct RarState {
    pub query: String,
    pub total_results: usize,
    pub relevant_count: usize,
    pub sufficiency_score: f64,
    pub support_ratio: f64,
    pub consistency_score: f64,
    pub unsupported_claims: Vec<String>,
    pub contradictions: Vec<Contradiction>,
    pub candidate_urls: Vec<String>,
    pub low_relevance_terms: Vec<String>,
}

impl RarState {
    /// Overall quality: average of relevance, sufficiency, and consistency.
    pub fn overall_quality(&self) -> f64 {
        let relevance = self.relevant_count as f64 / self.total_results.max(1) as f64;
        (relevance + self.sufficiency_score + self.consistency_score) / 3.0
    }
}

/// The RAR engine evaluates retrieval quality and triggers corrective actions.
#[derive(Default)]
pub struct RarEngine {
    config: RarConfig,
}

impl RarEngine {
    pub fn new(config: RarConfig) -> Self {
        Self { config }
    }

    /// Evaluate one RAR iteration and return the recommended action.
    ///
    /// The caller is responsible for executing actions (re-search, re-fetch)
    /// and calling this again with updated state.
    pub fn evaluate(&self, state: &RarState, iteration: usize) -> RarIterationResult {
        if iteration >= self.config.max_loops {
            return RarIterationResult {
                iteration,
                checkpoint: ReflectionCheckpoint::Consistent,
                action: RarAction::Proceed,
                quality_before: state.overall_quality(),
                quality_after: state.overall_quality(),
            };
        }

        // R1: Need more results?
        if state.total_results < self.config.min_results {
            let new_query = Self::expand_query(&state.query);
            return RarIterationResult {
                iteration,
                checkpoint: ReflectionCheckpoint::NeedMore,
                action: RarAction::ExpandQuery { new_query },
                quality_before: state.overall_quality(),
                quality_after: state.overall_quality(),
            };
        }

        // R2: Are results relevant?
        let relevant_ratio =
            state.relevant_count as f64 / state.total_results.max(1) as f64;
        if relevant_ratio < self.config.min_relevant_ratio {
            let new_query = Self::reformulate(&state.query, &state.low_relevance_terms);
            return RarIterationResult {
                iteration,
                checkpoint: ReflectionCheckpoint::Relevant,
                action: RarAction::ReformulateQuery {
                    reason: format!(
                        "Only {:.0}% relevant (need {:.0}%)",
                        relevant_ratio * 100.0,
                        self.config.min_relevant_ratio * 100.0
                    ),
                    new_query,
                },
                quality_before: relevant_ratio,
                quality_after: relevant_ratio,
            };
        }

        // R3: Is content sufficient?
        if state.sufficiency_score < self.config.min_sufficiency_score {
            return RarIterationResult {
                iteration,
                checkpoint: ReflectionCheckpoint::Sufficient,
                action: RarAction::FetchMore {
                    urls: state.candidate_urls.clone(),
                },
                quality_before: state.sufficiency_score,
                quality_after: state.sufficiency_score,
            };
        }

        // R4: Are claims supported?
        if !state.unsupported_claims.is_empty() {
            return RarIterationResult {
                iteration,
                checkpoint: ReflectionCheckpoint::Supported,
                action: RarAction::RemoveUnsupported {
                    claim_ids: state.unsupported_claims.clone(),
                },
                quality_before: state.support_ratio,
                quality_after: 1.0,
            };
        }

        // R5: Are sources consistent?
        if state.consistency_score < self.config.min_consistency_score {
            return RarIterationResult {
                iteration,
                checkpoint: ReflectionCheckpoint::Consistent,
                action: RarAction::FlagContradictions {
                    contradictions: state.contradictions.clone(),
                },
                quality_before: state.consistency_score,
                quality_after: state.consistency_score,
            };
        }

        // All checks pass
        RarIterationResult {
            iteration,
            checkpoint: ReflectionCheckpoint::Consistent,
            action: RarAction::Proceed,
            quality_before: state.overall_quality(),
            quality_after: state.overall_quality(),
        }
    }

    fn expand_query(query: &str) -> String {
        format!("{query} overview")
    }

    fn reformulate(query: &str, _low_terms: &[String]) -> String {
        let words: Vec<&str> = query
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .take(5)
            .collect();
        if words.is_empty() {
            query.to_string()
        } else {
            words.join(" ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good_state() -> RarState {
        RarState {
            query: "what is Rust".into(),
            total_results: 10,
            relevant_count: 8,
            sufficiency_score: 0.8,
            support_ratio: 1.0,
            consistency_score: 0.9,
            unsupported_claims: vec![],
            contradictions: vec![],
            candidate_urls: vec![],
            low_relevance_terms: vec![],
        }
    }

    #[test]
    fn good_state_proceeds() {
        let engine = RarEngine::default();
        let r = engine.evaluate(&good_state(), 0);
        assert!(matches!(r.action, RarAction::Proceed));
    }

    #[test]
    fn insufficient_results_expands() {
        let engine = RarEngine::default();
        let mut state = good_state();
        state.total_results = 1;
        state.relevant_count = 1;
        let r = engine.evaluate(&state, 0);
        assert!(matches!(r.action, RarAction::ExpandQuery { .. }));
        assert_eq!(r.checkpoint, ReflectionCheckpoint::NeedMore);
    }

    #[test]
    fn low_relevance_reformulates() {
        let engine = RarEngine::default();
        let mut state = good_state();
        state.total_results = 10;
        state.relevant_count = 2;
        let r = engine.evaluate(&state, 0);
        assert!(matches!(r.action, RarAction::ReformulateQuery { .. }));
    }

    #[test]
    fn max_loops_respected() {
        let engine = RarEngine::new(RarConfig {
            max_loops: 2,
            ..Default::default()
        });
        let mut state = good_state();
        state.total_results = 1;
        let r = engine.evaluate(&state, 2);
        assert!(matches!(r.action, RarAction::Proceed));
    }

    #[test]
    fn unsupported_claims_removed() {
        let engine = RarEngine::default();
        let mut state = good_state();
        state.unsupported_claims = vec!["claim_1".into()];
        let r = engine.evaluate(&state, 0);
        assert!(matches!(r.action, RarAction::RemoveUnsupported { .. }));
    }

    #[test]
    fn low_consistency_flags_contradictions() {
        let engine = RarEngine::default();
        let mut state = good_state();
        state.consistency_score = 0.2;
        let r = engine.evaluate(&state, 0);
        assert!(matches!(r.action, RarAction::FlagContradictions { .. }));
    }
}
