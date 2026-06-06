//! Domain-specific intelligence modes — academic, code, legal, financial,
//! medical, security (PRD §38).

pub mod academic;
pub mod code;
pub mod financial;
pub mod legal;
pub mod medical;
pub mod security;

use crate::error::FetchiumError;

/// A pre-configured intelligence mode for a specific research domain.
#[derive(Debug, Clone)]
pub struct DomainMode {
    pub name: String,
    /// Preferred backend order (first = highest priority).
    pub backends_priority: Vec<String>,
    pub ranking_overrides: RankingOverrides,
    pub special_features: Vec<SpecialFeature>,
    pub default_citation_style: String,
}

/// Per-signal weight multipliers applied on top of HyperFusion defaults.
#[derive(Debug, Clone)]
pub struct RankingOverrides {
    pub authority_multiplier: f64,
    pub temporal_multiplier: f64,
    pub evidence_multiplier: f64,
    pub consensus_multiplier: f64,
}

impl Default for RankingOverrides {
    fn default() -> Self {
        Self {
            authority_multiplier: 1.0,
            temporal_multiplier: 1.0,
            evidence_multiplier: 1.0,
            consensus_multiplier: 1.0,
        }
    }
}

/// Optional domain-specific output feature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecialFeature {
    CitationGraph,
    BibTexExport,
    DependencyAnalysis,
    LicenseCheck,
    PrecedentMapping,
    TrendAnalysis,
    EvidenceGrading,
    CvssScoring,
}

/// Retrieve a domain mode by name (case-insensitive).
pub fn get_mode(name: &str) -> Result<DomainMode, FetchiumError> {
    match name.to_lowercase().trim() {
        "academic" | "research" => Ok(academic::mode()),
        "code" | "dev" | "programming" => Ok(code::mode()),
        "legal" | "law" => Ok(legal::mode()),
        "financial" | "finance" | "economics" => Ok(financial::mode()),
        "medical" | "medicine" | "health" | "science" => Ok(medical::mode()),
        "security" | "cybersecurity" | "infosec" => Ok(security::mode()),
        other => Err(FetchiumError::Config(format!(
            "Unknown domain mode '{}'. Valid: academic, code, legal, financial, medical, security",
            other
        ))),
    }
}

/// All available domain mode names.
pub fn available_modes() -> &'static [&'static str] {
    &[
        "academic",
        "code",
        "legal",
        "financial",
        "medical",
        "security",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_mode_academic() {
        let m = get_mode("academic").unwrap();
        assert_eq!(m.name, "academic");
        assert!(m.ranking_overrides.temporal_multiplier < 1.0);
    }

    #[test]
    fn get_mode_security() {
        let m = get_mode("security").unwrap();
        assert!(m.ranking_overrides.temporal_multiplier > 1.0);
    }

    #[test]
    fn get_mode_unknown_errors() {
        assert!(get_mode("unknown").is_err());
    }
}
