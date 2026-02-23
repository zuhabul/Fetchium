//! Channel message types for inter-agent communication (PRD §8.8).

use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};

/// Agent type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    Search,
    Extract,
    Verify,
    Synthesize,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::Search    => write!(f, "Search"),
            AgentType::Extract   => write!(f, "Extract"),
            AgentType::Verify    => write!(f, "Verify"),
            AgentType::Synthesize => write!(f, "Synthesize"),
        }
    }
}

/// A contradiction detected between two sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmrsContradiction {
    pub claim: String,
    pub source_a: String,
    pub source_b: String,
    pub source_a_says: String,
    pub source_b_says: String,
    /// Severity 0.0–1.0
    pub severity: f64,
}

/// An audit trail entry recording an agent action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub agent: AgentType,
    pub action: String,
    pub detail: String,
}

/// A verified finding from the research session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmrsFinding {
    pub claim: String,
    pub confidence: f64,
    pub source_indices: Vec<usize>,
    pub evidence_type: String,
}

/// A source document used in the research session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmrsSource {
    pub url: String,
    pub title: String,
    pub content: String,
    pub content_hash: String,
}

/// Messages flowing between agents through the coordinator.
#[derive(Debug)]
pub enum AgentMessage {
    // Agent → Coordinator
    SearchComplete {
        sub_query: String,
        results: Vec<crate::types::ResultItem>,
        follow_up_queries: Vec<String>,
    },
    ExtractComplete {
        sources: Vec<AmrsSource>,
    },
    VerifyComplete {
        findings: Vec<AmrsFinding>,
        contradictions: Vec<AmrsContradiction>,
    },
    SynthesisComplete {
        report: String,
        audit_entries: Vec<AuditEntry>,
    },

    // Coordinator → Agent
    SpawnSearch {
        query: String,
        depth: usize,
    },
    SpawnExtract {
        urls: Vec<String>,
        query: String,
    },
    SpawnVerify {
        sources: Vec<AmrsSource>,
        query: String,
    },
    SpawnSynthesize {
        findings: Vec<AmrsFinding>,
        sources: Vec<AmrsSource>,
        query: String,
    },

    // Control
    Shutdown,
    ProgressUpdate {
        agent_type: AgentType,
        message: String,
        progress: f64,
    },
}

pub type AgentSender   = mpsc::Sender<AgentMessage>;
pub type AgentReceiver = mpsc::Receiver<AgentMessage>;
