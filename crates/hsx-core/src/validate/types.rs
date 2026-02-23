//! Shared validation types for the 6-layer pipeline (PRD §19).

use serde::{Deserialize, Serialize};

/// Complete result of the 6-layer validation pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub layers_run: Vec<ValidationLayerId>,
    pub layer_results: Vec<LayerResult>,
    pub passed: bool,
    pub confidence: f64,
    pub contradictions: Vec<Contradiction>,
    pub consensus: Vec<ClaimConsensus>,
    pub mode: ValidationMode,
}

/// Identifier for each of the 6 validation layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValidationLayerId {
    V1Source,
    V2Content,
    V3Freshness,
    V4CrossSource,
    V5ExtractionQuality,
    V6OutputIntegrity,
}

/// Result of a single validation layer run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerResult {
    pub layer: ValidationLayerId,
    pub passed: bool,
    pub score: f64,
    pub issues: Vec<ValidationIssue>,
    pub duration_ms: u64,
}

/// A validation issue detected by a layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub code: String,
    pub message: String,
    pub source_url: Option<String>,
}

/// Severity of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

/// Which layers to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ValidationMode {
    /// All 6 layers + RAR.
    Strict,
    /// V1-V3 + basic V4.
    #[default]
    Standard,
    /// V1 only.
    Fast,
    /// Skip validation.
    Off,
}

impl ValidationMode {
    /// Return the active layers for this mode.
    pub fn active_layers(&self) -> Vec<ValidationLayerId> {
        match self {
            Self::Strict => vec![
                ValidationLayerId::V1Source,
                ValidationLayerId::V2Content,
                ValidationLayerId::V3Freshness,
                ValidationLayerId::V4CrossSource,
                ValidationLayerId::V5ExtractionQuality,
                ValidationLayerId::V6OutputIntegrity,
            ],
            Self::Standard => vec![
                ValidationLayerId::V1Source,
                ValidationLayerId::V2Content,
                ValidationLayerId::V3Freshness,
                ValidationLayerId::V4CrossSource,
            ],
            Self::Fast => vec![ValidationLayerId::V1Source],
            Self::Off => vec![],
        }
    }

    /// Parse from a string (case-insensitive).
    pub fn from_str_loose(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "strict" => Self::Strict,
            "fast" => Self::Fast,
            "off" | "none" => Self::Off,
            _ => Self::Standard,
        }
    }
}

/// A single claim extracted from a source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub id: String,
    pub text: String,
    pub normalized: String,
    pub source_url: String,
    pub source_index: usize,
    pub confidence: f64,
}

/// A contradiction between two claims from different sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    pub claim_a: Claim,
    pub claim_b: Claim,
    pub severity: ContradictionSeverity,
    pub description: String,
}

/// Severity of a contradiction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContradictionSeverity {
    High,
    Medium,
    Low,
}

/// Consensus score for a claim across multiple sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimConsensus {
    pub claim: Claim,
    pub supporting_sources: usize,
    pub contradicting_sources: usize,
    pub total_sources: usize,
    pub consensus_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_mode_active_layers() {
        assert_eq!(ValidationMode::Off.active_layers().len(), 0);
        assert_eq!(ValidationMode::Fast.active_layers().len(), 1);
        assert_eq!(ValidationMode::Standard.active_layers().len(), 4);
        assert_eq!(ValidationMode::Strict.active_layers().len(), 6);
    }

    #[test]
    fn validation_mode_from_str() {
        assert_eq!(ValidationMode::from_str_loose("strict"), ValidationMode::Strict);
        assert_eq!(ValidationMode::from_str_loose("FAST"), ValidationMode::Fast);
        assert_eq!(ValidationMode::from_str_loose("off"), ValidationMode::Off);
        assert_eq!(ValidationMode::from_str_loose("anything"), ValidationMode::Standard);
    }

    #[test]
    fn validation_result_serialization() {
        let result = ValidationResult {
            layers_run: vec![ValidationLayerId::V1Source],
            layer_results: vec![],
            passed: true,
            confidence: 0.9,
            contradictions: vec![],
            consensus: vec![],
            mode: ValidationMode::Fast,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("V1Source"));
    }
}
