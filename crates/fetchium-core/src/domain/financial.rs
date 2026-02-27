//! Financial domain mode configuration.

use super::{DomainMode, RankingOverrides, SpecialFeature};

pub fn mode() -> DomainMode {
    DomainMode {
        name: "financial".into(),
        backends_priority: vec!["duckduckgo".into(), "reddit".into(), "hackernews".into()],
        ranking_overrides: RankingOverrides {
            authority_multiplier: 1.5,
            temporal_multiplier: 2.5,
            evidence_multiplier: 1.2,
            consensus_multiplier: 1.3,
        },
        special_features: vec![SpecialFeature::TrendAnalysis],
        default_citation_style: "apa".into(),
    }
}
