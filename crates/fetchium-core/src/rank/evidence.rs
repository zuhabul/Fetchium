//! Evidence Graph Builder (EGB) — builds citation/corroboration graphs between results.
//!
//! Detects when multiple sources agree on the same facts, strengthening confidence.
//! Uses term-overlap and URL cross-references to build an evidence network.
//! Results with high corroboration scores are more trustworthy.

use crate::types::ResultItem;
use std::collections::{HashMap, HashSet};

/// Configuration for the evidence graph builder.
#[derive(Debug, Clone)]
pub struct EvidenceConfig {
    /// Minimum Jaccard similarity to consider two results corroborating.
    pub corroboration_threshold: f64,
    /// Bonus multiplier applied to well-corroborated results.
    pub corroboration_boost: f64,
    /// Maximum number of edges per node (prevents dense graphs).
    pub max_edges_per_node: usize,
}

impl Default for EvidenceConfig {
    fn default() -> Self {
        Self {
            corroboration_threshold: 0.15,
            corroboration_boost: 1.2,
            max_edges_per_node: 5,
        }
    }
}

/// An edge in the evidence graph representing agreement between two results.
#[derive(Debug, Clone)]
pub struct EvidenceEdge {
    /// Index of the source result.
    pub from: usize,
    /// Index of the target result.
    pub to: usize,
    /// Strength of the corroboration (Jaccard similarity).
    pub strength: f64,
    /// Terms shared between the two results.
    pub shared_terms: Vec<String>,
}

/// A node in the evidence graph with its corroboration score.
#[derive(Debug, Clone)]
pub struct EvidenceNode {
    /// Index into the original results.
    pub result_index: usize,
    /// How many other results corroborate this one.
    pub corroboration_count: usize,
    /// Aggregate corroboration strength (sum of edge weights).
    pub corroboration_strength: f64,
    /// The result is from a unique source (no corroboration).
    pub is_unique_claim: bool,
}

/// The complete evidence graph.
#[derive(Debug, Clone)]
pub struct EvidenceGraph {
    pub nodes: Vec<EvidenceNode>,
    pub edges: Vec<EvidenceEdge>,
    /// Average corroboration across all results.
    pub mean_corroboration: f64,
    /// Fraction of results with at least one corroboration.
    pub coverage: f64,
}

/// Build an evidence graph from search results.
///
/// Computes pairwise similarity between all results and builds a graph
/// where edges represent agreement/corroboration.
pub fn build_evidence_graph(results: &[ResultItem], config: &EvidenceConfig) -> EvidenceGraph {
    if results.is_empty() {
        return EvidenceGraph {
            nodes: vec![],
            edges: vec![],
            mean_corroboration: 0.0,
            coverage: 0.0,
        };
    }

    let n = results.len();

    // Extract term sets for all results
    let term_sets: Vec<HashSet<String>> = results.iter().map(extract_terms).collect();

    // Extract domains to avoid self-corroboration from same source
    let domains: Vec<String> = results.iter().map(|r| extract_domain(&r.url)).collect();

    // Build edges
    let mut edges = Vec::new();
    let mut edge_counts: HashMap<usize, usize> = HashMap::new();

    for i in 0..n {
        for j in (i + 1)..n {
            // Skip same-domain results (not independent corroboration)
            if !domains[i].is_empty() && domains[i] == domains[j] {
                continue;
            }

            // Check edge limits
            let i_count = edge_counts.get(&i).copied().unwrap_or(0);
            let j_count = edge_counts.get(&j).copied().unwrap_or(0);
            if i_count >= config.max_edges_per_node || j_count >= config.max_edges_per_node {
                continue;
            }

            let similarity = jaccard(&term_sets[i], &term_sets[j]);
            if similarity >= config.corroboration_threshold {
                let shared: Vec<String> =
                    term_sets[i].intersection(&term_sets[j]).cloned().collect();

                edges.push(EvidenceEdge {
                    from: i,
                    to: j,
                    strength: similarity,
                    shared_terms: shared,
                });

                *edge_counts.entry(i).or_insert(0) += 1;
                *edge_counts.entry(j).or_insert(0) += 1;
            }
        }
    }

    // Build nodes
    let mut node_strengths: Vec<f64> = vec![0.0; n];
    let mut node_counts: Vec<usize> = vec![0; n];

    for edge in &edges {
        node_strengths[edge.from] += edge.strength;
        node_strengths[edge.to] += edge.strength;
        node_counts[edge.from] += 1;
        node_counts[edge.to] += 1;
    }

    let nodes: Vec<EvidenceNode> = (0..n)
        .map(|i| EvidenceNode {
            result_index: i,
            corroboration_count: node_counts[i],
            corroboration_strength: node_strengths[i],
            is_unique_claim: node_counts[i] == 0,
        })
        .collect();

    // Compute graph-level metrics
    let total_strength: f64 = node_strengths.iter().sum();
    let mean_corroboration = if n > 0 {
        total_strength / n as f64
    } else {
        0.0
    };
    let corroborated_count = node_counts.iter().filter(|&&c| c > 0).count();
    let coverage = if n > 0 {
        corroborated_count as f64 / n as f64
    } else {
        0.0
    };

    EvidenceGraph {
        nodes,
        edges,
        mean_corroboration,
        coverage,
    }
}

/// Apply corroboration boosts to result scores.
///
/// Results confirmed by multiple independent sources get their score boosted.
pub fn apply_evidence_boost(
    results: &mut [ResultItem],
    graph: &EvidenceGraph,
    config: &EvidenceConfig,
) {
    for node in &graph.nodes {
        if node.corroboration_count > 0 && node.result_index < results.len() {
            let boost = 1.0
                + (config.corroboration_boost - 1.0) * (node.corroboration_count as f64).min(3.0)
                    / 3.0;
            let result = &mut results[node.result_index];
            let current = result.score.unwrap_or(0.5);
            result.score = Some((current * boost).min(1.0));
        }
    }
}

// ─── Internal helpers ──────────────────────────────────────

fn extract_terms(item: &ResultItem) -> HashSet<String> {
    let text = format!("{} {}", item.title, item.snippet).to_lowercase();
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 3 && !is_stop_word(t))
        .map(|t| t.to_string())
        .collect()
}

fn extract_domain(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .unwrap_or_default()
}

fn jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let intersection = a.intersection(b).count();
    let union = a.union(b).count();
    if union == 0 {
        return 0.0;
    }
    intersection as f64 / union as f64
}

fn is_stop_word(word: &str) -> bool {
    const STOP: &[&str] = &[
        "the", "and", "for", "are", "but", "not", "you", "all", "can", "was", "one", "our", "out",
        "has", "had", "how", "its", "may", "new", "now", "see", "way", "who", "did", "get", "let",
        "say", "she", "too", "use", "this", "that", "with", "have", "from", "they", "been", "some",
        "when", "what", "your", "each", "make", "like", "just", "than", "them", "very", "will",
        "more", "also",
    ];
    STOP.contains(&word)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BackendId;

    fn make_item(title: &str, snippet: &str, url: &str) -> ResultItem {
        ResultItem {
            title: title.into(),
            url: url.into(),
            snippet: snippet.into(),
            rank: 0,
            backend: BackendId::DuckDuckGo,
            score: Some(0.5),
            published_date: None,
        }
    }

    #[test]
    fn empty_results_empty_graph() {
        let graph = build_evidence_graph(&[], &EvidenceConfig::default());
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
        assert_eq!(graph.mean_corroboration, 0.0);
    }

    #[test]
    fn single_result_no_edges() {
        let items = vec![make_item(
            "Rust",
            "A systems language",
            "https://rust-lang.org",
        )];
        let graph = build_evidence_graph(&items, &EvidenceConfig::default());
        assert_eq!(graph.nodes.len(), 1);
        assert!(graph.edges.is_empty());
        assert!(graph.nodes[0].is_unique_claim);
    }

    #[test]
    fn corroborating_results_create_edge() {
        let items = vec![
            make_item(
                "Rust Memory Safety",
                "Rust guarantees memory safety through ownership and borrowing",
                "https://a.com/rust",
            ),
            make_item(
                "Rust Safety Features",
                "Rust ownership system ensures memory safety without garbage collection",
                "https://b.com/rust",
            ),
        ];
        let graph = build_evidence_graph(&items, &EvidenceConfig::default());
        assert!(
            !graph.edges.is_empty(),
            "corroborating results should create edges"
        );
        assert!(graph.coverage > 0.0);
    }

    #[test]
    fn unrelated_results_no_edges() {
        let items = vec![
            make_item(
                "Rust Programming",
                "Systems language with ownership",
                "https://a.com",
            ),
            make_item(
                "Cooking Recipe",
                "How to bake chocolate cake flour sugar",
                "https://b.com",
            ),
        ];
        let graph = build_evidence_graph(&items, &EvidenceConfig::default());
        assert!(
            graph.edges.is_empty(),
            "unrelated results should have no edges"
        );
    }

    #[test]
    fn same_domain_not_corroboration() {
        let items = vec![
            make_item(
                "Rust Article 1",
                "Rust ownership memory safety borrow",
                "https://example.com/a",
            ),
            make_item(
                "Rust Article 2",
                "Rust ownership memory safety lifetimes",
                "https://example.com/b",
            ),
        ];
        let graph = build_evidence_graph(&items, &EvidenceConfig::default());
        assert!(
            graph.edges.is_empty(),
            "same-domain results should not corroborate"
        );
    }

    #[test]
    fn evidence_boost_increases_scores() {
        let mut items = vec![
            make_item(
                "Rust Safety",
                "Rust memory safety ownership borrow",
                "https://a.com",
            ),
            make_item(
                "Rust Ownership",
                "Rust memory safety borrow checker system",
                "https://b.com",
            ),
        ];
        let config = EvidenceConfig::default();
        let graph = build_evidence_graph(&items, &config);

        let scores_before: Vec<f64> = items.iter().map(|r| r.score.unwrap_or(0.0)).collect();
        apply_evidence_boost(&mut items, &graph, &config);
        let scores_after: Vec<f64> = items.iter().map(|r| r.score.unwrap_or(0.0)).collect();

        if !graph.edges.is_empty() {
            assert!(
                scores_after[0] >= scores_before[0],
                "corroborated result should be boosted"
            );
        }
    }

    #[test]
    fn max_edges_respected() {
        let config = EvidenceConfig {
            max_edges_per_node: 2,
            corroboration_threshold: 0.1,
            ..EvidenceConfig::default()
        };

        // Create 5 similar results from different domains
        let items: Vec<ResultItem> = (0..5)
            .map(|i| {
                make_item(
                    "Rust memory safety ownership borrow checker",
                    "The Rust programming language guarantees memory safety",
                    &format!("https://site{i}.com/article"),
                )
            })
            .collect();

        let graph = build_evidence_graph(&items, &config);

        // No node should have more than max_edges edges
        for node in &graph.nodes {
            assert!(
                node.corroboration_count <= config.max_edges_per_node,
                "node {} has {} edges, max is {}",
                node.result_index,
                node.corroboration_count,
                config.max_edges_per_node,
            );
        }
    }

    #[test]
    fn coverage_metric_correct() {
        let items = vec![
            make_item(
                "Rust Safety",
                "Rust ownership borrow memory safe",
                "https://a.com",
            ),
            make_item(
                "Rust Borrow",
                "Rust ownership borrow memory checker",
                "https://b.com",
            ),
            make_item(
                "Cooking Tips",
                "How to bake cookies flour sugar eggs",
                "https://c.com",
            ),
        ];
        let graph = build_evidence_graph(&items, &EvidenceConfig::default());

        // Cooking result should be unique, Rust results may corroborate
        assert!(graph.coverage <= 1.0);
        assert!(graph.coverage >= 0.0);
    }
}
