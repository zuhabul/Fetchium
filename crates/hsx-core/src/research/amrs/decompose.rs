//! Query decomposition tree for multi-hop research (PRD §8.8 / §10 Mode C).

/// Status of a query node in the decomposition tree.
#[derive(Debug, Clone, PartialEq)]
pub enum QueryStatus {
    Pending,
    InProgress,
    Complete,
    Failed(String),
}

/// A node in the query decomposition tree.
#[derive(Debug, Clone)]
pub struct QueryNode {
    pub query: String,
    pub depth: usize,
    /// Index of parent node in the tree (None for root).
    pub parent: Option<usize>,
    /// Indices of child nodes in the tree.
    pub children: Vec<usize>,
    pub status: QueryStatus,
}

/// Decompose a complex query into a tree of sub-queries.
///
/// # Heuristics
/// - `"A vs B"` → sub-queries for A and B individually
/// - `"compare A and B"` → sub-queries for A and B
/// - `"implications of X"` → benefits, drawbacks, current state
/// - Default → single root node with the original query
///
/// `max_depth` caps how deep the decomposition tree goes.
pub fn decompose_query(query: &str, max_depth: usize) -> Vec<QueryNode> {
    let mut nodes = Vec::new();

    // Root node
    nodes.push(QueryNode {
        query: query.to_string(),
        depth: 0,
        parent: None,
        children: Vec::new(),
        status: QueryStatus::Pending,
    });

    if max_depth == 0 {
        return nodes;
    }

    let lower = query.to_lowercase();

    // Pattern: comparison ("A vs B" or "compare A and B")
    if lower.contains(" vs ") || lower.contains(" versus ") || lower.contains("compare ") {
        let parts = split_comparison(query);
        if parts.len() > 1 {
            for part in parts {
                let child_idx = nodes.len();
                nodes.push(QueryNode {
                    query: part,
                    depth: 1,
                    parent: Some(0),
                    children: Vec::new(),
                    status: QueryStatus::Pending,
                });
                nodes[0].children.push(child_idx);
            }
        }
    }
    // Pattern: multi-faceted ("implications of", "pros and cons")
    else if lower.contains("implications") || lower.contains("pros and cons") || lower.contains("advantages and disadvantages") {
        let aspects = extract_aspects(query);
        for aspect in aspects {
            let child_idx = nodes.len();
            nodes.push(QueryNode {
                query: aspect,
                depth: 1,
                parent: Some(0),
                children: Vec::new(),
                status: QueryStatus::Pending,
            });
            nodes[0].children.push(child_idx);
        }
    }

    nodes
}

fn split_comparison(query: &str) -> Vec<String> {
    let lower = query.to_lowercase();

    if lower.contains(" vs ") {
        return query.split(" vs ").map(|s| s.trim().to_string()).collect();
    }
    if lower.contains(" versus ") {
        return query.split(" versus ").map(|s| s.trim().to_string()).collect();
    }
    // "compare X and Y" → ["X", "Y"]
    if lower.starts_with("compare ") || lower.starts_with("comparing ") {
        let stripped = lower
            .trim_start_matches("comparing ")
            .trim_start_matches("compare ");
        return stripped
            .split(" and ")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    vec![query.to_string()]
}

fn extract_aspects(query: &str) -> Vec<String> {
    let base = query
        .to_lowercase()
        .replace("implications of ", "")
        .replace("pros and cons of ", "")
        .replace("advantages and disadvantages of ", "");
    let base = base.trim();
    vec![
        format!("benefits of {base}"),
        format!("drawbacks of {base}"),
        format!("current state of {base}"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_query_single_root() {
        let nodes = decompose_query("what is Rust", 3);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].query, "what is Rust");
        assert!(nodes[0].children.is_empty());
    }

    #[test]
    fn vs_query_decomposes_to_two_children() {
        let nodes = decompose_query("Rust vs Go performance", 3);
        assert!(nodes.len() >= 2);
        assert!(!nodes[0].children.is_empty());
    }

    #[test]
    fn implications_decomposes_to_aspects() {
        let nodes = decompose_query("implications of AI in healthcare", 3);
        // Should have root + aspects
        assert!(nodes.len() > 1);
    }

    #[test]
    fn max_depth_zero_returns_root_only() {
        let nodes = decompose_query("Rust vs Go", 0);
        assert_eq!(nodes.len(), 1);
    }
}
