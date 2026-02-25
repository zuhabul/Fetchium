//! Research pipeline types and module root (PRD §10).

pub mod amrs;
pub mod decompose;
pub mod pipeline;
pub mod srp;
pub mod srp_types;

pub use pipeline::ResearchPipeline;
pub use srp::run_srp_pipeline;
pub use srp_types::{SrpChunk, SrpConfig, SrpEvent};

use crate::citation::evidence_graph::EvidenceGraph;
use crate::citation::types::{CitationStyle, FormattedCitation, SourceMeta};
use crate::validate::types::{ValidationMode, ValidationResult};
use serde::{Deserialize, Serialize};

/// Configuration for a research pipeline run.
#[derive(Debug, Clone)]
pub struct ResearchConfig {
    pub query: String,
    pub max_sources: usize,
    pub token_budget: Option<usize>,
    pub citation_style: CitationStyle,
    pub validation_mode: ValidationMode,
    pub strict_evidence: bool,
    pub evidence_graph: bool,
    pub trace_sources: bool,
    pub trust_verify: bool,
    pub max_rar_loops: usize,
}

impl Default for ResearchConfig {
    fn default() -> Self {
        Self {
            query: String::new(),
            max_sources: 10,
            token_budget: None,
            citation_style: CitationStyle::Inline,
            validation_mode: ValidationMode::Standard,
            strict_evidence: false,
            evidence_graph: false,
            trace_sources: false,
            trust_verify: false,
            max_rar_loops: 3,
        }
    }
}

/// The output of a complete research pipeline run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchReport {
    pub query: String,
    pub sub_queries: Vec<String>,
    pub synthesis: String,
    pub sources: Vec<SourceMeta>,
    pub citations: Vec<FormattedCitation>,
    pub reference_section: String,
    pub validation: ValidationResult,
    pub evidence_graph: Option<EvidenceGraph>,
    pub rar_iterations: Vec<crate::validate::rar::RarIterationResult>,
    pub meta: ResearchMeta,
}

/// Metadata about a research pipeline run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchMeta {
    pub duration_ms: u64,
    pub sources_fetched: usize,
    pub sources_validated: usize,
    pub validation_pass_rate: f64,
    pub overall_confidence: f64,
    pub rar_loops_executed: usize,
}
