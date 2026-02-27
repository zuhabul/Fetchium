//! Evidence Graph Protocol (EGP) — graph-based claim provenance (PRD §8.7, §24).

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// The complete evidence graph for a research query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceGraph {
    pub nodes: Vec<EvidenceNode>,
    pub edges: Vec<EvidenceEdge>,
    pub root_claim: String,
    pub overall_confidence: f64,
    pub content_hashes: HashMap<String, String>,
}

/// A node in the evidence graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: NodeType,
    pub content: String,
    pub confidence: f64,
    pub timestamp: String,
    pub source_url: Option<String>,
}

/// Node type in the evidence graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    Claim,
    Source,
    Fact,
    Inference,
}

/// An edge linking nodes in the evidence graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceEdge {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub edge_type: EdgeType,
    pub quote: String,
    pub quote_hash: String,
}

/// Type of evidence relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    Supports,
    Contradicts,
    PartiallySupports,
    InferredFrom,
}

/// Incrementally builds an evidence graph.
pub struct EvidenceGraphBuilder {
    nodes: Vec<EvidenceNode>,
    edges: Vec<EvidenceEdge>,
    content_hashes: HashMap<String, String>,
    root_claim: String,
    node_counter: usize,
}

impl EvidenceGraphBuilder {
    pub fn new(root_claim: &str) -> Self {
        let mut builder = Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            content_hashes: HashMap::new(),
            root_claim: root_claim.to_string(),
            node_counter: 0,
        };
        // n1 is always the root claim node
        builder.add_node(NodeType::Claim, root_claim, 0.0, None);
        builder
    }

    /// Add a node and return its ID.
    pub fn add_node(
        &mut self,
        node_type: NodeType,
        content: &str,
        confidence: f64,
        source_url: Option<&str>,
    ) -> String {
        self.node_counter += 1;
        let id = format!("n{}", self.node_counter);
        self.nodes.push(EvidenceNode {
            id: id.clone(),
            node_type,
            content: content.to_string(),
            confidence,
            timestamp: Utc::now().to_rfc3339(),
            source_url: source_url.map(|s| s.to_string()),
        });
        id
    }

    /// Add a source node and register its content hash.
    pub fn add_source(&mut self, url: &str, title: &str, content: &str, confidence: f64) -> String {
        let hash = Self::sha256_hex(content);
        self.content_hashes.insert(url.to_string(), hash);
        self.add_node(NodeType::Source, title, confidence, Some(url))
    }

    /// Add an evidence edge with a supporting quote.
    pub fn add_edge(&mut self, from: &str, to: &str, edge_type: EdgeType, quote: &str) {
        let quote_hash = Self::sha256_hex(quote);
        self.edges.push(EvidenceEdge {
            from: from.to_string(),
            to: to.to_string(),
            edge_type,
            quote: quote.to_string(),
            quote_hash,
        });
    }

    /// Add a fact extracted from a source and link it to that source.
    pub fn add_fact_from_source(
        &mut self,
        source_id: &str,
        fact_text: &str,
        supporting_quote: &str,
        confidence: f64,
    ) -> String {
        let fact_id = self.add_node(NodeType::Fact, fact_text, confidence, None);
        self.add_edge(source_id, &fact_id, EdgeType::Supports, supporting_quote);
        fact_id
    }

    /// Link a fact to the root claim (n1).
    pub fn link_to_root(&mut self, fact_id: &str, edge_type: EdgeType, quote: &str) {
        self.add_edge(fact_id, "n1", edge_type, quote);
    }

    /// Build the final evidence graph with computed overall confidence.
    pub fn build(self) -> EvidenceGraph {
        let confidence = Self::compute_overall_confidence(&self.edges);
        EvidenceGraph {
            nodes: self.nodes,
            edges: self.edges,
            root_claim: self.root_claim,
            overall_confidence: confidence,
            content_hashes: self.content_hashes,
        }
    }

    fn compute_overall_confidence(edges: &[EvidenceEdge]) -> f64 {
        let support_count = edges
            .iter()
            .filter(|e| e.edge_type == EdgeType::Supports)
            .count();
        let contradict_count = edges
            .iter()
            .filter(|e| e.edge_type == EdgeType::Contradicts)
            .count();
        let total = (support_count + contradict_count).max(1);
        support_count as f64 / total as f64
    }

    fn sha256_hex(input: &str) -> String {
        let mut h = Sha256::new();
        h.update(input.as_bytes());
        h.finalize().iter().map(|b| format!("{b:02x}")).collect()
    }
}

impl EvidenceGraph {
    /// Verify that a quote's hash matches the stored hash.
    pub fn verify_quote(&self, edge_index: usize) -> bool {
        if let Some(edge) = self.edges.get(edge_index) {
            let computed = {
                let mut h = Sha256::new();
                h.update(edge.quote.as_bytes());
                h.finalize()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            };
            computed == edge.quote_hash
        } else {
            false
        }
    }

    /// Verify a source's content hash against freshly fetched content.
    pub fn verify_source(&self, url: &str, current_content: &str) -> bool {
        if let Some(stored_hash) = self.content_hashes.get(url) {
            let mut h = Sha256::new();
            h.update(current_content.as_bytes());
            let current: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
            &current == stored_hash
        } else {
            false
        }
    }

    /// Return all source->fact->claim paths to the root (n1).
    pub fn trace_root_evidence(&self) -> Vec<Vec<String>> {
        let mut paths = Vec::new();
        for edge in &self.edges {
            if edge.to == "n1" {
                let mut path = vec![edge.from.clone(), "n1".to_string()];
                for inner_edge in &self.edges {
                    if inner_edge.to == edge.from {
                        path.insert(0, inner_edge.from.clone());
                    }
                }
                paths.push(path);
            }
        }
        paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_and_verify_graph() {
        let mut b = EvidenceGraphBuilder::new("Rust is memory-safe");
        let s1 = b.add_source(
            "https://rust-lang.org",
            "Rust Lang",
            "Rust guarantees memory safety without GC",
            0.95,
        );
        let f1 = b.add_fact_from_source(
            &s1,
            "Rust uses ownership for memory safety",
            "Rust guarantees memory safety without GC",
            0.9,
        );
        b.link_to_root(&f1, EdgeType::Supports, "memory safety without GC");
        let graph = b.build();
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2);
        assert!(graph.overall_confidence > 0.9);
        assert!(graph.verify_quote(0));
        assert!(graph.verify_source(
            "https://rust-lang.org",
            "Rust guarantees memory safety without GC"
        ));
        assert!(!graph.verify_source("https://rust-lang.org", "tampered content"));
    }

    #[test]
    fn trace_evidence_chain() {
        let mut b = EvidenceGraphBuilder::new("claim");
        let s = b.add_source("https://a.com", "A", "content", 0.8);
        let f = b.add_fact_from_source(&s, "fact", "content", 0.8);
        b.link_to_root(&f, EdgeType::Supports, "evidence");
        let graph = b.build();
        let paths = graph.trace_root_evidence();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].len(), 3);
    }

    #[test]
    fn serialization_roundtrip() {
        let mut b = EvidenceGraphBuilder::new("test claim");
        b.add_source("https://x.com", "X", "data", 0.7);
        let graph = b.build();
        let json = serde_json::to_string(&graph).unwrap();
        let parsed: EvidenceGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.root_claim, "test claim");
    }
}
