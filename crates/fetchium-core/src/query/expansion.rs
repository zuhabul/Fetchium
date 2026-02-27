//! Query Expansion Engine (QXE) — automatic synonym and acronym expansion.
//!
//! Novel algorithm: Expand queries with synonyms, related terms, and acronym
//! expansions to improve recall without sacrificing precision.
//!
//! - "JS frameworks" → also searches "JavaScript frameworks"
//! - "ML models" → "machine learning models"
//! - "k8s deployment" → "kubernetes deployment"
//!
//! Uses a compact, hand-curated dictionary optimized for technical queries.
//! Expansion is controlled to prevent query drift.

/// An expanded query with its variants.
#[derive(Debug, Clone)]
pub struct ExpandedQuery {
    /// The original query, unchanged.
    pub original: String,
    /// Expanded variants to also search for.
    pub expansions: Vec<QueryVariant>,
}

/// A single query variant produced by expansion.
#[derive(Debug, Clone)]
pub struct QueryVariant {
    /// The expanded query text.
    pub text: String,
    /// Confidence that this expansion is relevant (0.0-1.0).
    pub confidence: f64,
    /// What type of expansion produced this variant.
    pub expansion_type: ExpansionType,
}

/// Types of query expansion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpansionType {
    /// Acronym → full form ("JS" → "JavaScript")
    AcronymExpansion,
    /// Full form → acronym ("JavaScript" → "JS")
    AcronymContraction,
    /// Synonym substitution ("fast" → "quick")
    Synonym,
    /// Related term addition
    Related,
}

/// Acronym dictionary — maps abbreviations to full forms.
const ACRONYMS: &[(&str, &str)] = &[
    ("js", "javascript"),
    ("ts", "typescript"),
    ("py", "python"),
    ("rb", "ruby"),
    ("rs", "rust"),
    ("ml", "machine learning"),
    ("ai", "artificial intelligence"),
    ("dl", "deep learning"),
    ("nlp", "natural language processing"),
    ("cv", "computer vision"),
    ("llm", "large language model"),
    ("gpt", "generative pre-trained transformer"),
    ("rl", "reinforcement learning"),
    ("nn", "neural network"),
    ("cnn", "convolutional neural network"),
    ("rnn", "recurrent neural network"),
    ("api", "application programming interface"),
    ("sdk", "software development kit"),
    ("cli", "command line interface"),
    ("gui", "graphical user interface"),
    ("db", "database"),
    ("sql", "structured query language"),
    ("nosql", "not only sql"),
    ("orm", "object relational mapping"),
    ("css", "cascading style sheets"),
    ("html", "hypertext markup language"),
    ("http", "hypertext transfer protocol"),
    ("rest", "representational state transfer"),
    ("grpc", "remote procedure call"),
    ("ci", "continuous integration"),
    ("cd", "continuous deployment"),
    ("k8s", "kubernetes"),
    ("tf", "terraform"),
    ("aws", "amazon web services"),
    ("gcp", "google cloud platform"),
    ("vm", "virtual machine"),
    ("os", "operating system"),
    ("cpu", "central processing unit"),
    ("gpu", "graphics processing unit"),
    ("ram", "random access memory"),
    ("ssd", "solid state drive"),
    ("ux", "user experience"),
    ("ui", "user interface"),
    ("seo", "search engine optimization"),
    ("mvp", "minimum viable product"),
    ("saas", "software as a service"),
    ("iot", "internet of things"),
    ("tls", "transport layer security"),
    ("jwt", "json web token"),
    ("oauth", "open authorization"),
    ("xss", "cross site scripting"),
    ("dos", "denial of service"),
    ("wasm", "webassembly"),
    ("ssr", "server side rendering"),
    ("csr", "client side rendering"),
    ("ssg", "static site generation"),
];

/// Synonym pairs — bidirectional substitutions.
const SYNONYMS: &[(&str, &str)] = &[
    ("fast", "quick"),
    ("error", "bug"),
    ("fix", "resolve"),
    ("issue", "problem"),
    ("tutorial", "guide"),
    ("library", "package"),
    ("framework", "library"),
    ("install", "setup"),
    ("config", "configuration"),
    ("deploy", "deployment"),
    ("auth", "authentication"),
    ("async", "asynchronous"),
    ("sync", "synchronous"),
    ("perf", "performance"),
    ("benchmark", "performance test"),
    ("regex", "regular expression"),
];

/// Expand a query with synonyms and acronyms.
///
/// Returns the original query plus expanded variants, each with a
/// confidence score. Higher confidence = more likely to be helpful.
pub fn expand_query(query: &str) -> ExpandedQuery {
    let original = query.to_string();
    let lower = query.to_lowercase();
    let tokens: Vec<&str> = lower.split_whitespace().collect();
    let mut expansions = Vec::new();

    // 1. Acronym expansion: replace abbreviations with full forms
    for (abbrev, full) in ACRONYMS {
        if tokens
            .iter()
            .any(|t| t.trim_matches(|c: char| c.is_ascii_punctuation()) == *abbrev)
        {
            let expanded = lower.replace(abbrev, full);
            if expanded != lower {
                expansions.push(QueryVariant {
                    text: expanded,
                    confidence: 0.9,
                    expansion_type: ExpansionType::AcronymExpansion,
                });
            }
        }
    }

    // 2. Reverse acronym: if query has full form, also try abbreviation
    for (abbrev, full) in ACRONYMS {
        if lower.contains(full) {
            let contracted = lower.replace(full, abbrev);
            if contracted != lower {
                expansions.push(QueryVariant {
                    text: contracted,
                    confidence: 0.7,
                    expansion_type: ExpansionType::AcronymContraction,
                });
            }
        }
    }

    // 3. Synonym substitution
    for (a, b) in SYNONYMS {
        if tokens
            .iter()
            .any(|t| t.trim_matches(|c: char| c.is_ascii_punctuation()) == *a)
        {
            let expanded = lower.replace(a, b);
            if expanded != lower {
                expansions.push(QueryVariant {
                    text: expanded,
                    confidence: 0.6,
                    expansion_type: ExpansionType::Synonym,
                });
            }
        }
        // Bidirectional
        if tokens
            .iter()
            .any(|t| t.trim_matches(|c: char| c.is_ascii_punctuation()) == *b)
        {
            let expanded = lower.replace(b, a);
            if expanded != lower {
                expansions.push(QueryVariant {
                    text: expanded,
                    confidence: 0.6,
                    expansion_type: ExpansionType::Synonym,
                });
            }
        }
    }

    // Deduplicate expansions
    let mut seen = std::collections::HashSet::new();
    seen.insert(lower.clone());
    expansions.retain(|v| seen.insert(v.text.clone()));

    // Sort by confidence descending
    expansions.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Limit to top 3 expansions to prevent query drift
    expansions.truncate(3);

    ExpandedQuery {
        original,
        expansions,
    }
}

/// Check if a query would benefit from expansion.
///
/// Returns true if the query contains any known acronyms or synonyms.
pub fn should_expand(query: &str) -> bool {
    let lower = query.to_lowercase();
    let tokens: Vec<&str> = lower.split_whitespace().collect();

    // Check acronyms
    for (abbrev, _) in ACRONYMS {
        if tokens
            .iter()
            .any(|t| t.trim_matches(|c: char| c.is_ascii_punctuation()) == *abbrev)
        {
            return true;
        }
    }

    // Check synonyms
    for (a, b) in SYNONYMS {
        if tokens.iter().any(|t| {
            let clean = t.trim_matches(|c: char| c.is_ascii_punctuation());
            clean == *a || clean == *b
        }) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn js_expands_to_javascript() {
        let expanded = expand_query("JS frameworks comparison");
        assert!(
            expanded
                .expansions
                .iter()
                .any(|v| v.text.contains("javascript")),
            "JS should expand to JavaScript, got: {:?}",
            expanded.expansions
        );
    }

    #[test]
    fn ml_expands_to_machine_learning() {
        let expanded = expand_query("ML models for text");
        assert!(
            expanded
                .expansions
                .iter()
                .any(|v| v.text.contains("machine learning")),
            "ML should expand to machine learning"
        );
    }

    #[test]
    fn k8s_expands_to_kubernetes() {
        let expanded = expand_query("k8s deployment guide");
        assert!(
            expanded
                .expansions
                .iter()
                .any(|v| v.text.contains("kubernetes")),
            "k8s should expand to kubernetes"
        );
    }

    #[test]
    fn synonym_expansion() {
        let expanded = expand_query("fast database library");
        let has_quick = expanded.expansions.iter().any(|v| v.text.contains("quick"));
        let has_package = expanded
            .expansions
            .iter()
            .any(|v| v.text.contains("package"));
        assert!(
            has_quick || has_package,
            "Should expand synonyms: {:?}",
            expanded.expansions
        );
    }

    #[test]
    fn no_expansion_for_unknown() {
        let expanded = expand_query("quantum entanglement theory");
        assert!(
            expanded.expansions.is_empty(),
            "Unknown terms should not expand"
        );
    }

    #[test]
    fn preserves_original() {
        let expanded = expand_query("JS tutorial");
        assert_eq!(expanded.original, "JS tutorial");
    }

    #[test]
    fn max_three_expansions() {
        let expanded = expand_query("fast ML api tutorial");
        assert!(
            expanded.expansions.len() <= 3,
            "Should limit to 3 expansions, got {}",
            expanded.expansions.len()
        );
    }

    #[test]
    fn should_expand_detects_acronyms() {
        assert!(should_expand("JS frameworks"));
        assert!(should_expand("deploy ML model"));
        assert!(!should_expand("quantum physics"));
    }

    #[test]
    fn deduplicates_expansions() {
        let expanded = expand_query("JS JS tutorial");
        let texts: Vec<&str> = expanded
            .expansions
            .iter()
            .map(|v| v.text.as_str())
            .collect();
        let unique: std::collections::HashSet<&&str> = texts.iter().collect();
        assert_eq!(
            texts.len(),
            unique.len(),
            "Should not have duplicate expansions"
        );
    }

    #[test]
    fn confidence_ordering() {
        let expanded = expand_query("fast JS api");
        for i in 1..expanded.expansions.len() {
            assert!(
                expanded.expansions[i - 1].confidence >= expanded.expansions[i].confidence,
                "Expansions should be sorted by confidence"
            );
        }
    }
}
