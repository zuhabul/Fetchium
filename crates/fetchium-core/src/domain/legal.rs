//! Legal domain mode configuration.

use super::{DomainMode, RankingOverrides, SpecialFeature};

pub fn mode() -> DomainMode {
    DomainMode {
        name: "legal".into(),
        backends_priority: vec!["duckduckgo".into(), "scholar".into()],
        ranking_overrides: RankingOverrides {
            authority_multiplier: 2.5,
            temporal_multiplier: 0.3,
            evidence_multiplier: 1.8,
            consensus_multiplier: 1.5,
        },
        special_features: vec![SpecialFeature::PrecedentMapping],
        default_citation_style: "chicago".into(),
    }
}
