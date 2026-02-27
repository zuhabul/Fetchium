//! Citation system (6 styles) + Evidence Graph Protocol (PRD §24).

pub mod evidence_graph;
pub mod evidence_tracker;
pub mod formatter;
pub mod types;

pub use evidence_graph::{EvidenceGraph, EvidenceGraphBuilder};
pub use evidence_tracker::EvidenceTracker;
pub use formatter::CitationFormatter;
pub use types::*;
