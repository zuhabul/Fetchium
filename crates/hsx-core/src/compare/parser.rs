//! Parse comparison queries like "A vs B vs C" into individual items (Phase 5, PRD §10).

/// A parsed comparison query.
#[derive(Debug, Clone)]
pub struct ComparisonQuery {
    /// The individual items to compare.
    pub items: Vec<String>,
    /// Original query string.
    pub raw_query: String,
}

/// Parse a comparison query into its constituent items.
///
/// Recognises common separators: "vs", "versus", "or", "compared to", "vs.".
pub fn parse_comparison_query(query: &str) -> ComparisonQuery {
    let separators = [" vs ", " versus ", " or ", " compared to ", " vs. "];

    for sep in &separators {
        if query.to_lowercase().contains(sep) {
            let items: Vec<String> = query
                .to_lowercase()
                .split(sep)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if items.len() >= 2 {
                return ComparisonQuery {
                    items,
                    raw_query: query.to_string(),
                };
            }
        }
    }

    // "compare X Y Z" prefix
    let lower = query.to_lowercase();
    if let Some(rest) = lower.strip_prefix("compare ") {
        let items: Vec<String> = rest.split_whitespace().map(|s| s.to_string()).collect();
        if items.len() >= 2 {
            return ComparisonQuery {
                items,
                raw_query: query.to_string(),
            };
        }
    }

    // Fallback: treat the whole query as one item
    ComparisonQuery {
        items: vec![query.to_string()],
        raw_query: query.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_vs_separator() {
        let q = parse_comparison_query("React vs Vue vs Svelte");
        assert_eq!(q.items, vec!["react", "vue", "svelte"]);
    }

    #[test]
    fn parse_versus() {
        let q = parse_comparison_query("Rust versus Go");
        assert_eq!(q.items, vec!["rust", "go"]);
    }

    #[test]
    fn parse_compared_to() {
        let q = parse_comparison_query("Python compared to JavaScript");
        assert_eq!(q.items, vec!["python", "javascript"]);
    }

    #[test]
    fn parse_compare_prefix() {
        let q = parse_comparison_query("compare Redis Memcached");
        assert_eq!(q.items.len(), 2);
    }

    #[test]
    fn single_item_fallback() {
        let q = parse_comparison_query("Rust programming language");
        assert_eq!(q.items.len(), 1);
    }
}
