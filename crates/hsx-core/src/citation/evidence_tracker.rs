//! Evidence chain tracking — links citations to EGP nodes, strict evidence mode (PRD §24).

use crate::citation::evidence_graph::{EvidenceGraph, EvidenceNode, NodeType};
use crate::citation::formatter::CitationFormatter;
use crate::citation::types::{CitationStyle, FormattedCitation, SourceMeta};
use serde::{Deserialize, Serialize};

/// Result of evidence chain analysis on a piece of text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceAnalysis {
    pub cited_claims: Vec<CitedClaim>,
    pub unverified_claims: Vec<String>,
    pub annotated_text: String,
    pub strict_mode_passed: bool,
}

/// A claim that was successfully linked to a source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitedClaim {
    pub claim_text: String,
    pub citation: FormattedCitation,
    pub evidence_node_id: Option<String>,
    pub confidence: f64,
}

/// Tracks evidence chains and injects citations into text.
pub struct EvidenceTracker {
    formatter: CitationFormatter,
}

impl EvidenceTracker {
    pub fn new(style: CitationStyle) -> Self {
        Self {
            formatter: CitationFormatter::new(style),
        }
    }

    /// Analyze text against an evidence graph and inject citation markers.
    ///
    /// In strict mode, unverifiable factual claims are marked `[unverified]`.
    pub fn analyze(
        &self,
        text: &str,
        graph: &EvidenceGraph,
        sources: &[SourceMeta],
        strict: bool,
    ) -> EvidenceAnalysis {
        let sentences: Vec<&str> = text
            .split(". ")
            .flat_map(|s| s.split(".\n"))
            .collect();

        let mut cited = Vec::new();
        let mut unverified = Vec::new();
        let mut annotated_parts = Vec::new();

        for sentence in &sentences {
            let trimmed = sentence.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Some((node, source_idx)) =
                self.find_supporting_evidence(trimmed, graph, sources)
            {
                let citation = self.formatter.format(&sources[source_idx], source_idx + 1);
                annotated_parts.push(format!("{trimmed} {}", citation.inline_marker));
                cited.push(CitedClaim {
                    claim_text: trimmed.to_string(),
                    citation,
                    evidence_node_id: Some(node.id.clone()),
                    confidence: node.confidence,
                });
            } else if strict && Self::is_factual_claim(trimmed) {
                annotated_parts.push(format!("{trimmed} [unverified]"));
                unverified.push(trimmed.to_string());
            } else {
                annotated_parts.push(trimmed.to_string());
            }
        }

        let strict_passed = unverified.is_empty();
        EvidenceAnalysis {
            cited_claims: cited,
            unverified_claims: unverified,
            annotated_text: annotated_parts.join(". "),
            strict_mode_passed: strict_passed,
        }
    }

    fn find_supporting_evidence<'a>(
        &self,
        sentence: &str,
        graph: &'a EvidenceGraph,
        sources: &[SourceMeta],
    ) -> Option<(&'a EvidenceNode, usize)> {
        let lower = sentence.to_lowercase();
        for node in &graph.nodes {
            if node.node_type == NodeType::Fact || node.node_type == NodeType::Source {
                let node_lower = node.content.to_lowercase();
                if Self::word_overlap(&lower, &node_lower) > 0.3 {
                    let source_idx = node
                        .source_url
                        .as_ref()
                        .and_then(|url| sources.iter().position(|s| &s.url == url))
                        .unwrap_or(0);
                    return Some((node, source_idx));
                }
            }
        }
        None
    }

    fn word_overlap(a: &str, b: &str) -> f64 {
        let wa: std::collections::HashSet<&str> = a.split_whitespace().collect();
        let wb: std::collections::HashSet<&str> = b.split_whitespace().collect();
        let inter = wa.intersection(&wb).count();
        let union = wa.union(&wb).count();
        if union == 0 { 0.0 } else { inter as f64 / union as f64 }
    }

    fn is_factual_claim(text: &str) -> bool {
        if text.len() < 20 { return false; }
        if text.ends_with('?') { return false; }
        let lower = text.to_lowercase();
        let transitions = ["however", "therefore", "in conclusion", "overall",
            "additionally", "furthermore", "in summary"];
        !transitions.iter().any(|t| lower.starts_with(t))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::citation::evidence_graph::EvidenceGraphBuilder;

    #[test]
    fn strict_mode_flags_unverified() {
        let mut b = EvidenceGraphBuilder::new("root");
        b.add_source("https://a.com", "A", "Rust is memory safe", 0.9);
        let graph = b.build();
        let sources = vec![SourceMeta {
            url: "https://a.com".into(),
            title: "A".into(),
            author: None,
            publisher: None,
            published_date: None,
            accessed_date: "2026-02-23".into(),
        }];
        let tracker = EvidenceTracker::new(CitationStyle::Inline);
        let analysis = tracker.analyze(
            "Rust is memory safe. Python is dynamically typed.",
            &graph,
            &sources,
            true,
        );
        assert!(!analysis.strict_mode_passed);
        assert!(analysis.annotated_text.contains("[unverified]"));
    }

    #[test]
    fn cited_claim_gets_marker() {
        let mut b = EvidenceGraphBuilder::new("root");
        let s = b.add_source("https://rust-lang.org", "Rust Lang", "Rust is fast and safe", 0.9);
        b.add_fact_from_source(&s, "Rust is fast and safe", "Rust is fast and safe", 0.9);
        let graph = b.build();
        let sources = vec![SourceMeta {
            url: "https://rust-lang.org".into(),
            title: "Rust Lang".into(),
            author: None,
            publisher: None,
            published_date: None,
            accessed_date: "2026-02-23".into(),
        }];
        let tracker = EvidenceTracker::new(CitationStyle::Inline);
        let analysis = tracker.analyze("Rust is fast and safe", &graph, &sources, false);
        assert!(!analysis.cited_claims.is_empty());
    }
}
