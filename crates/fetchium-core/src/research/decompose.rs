//! Query decomposition — breaks complex queries into parallel sub-questions (PRD §10).

/// Decompose a complex query into parallel sub-questions.
///
/// Generates diverse perspective-aware sub-queries for broader coverage:
/// - Original query (always included)
/// - Comparison items (if "vs" or "compare" detected)
/// - Implication/impact subjects
/// - Perspective queries (factual, recent, expert)
pub fn decompose_query(query: &str) -> Vec<String> {
    let lower = query.to_lowercase();
    let mut sub_queries = Vec::new();

    // Pattern: "compare X vs Y vs Z" -> individual queries per item
    if lower.contains(" vs ") || lower.contains("compare") {
        let items = extract_comparison_items(query);
        if items.len() >= 2 {
            for item in &items {
                sub_queries.push(format!("{item} features overview"));
            }
            sub_queries.push(query.to_string());
            return sub_queries;
        }
    }

    // Pattern: "X implications for Y" -> query about X, query about Y, original
    if lower.contains("implications") || lower.contains("impact") {
        sub_queries.push(query.to_string());
        if let Some(pos) = lower.find(" for ") {
            let topic = &query[..pos];
            let context = &query[pos + 5..];
            sub_queries.push(topic.trim().to_string());
            sub_queries.push(context.trim().to_string());
        }
        return sub_queries;
    }

    // Default: generate perspective-aware sub-queries for richer coverage
    sub_queries.push(query.to_string());

    // Extract the core topic (strip question words)
    let topic = extract_topic(query);
    if topic.len() > 3 {
        // Add a "recent developments" perspective for freshness
        sub_queries.push(format!("{topic} 2025 2026 latest"));
        // Add an expert/detailed perspective
        sub_queries.push(format!("{topic} explained overview"));
    }

    sub_queries
}

/// Extract the core topic from a query by stripping question words.
fn extract_topic(query: &str) -> String {
    let lower = query.to_lowercase();
    let stripped = lower
        .trim_start_matches("what is ")
        .trim_start_matches("what are ")
        .trim_start_matches("how does ")
        .trim_start_matches("how do ")
        .trim_start_matches("how to ")
        .trim_start_matches("why is ")
        .trim_start_matches("why are ")
        .trim_start_matches("explain ")
        .trim_start_matches("describe ")
        .trim_start_matches("tell me about ")
        .trim_end_matches('?')
        .trim();
    stripped.to_string()
}

fn extract_comparison_items(query: &str) -> Vec<String> {
    let cleaned = query
        .to_lowercase()
        .replace("compare ", "")
        .replace("comparison of ", "")
        .replace(" and ", " vs ");
    cleaned
        .split(" vs ")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparison_decomposition() {
        let subs = decompose_query("compare Rust vs Go vs C++");
        assert!(subs.len() >= 3);
        assert!(subs.iter().any(|q| q.to_lowercase().contains("rust")));
        assert!(subs.iter().any(|q| q.to_lowercase().contains("go")));
    }

    #[test]
    fn simple_query_no_decomposition() {
        let subs = decompose_query("what is Rust");
        assert!(subs.len() >= 1);
        assert!(subs
            .iter()
            .any(|q| q.contains("Rust") || q.contains("rust")));
    }

    #[test]
    fn implications_decomposition() {
        let subs = decompose_query("GDPR implications for AI training");
        assert!(subs.len() >= 2);
    }

    #[test]
    fn vs_query_includes_original() {
        let subs = decompose_query("Rust vs Python performance");
        assert!(subs.iter().any(|q| q.contains("Rust vs Python")));
    }

    #[test]
    fn perspective_aware_queries() {
        let subs = decompose_query("how does async work in Rust");
        assert!(subs.len() >= 2);
        // Should have original + at least one perspective query
        assert!(subs[0].contains("async"));
    }
}
