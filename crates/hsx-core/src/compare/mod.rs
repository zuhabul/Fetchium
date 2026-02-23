//! Comparison research engine — "A vs B vs C" (Phase 5, PRD §10, Mode F).
//!
//! Researches each item in parallel, extracts comparison dimensions,
//! and generates a structured comparison table.

pub mod parser;
pub use parser::{parse_comparison_query, ComparisonQuery};

use serde::{Deserialize, Serialize};

/// Configuration for a comparison run.
#[derive(Debug, Clone)]
pub struct CompareConfig {
    /// Maximum sources per item.
    pub max_sources_per_item: usize,
    /// Token budget for each item's research.
    pub token_budget: usize,
    /// Output format: "markdown", "json".
    pub format: String,
}

impl Default for CompareConfig {
    fn default() -> Self {
        Self {
            max_sources_per_item: 5,
            token_budget: 2000,
            format: "markdown".into(),
        }
    }
}

/// A single cell in the comparison table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonCell {
    /// The item being described (e.g., "React").
    pub item: String,
    /// The dimension (e.g., "Performance").
    pub dimension: String,
    /// The value/description for this cell.
    pub value: String,
    /// Source URL(s) supporting this cell.
    pub sources: Vec<String>,
}

/// A complete comparison result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    /// Original parsed query.
    pub raw_query: String,
    /// Items compared.
    pub items: Vec<String>,
    /// Dimensions extracted (e.g., "Performance", "Learning Curve").
    pub dimensions: Vec<String>,
    /// Flat list of comparison cells (items × dimensions).
    pub cells: Vec<ComparisonCell>,
    /// Pre-rendered markdown table.
    pub markdown_table: String,
}

/// Standard comparison dimensions extracted from research.
const STANDARD_DIMENSIONS: &[&str] = &[
    "Performance",
    "Learning Curve",
    "Ecosystem",
    "Community",
    "Production Readiness",
    "License",
    "Key Use Case",
];

/// Build a comparison result from per-item research snippets.
///
/// `item_data` maps item name → (summary, source URLs).
pub fn build_comparison(
    query: &ComparisonQuery,
    item_data: &[(String, String, Vec<String>)], // (item, summary, sources)
) -> ComparisonResult {
    let items: Vec<String> = query.items.clone();
    let dimensions: Vec<String> = STANDARD_DIMENSIONS
        .iter()
        .map(|s| s.to_string())
        .collect();

    let mut cells = Vec::new();

    for (item, summary, sources) in item_data {
        // Extract a value for each dimension from the summary via keyword search
        for dim in &dimensions {
            let value = extract_dimension_value(summary, dim);
            cells.push(ComparisonCell {
                item: item.clone(),
                dimension: dim.clone(),
                value,
                sources: sources.clone(),
            });
        }
    }

    let markdown_table = render_markdown_table(&items, &dimensions, &cells);

    ComparisonResult {
        raw_query: query.raw_query.clone(),
        items,
        dimensions,
        cells,
        markdown_table,
    }
}

/// Extract a value for a dimension from a text summary.
fn extract_dimension_value(summary: &str, dimension: &str) -> String {
    let dim_lower = dimension.to_lowercase();
    let words: Vec<&str> = summary.split_whitespace().collect();

    // Find sentences containing keywords related to the dimension
    let keywords: &[&str] = match dim_lower.as_str() {
        "performance" => &["fast", "slow", "speed", "benchmark", "performance", "throughput"],
        "learning curve" => &["easy", "difficult", "complex", "beginner", "learn", "learning"],
        "ecosystem" => &["libraries", "packages", "npm", "crate", "ecosystem", "plugins"],
        "community" => &["community", "users", "popular", "adoption", "developers"],
        "production readiness" => &["stable", "production", "enterprise", "mature", "ready"],
        "license" => &["mit", "apache", "gpl", "bsd", "license", "open source"],
        "key use case" => &["use", "application", "web", "server", "mobile", "cli"],
        _ => &[],
    };

    // Find best matching sentence
    let sentences: Vec<&str> = summary.split('.').collect();
    for sentence in &sentences {
        let lower = sentence.to_lowercase();
        if keywords.iter().any(|kw| lower.contains(kw)) {
            let trimmed = sentence.trim();
            if !trimmed.is_empty() && trimmed.len() > 10 {
                // Truncate to ~80 chars
                let truncated: String = trimmed.chars().take(80).collect();
                return if trimmed.len() > 80 {
                    format!("{}…", truncated.trim_end())
                } else {
                    truncated
                };
            }
        }
    }

    // Fallback: grab first N words from the summary
    let snippet: String = words.iter().take(12).cloned().collect::<Vec<_>>().join(" ");
    if snippet.is_empty() {
        "N/A".to_string()
    } else {
        format!("{snippet}…")
    }
}

/// Render a markdown comparison table.
fn render_markdown_table(
    items: &[String],
    dimensions: &[String],
    cells: &[ComparisonCell],
) -> String {
    if items.is_empty() || dimensions.is_empty() {
        return String::new();
    }

    let mut out = String::new();

    // Header
    out.push_str("| Dimension |");
    for item in items {
        out.push_str(&format!(" {} |", capitalise(item)));
    }
    out.push('\n');

    // Separator
    out.push_str("|-----------|");
    for _ in items {
        out.push_str("--------|");
    }
    out.push('\n');

    // Rows
    for dim in dimensions {
        out.push_str(&format!("| **{}** |", dim));
        for item in items {
            let value = cells
                .iter()
                .find(|c| &c.item == item && &c.dimension == dim)
                .map(|c| c.value.as_str())
                .unwrap_or("N/A");
            out.push_str(&format!(" {} |", value.replace('|', "\\|")));
        }
        out.push('\n');
    }

    out
}

fn capitalise(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_comparison_produces_table() {
        let query = parse_comparison_query("Rust vs Go");
        let item_data = vec![
            (
                "rust".to_string(),
                "Rust is fast and has great performance. It has a steep learning curve.".to_string(),
                vec!["https://rust-lang.org".to_string()],
            ),
            (
                "go".to_string(),
                "Go is easy to learn and has a good community ecosystem.".to_string(),
                vec!["https://golang.org".to_string()],
            ),
        ];
        let result = build_comparison(&query, &item_data);
        assert_eq!(result.items.len(), 2);
        assert!(!result.markdown_table.is_empty());
        assert!(result.markdown_table.contains("Rust") || result.markdown_table.contains("rust"));
    }

    #[test]
    fn markdown_table_has_header() {
        let query = parse_comparison_query("A vs B");
        let item_data = vec![
            ("a".to_string(), "A is good.".to_string(), vec![]),
            ("b".to_string(), "B is better.".to_string(), vec![]),
        ];
        let result = build_comparison(&query, &item_data);
        assert!(result.markdown_table.contains("Dimension"));
        assert!(result.markdown_table.contains("---"));
    }

    #[test]
    fn extract_dimension_value_finds_relevant_sentence() {
        let summary = "Rust is blazingly fast. Learning Rust has a steep learning curve for beginners.";
        let val = extract_dimension_value(summary, "Learning Curve");
        assert!(!val.is_empty());
        assert_ne!(val, "N/A");
    }
}
