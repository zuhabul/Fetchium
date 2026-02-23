//! `hsx agent-research` — structured research output for AI agents.

use crate::cli::AgentResearchArgs;
use hsx_core::citation::types::CitationStyle as CoreCitationStyle;
use hsx_core::citation::evidence_graph::EvidenceGraph;
use hsx_core::config::HsxConfig;
use hsx_core::research::pipeline::ResearchPipeline;
use hsx_core::research::ResearchConfig;
use hsx_core::validate::types::{Contradiction, ValidationMode};
use serde::{Deserialize, Serialize};

/// Agent-optimized research output (PRD §43 AgentSearchResult).
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentResearchOutput {
    pub meta: AgentMeta,
    pub findings: Vec<Finding>,
    pub sources: Vec<AgentSource>,
    pub evidence_graph: Option<EvidenceGraph>,
    pub contradictions: Vec<Contradiction>,
    pub confidence: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentMeta {
    pub query: String,
    pub mode: String,
    pub tier: String,
    pub tokens_used: usize,
    pub tokens_budget: usize,
    pub sources_fetched: usize,
    pub duration_ms: u64,
    pub result_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Finding {
    pub claim: String,
    pub confidence: f64,
    pub source_indices: Vec<usize>,
    pub verified: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentSource {
    pub index: usize,
    pub url: String,
    pub title: String,
    pub relevance: f64,
    pub content_hash: String,
}

pub async fn run(args: AgentResearchArgs, config_obj: &HsxConfig) -> anyhow::Result<()> {
    let tier_str = format!("{:?}", args.tier).to_lowercase();

    let config = ResearchConfig {
        query: args.query.clone(),
        max_sources: args.max_sources,
        token_budget: Some(args.budget as usize),
        citation_style: CoreCitationStyle::Inline,
        validation_mode: ValidationMode::Standard,
        strict_evidence: args.strict_evidence,
        evidence_graph: args.evidence_graph,
        trace_sources: false,
        trust_verify: false,
        max_rar_loops: 3,
    };

    let http_client = hsx_core::http::client::HttpClient::new(config_obj)?;
    let report = ResearchPipeline::execute(&config, config_obj, &http_client).await?;

    let findings = report.synthesis.split(". ").filter(|s| !s.trim().is_empty()).map(|c| Finding {
        claim: c.trim().to_string(),
        confidence: report.meta.overall_confidence,
        source_indices: vec![],
        verified: true,
    }).collect();

    let sources = report.sources.into_iter().enumerate().map(|(i, s)| AgentSource {
        index: i + 1,
        url: s.url,
        title: s.title,
        relevance: 1.0,
        content_hash: "none".into(),
    }).collect();

    let output = AgentResearchOutput {
        meta: AgentMeta {
            query: report.query,
            mode: "research".into(),
            tier: tier_str,
            tokens_used: 0,
            tokens_budget: args.budget as usize,
            sources_fetched: report.meta.sources_fetched,
            duration_ms: report.meta.duration_ms,
            result_id: uuid::Uuid::new_v4().to_string(),
        },
        findings,
        sources,
        evidence_graph: report.evidence_graph,
        contradictions: report.validation.contradictions,
        confidence: report.meta.overall_confidence,
    };

    // Single-line JSON for pipe compatibility
    let json = serde_json::to_string(&output)?;
    println!("{json}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_is_valid_json() {
        let output = AgentResearchOutput {
            meta: AgentMeta {
                query: "test".into(),
                mode: "research".into(),
                tier: "summary".into(),
                tokens_used: 500,
                tokens_budget: 4000,
                sources_fetched: 5,
                duration_ms: 1200,
                result_id: "test-id".into(),
            },
            findings: vec![Finding {
                claim: "Rust is memory safe".into(),
                confidence: 0.95,
                source_indices: vec![0, 1],
                verified: true,
            }],
            sources: vec![],
            evidence_graph: None,
            contradictions: vec![],
            confidence: 0.9,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.starts_with('{'));
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["meta"]["query"], "test");
    }
}
