//! Security domain mode configuration.

use super::{DomainMode, RankingOverrides, SpecialFeature};

pub fn mode() -> DomainMode {
    DomainMode {
        name: "security".into(),
        backends_priority: vec![
            "duckduckgo".into(),
            "github".into(),
            "stackoverflow".into(),
        ],
        ranking_overrides: RankingOverrides {
            authority_multiplier: 1.5,
            temporal_multiplier: 3.0,
            evidence_multiplier: 1.0,
            consensus_multiplier: 0.8,
        },
        special_features: vec![SpecialFeature::CvssScoring],
        default_citation_style: "inline".into(),
    }
}
