//! Speculative Research Pipelining types (PRD §8.5).
//!
//! SRP streams partial findings to the caller as they become available,
//! emitting updates and corrections as more sources are processed.

use serde::{Deserialize, Serialize};

/// A single chunk emitted by the SRP pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SrpChunk {
    /// Event type
    pub event: SrpEvent,
    /// Human-readable content for this chunk
    pub content: String,
    /// Indices of sources supporting this chunk
    pub sources: Vec<usize>,
    /// Confidence score 0.0–1.0
    pub confidence: f64,
    /// Milliseconds since pipeline start
    pub timestamp_ms: u64,
}

/// The type of SRP event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SrpEvent {
    /// First findings from initial sources
    Initial,
    /// Additional data confirms or extends earlier findings
    Update,
    /// New data contradicts earlier findings — correction
    Correction,
    /// All sources processed — final validated output
    Final,
}

/// Configuration for the SRP pipeline.
#[derive(Debug, Clone)]
pub struct SrpConfig {
    /// Minimum number of sources before emitting `Initial`
    pub min_initial_sources: usize,
    /// Confidence delta below which a correction is triggered
    pub correction_threshold: f64,
    /// Maximum time to wait for all sources (milliseconds)
    pub max_wait_ms: u64,
}

impl Default for SrpConfig {
    fn default() -> Self {
        Self {
            min_initial_sources: 2,
            correction_threshold: 0.3,
            max_wait_ms: 30_000,
        }
    }
}
