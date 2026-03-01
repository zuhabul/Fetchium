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
    let dimensions: Vec<String> = STANDARD_DIMENSIONS.iter().map(|s| s.to_string()).collect();

    let mut cells = Vec::new();

    for (item, summary, sources) in item_data {
        // Extract a value for each dimension — BM25-scored with item awareness
        for dim in &dimensions {
            let value = extract_dimension_value(summary, dim, item);
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

/// Extract a value for a dimension from a text summary using BM25 scoring.
///
/// 1. Split text into sentences
/// 2. Filter to sentences that mention the item (case-insensitive)
/// 3. Score each with BM25 against "{item} {dimension}"
/// 4. Return highest-scoring sentence (truncated to 120 chars)
/// 5. Falls back to "Insufficient data" instead of garbage
fn extract_dimension_value(summary: &str, dimension: &str, item: &str) -> String {
    if summary.trim().is_empty() {
        return "Insufficient data".to_string();
    }

    let item_lower = item.to_lowercase();
    let dim_lower = dimension.to_lowercase();
    let bm25_query = format!("{} {}", item, dimension);

    // Split into sentences (handle multiple delimiters)
    let sentences: Vec<&str> = summary
        .split(['.', '!', '?'])
        .map(|s| s.trim())
        .filter(|s| s.len() > 15) // skip very short fragments
        .collect();

    if sentences.is_empty() {
        return "Insufficient data".to_string();
    }

    // Score sentences: prefer those mentioning the item AND relevant to dimension
    let mut scored: Vec<(f64, &str)> = sentences
        .iter()
        .map(|&sentence| {
            let lower = sentence.to_lowercase();
            let mut score = crate::rank::bm25_score(sentence, &bm25_query);

            // Bonus for mentioning the item directly
            if lower.contains(&item_lower) {
                score += 0.5;
            }

            // Bonus for dimension-related keywords
            let keywords = dimension_keywords(&dim_lower);
            for kw in keywords {
                if lower.contains(kw) {
                    score += 0.3;
                }
            }

            (score, sentence)
        })
        .collect();

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    // Take best scoring sentence
    if let Some((score, best)) = scored.first() {
        if *score < 0.1 {
            return "Insufficient data".to_string();
        }
        let truncated: String = best.chars().take(120).collect();
        if best.len() > 120 {
            format!("{}…", truncated.trim_end())
        } else {
            truncated
        }
    } else {
        "Insufficient data".to_string()
    }
}

/// Return dimension-specific keywords for scoring boosts.
fn dimension_keywords(dim_lower: &str) -> &'static [&'static str] {
    match dim_lower {
        "performance" => &[
            "fast",
            "slow",
            "speed",
            "benchmark",
            "performance",
            "throughput",
            "latency",
            "efficient",
        ],
        "learning curve" => &[
            "easy",
            "difficult",
            "complex",
            "beginner",
            "learn",
            "learning",
            "intuitive",
            "documentation",
        ],
        "ecosystem" => &[
            "libraries",
            "packages",
            "npm",
            "crate",
            "ecosystem",
            "plugins",
            "tools",
            "framework",
        ],
        "community" => &[
            "community",
            "users",
            "popular",
            "adoption",
            "developers",
            "contributors",
            "active",
        ],
        "production readiness" => &[
            "stable",
            "production",
            "enterprise",
            "mature",
            "ready",
            "reliable",
            "battle-tested",
        ],
        "license" => &[
            "mit",
            "apache",
            "gpl",
            "bsd",
            "license",
            "open source",
            "proprietary",
        ],
        "key use case" => &[
            "use case",
            "application",
            "web",
            "server",
            "mobile",
            "cli",
            "ideal for",
            "best for",
        ],
        _ => &[],
    }
}

/// Configuration for AI-powered comparison.
#[derive(Debug, Clone, Default)]
pub struct AiCompareConfig {
    /// Whether to use AI for dimension extraction.
    pub use_ai: bool,
    /// Optional model override.
    pub model: Option<String>,
}

/// Build a comparison using AI with a SINGLE call for the entire table.
///
/// Uses all collected search snippets as context and asks the AI to fill
/// every cell at once. Much faster and more accurate than per-item calls.
pub async fn build_comparison_ai_unified(
    query: &ComparisonQuery,
    snippet_text: &str,
    sources: &[String],
    hsx_config: &crate::config::HsxConfig,
) -> ComparisonResult {
    use crate::ai::types::{AiConfig, ChatMessage};

    let items: Vec<String> = query.items.clone();
    let dimensions: Vec<String> = STANDARD_DIMENSIONS.iter().map(|s| s.to_string()).collect();
    let ai_config = AiConfig::from_hsx_config(hsx_config);
    let providers = ai_config.providers.clone();

    let items_str = items
        .iter()
        .map(|i| capitalise(i))
        .collect::<Vec<_>>()
        .join(", ");
    let _dim_str = dimensions.join(", ");

    // Truncate snippet text to fit in context
    let context: String = snippet_text.chars().take(6000).collect();

    let prompt = format!(
        r#"Compare these items: {items_str}

Context from the user's query: "{raw_query}"

Below are search results that MAY contain relevant information. WARNING: Search results
can be noisy or irrelevant (e.g., "Go" may match "GoPro" cameras, "Rust" may match the
video game). IGNORE any search result that is not actually about the items being compared.
Use YOUR OWN KNOWLEDGE as the primary source of information.

SEARCH RESULTS (use only if relevant):
{context}

Fill in this comparison table with factual, specific assessments.

Respond in EXACTLY this format:

{format_str}

Rules:
- Use YOUR knowledge as primary source — search results are supplementary
- Be specific: cite numbers, stats, percentages where possible
- Keep each value to 1-2 sentences max
- IGNORE irrelevant search results entirely
- If genuinely unknown, write "Insufficient data"
- Do NOT repeat the dimension name in the value"#,
        raw_query = query.raw_query,
        format_str = items
            .iter()
            .map(|item| {
                let cap = capitalise(item);
                dimensions
                    .iter()
                    .map(|d| format!("[{cap}] {d}: ..."))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    );

    let messages = vec![
        ChatMessage {
            role: "system".into(),
            content: "You are a technical comparison expert. Produce concise, factual comparisons."
                .into(),
        },
        ChatMessage {
            role: "user".into(),
            content: prompt,
        },
    ];

    let mut noop = |_: &str| {};
    tracing::debug!(
        "Compare AI: calling chat_with_fallback with {} chars of context",
        context.len()
    );
    let ai_result = crate::ai::provider_client::chat_with_fallback(
        &messages, None, &ai_config, &providers, false, &mut noop,
    )
    .await;

    let mut cells = Vec::new();

    match ai_result {
        Ok(result) => {
            tracing::debug!(
                "Compare AI succeeded: {} chars response",
                result.content.len()
            );
            for item in &items {
                for dim in &dimensions {
                    let value = parse_unified_response(&result.content, item, dim);
                    cells.push(ComparisonCell {
                        item: item.clone(),
                        dimension: dim.clone(),
                        value,
                        sources: sources.to_vec(),
                    });
                }
            }
        }
        Err(e) => {
            tracing::warn!("Compare AI call failed: {e}, falling back to heuristic extraction");
            // Fall back to heuristic — build per-item data from snippet_text
            for item in &items {
                for dim in &dimensions {
                    let value = extract_dimension_value(snippet_text, dim, item);
                    cells.push(ComparisonCell {
                        item: item.clone(),
                        dimension: dim.clone(),
                        value,
                        sources: sources.to_vec(),
                    });
                }
            }
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

/// Parse unified AI response for a specific [Item] Dimension: value.
fn parse_unified_response(response: &str, item: &str, dimension: &str) -> String {
    let item_cap = capitalise(item);
    let prefix = format!("[{}] {}:", item_cap, dimension);
    let prefix_lower = prefix.to_lowercase();

    for line in response.lines() {
        let line_trimmed = line.trim();
        let line_lower = line_trimmed.to_lowercase();
        if line_lower.starts_with(&prefix_lower) {
            let value = line_trimmed[prefix.len()..].trim().to_string();
            if !value.is_empty() {
                return value;
            }
        }
    }

    // Fallback: try without the [Item] prefix (AI might use "Dimension:" format)
    let dim_prefix = format!("{}:", dimension);
    let dim_prefix_lower = dim_prefix.to_lowercase();
    for line in response.lines() {
        let line_lower = line.trim().to_lowercase();
        if line_lower.starts_with(&dim_prefix_lower) {
            let value = line.trim()[dim_prefix.len()..].trim().to_string();
            if !value.is_empty() {
                return value;
            }
        }
    }

    "Insufficient data".to_string()
}

/// Build a comparison using AI to extract dimension values (per-item calls).
pub async fn build_comparison_ai(
    query: &ComparisonQuery,
    item_data: &[(String, String, Vec<String>)],
    hsx_config: &crate::config::HsxConfig,
) -> ComparisonResult {
    use crate::ai::types::{AiConfig, ChatMessage};

    let items: Vec<String> = query.items.clone();
    let dimensions: Vec<String> = STANDARD_DIMENSIONS.iter().map(|s| s.to_string()).collect();
    let mut cells = Vec::new();
    let ai_config = AiConfig::from_hsx_config(hsx_config);
    let providers = ai_config.providers.clone();

    for (item, summary, sources) in item_data {
        let snippet: String = summary.chars().take(3000).collect();
        let dim_list = dimensions.join(", ");

        let prompt = format!(
            "Based on the following information about \"{item}\", provide a brief (1-2 sentence) assessment for each dimension.\n\n\
             SOURCE TEXT:\n{snippet}\n\n\
             DIMENSIONS: {dim_list}\n\n\
             Respond in this exact format (one line per dimension):\n\
             Performance: ...\n\
             Learning Curve: ...\n\
             Ecosystem: ...\n\
             Community: ...\n\
             Production Readiness: ...\n\
             License: ...\n\
             Key Use Case: ...\n\n\
             Be specific and concise. If information is insufficient, say \"Insufficient data\"."
        );

        let messages = vec![
            ChatMessage {
                role: "system".into(),
                content: "You are a technical comparison assistant. Extract factual dimension values from source text.".into(),
            },
            ChatMessage {
                role: "user".into(),
                content: prompt,
            },
        ];

        let mut noop = |_: &str| {};
        let ai_result = crate::ai::provider_client::chat_with_fallback(
            &messages, None, &ai_config, &providers, false, &mut noop,
        )
        .await;

        match ai_result {
            Ok(result) => {
                // Parse AI response into dimension values
                for dim in &dimensions {
                    let value = parse_ai_dimension(&result.content, dim);
                    cells.push(ComparisonCell {
                        item: item.clone(),
                        dimension: dim.clone(),
                        value,
                        sources: sources.clone(),
                    });
                }
            }
            Err(_) => {
                // Fall back to BM25 extraction
                for dim in &dimensions {
                    let value = extract_dimension_value(summary, dim, item);
                    cells.push(ComparisonCell {
                        item: item.clone(),
                        dimension: dim.clone(),
                        value,
                        sources: sources.clone(),
                    });
                }
            }
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

/// Parse an AI response to extract the value for a specific dimension.
fn parse_ai_dimension(response: &str, dimension: &str) -> String {
    let dim_prefix = format!("{}:", dimension);
    let dim_prefix_lower = dim_prefix.to_lowercase();
    for line in response.lines() {
        let line_lower = line.to_lowercase().trim().to_string();
        if line_lower.starts_with(&dim_prefix_lower) {
            let value = line[dim_prefix.len()..].trim().to_string();
            if !value.is_empty() {
                return value;
            }
        }
    }
    "Insufficient data".to_string()
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
                "Rust is fast and has great performance. It has a steep learning curve."
                    .to_string(),
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
        let summary =
            "Rust is blazingly fast. Learning Rust has a steep learning curve for beginners.";
        let val = extract_dimension_value(summary, "Learning Curve", "Rust");
        assert!(!val.is_empty());
        assert_ne!(val, "Insufficient data");
    }
}
