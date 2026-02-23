//! Academic domain mode configuration.

use super::{DomainMode, RankingOverrides, SpecialFeature};

pub fn mode() -> DomainMode {
    DomainMode {
        name: "academic".into(),
        backends_priority: vec![
            "arxiv".into(),
            "scholar".into(),
            "pubmed".into(),
            "semantic_scholar".into(),
            "duckduckgo".into(),
        ],
        ranking_overrides: RankingOverrides {
            authority_multiplier: 2.0,
            temporal_multiplier: 0.5,
            evidence_multiplier: 1.5,
            consensus_multiplier: 1.2,
        },
        special_features: vec![SpecialFeature::CitationGraph, SpecialFeature::BibTexExport],
        default_citation_style: "apa".into(),
    }
}
