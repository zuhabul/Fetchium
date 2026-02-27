//! Code domain mode configuration.

use super::{DomainMode, RankingOverrides, SpecialFeature};

pub fn mode() -> DomainMode {
    DomainMode {
        name: "code".into(),
        backends_priority: vec![
            "github".into(),
            "stackoverflow".into(),
            "duckduckgo".into(),
            "docs_rs".into(),
        ],
        ranking_overrides: RankingOverrides {
            authority_multiplier: 1.2,
            temporal_multiplier: 1.5,
            evidence_multiplier: 1.0,
            consensus_multiplier: 1.0,
        },
        special_features: vec![
            SpecialFeature::DependencyAnalysis,
            SpecialFeature::LicenseCheck,
        ],
        default_citation_style: "inline".into(),
    }
}
