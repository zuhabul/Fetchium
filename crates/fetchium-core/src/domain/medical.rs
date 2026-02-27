//! Medical domain mode configuration.

use super::{DomainMode, RankingOverrides, SpecialFeature};

pub fn mode() -> DomainMode {
    DomainMode {
        name: "medical".into(),
        backends_priority: vec![
            "pubmed".into(),
            "scholar".into(),
            "nih".into(),
            "duckduckgo".into(),
        ],
        ranking_overrides: RankingOverrides {
            authority_multiplier: 2.0,
            temporal_multiplier: 1.0,
            evidence_multiplier: 2.0,
            consensus_multiplier: 2.0,
        },
        special_features: vec![
            SpecialFeature::EvidenceGrading,
            SpecialFeature::CitationGraph,
        ],
        default_citation_style: "apa".into(),
    }
}
