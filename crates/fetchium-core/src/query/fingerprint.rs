//! Query Fingerprinting and Deduplication (QFD) — canonical query normalization.
//!
//! Novel algorithm: Normalize and fingerprint queries so semantically identical
//! queries produce the same cache key. Examples:
//! - "rust vs go" ≡ "go vs rust" ≡ "comparing rust and go"
//! - "what is machine learning" ≡ "machine learning" (stop-word removal)
//! - "JavaScript" ≡ "javascript" (case normalization)
//!
//! Uses canonical ordering of comparison terms, stop-word removal, stemming
//! approximation, and SimHash for near-duplicate detection.

/// Stop words removed during fingerprinting.
const STOP_WORDS: &[&str] = &[
    "a", "an", "the", "is", "are", "was", "were", "be", "been", "being", "what", "which", "who",
    "whom", "this", "that", "these", "those", "am", "do", "does", "did", "will", "would", "shall",
    "should", "can", "could", "may", "might", "must", "need", "dare", "i", "me", "my", "we", "our",
    "you", "your", "he", "she", "it", "they", "them", "their", "its", "has", "have", "had",
    "having", "in", "on", "at", "to", "for", "of", "with", "by", "from", "about", "into",
    "through", "during", "before", "after", "how", "why", "when", "where",
];

/// Comparison operators that signal a "versus" query.
const VS_OPERATORS: &[&str] = &[
    "vs",
    "vs.",
    "versus",
    "compared",
    "comparison",
    "comparing",
    "against",
    "differ",
    "difference",
    "differences",
    "between",
    "better",
    "worse",
    "or",
];

/// A fingerprinted query with its canonical form and hash.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryFingerprint {
    /// Canonical normalized form of the query.
    pub canonical: String,
    /// SimHash fingerprint (64-bit).
    pub simhash: u64,
    /// Whether this is a comparison query.
    pub is_comparison: bool,
    /// The extracted comparison entities (if comparison query).
    pub entities: Vec<String>,
}

/// Fingerprint a query into its canonical form.
///
/// The normalization pipeline:
/// 1. Lowercase
/// 2. Remove punctuation (except hyphens in compound words)
/// 3. Remove stop words
/// 4. Detect comparison pattern → sort entities alphabetically
/// 5. Apply lightweight stemming (trailing -s, -ing, -ed removal)
/// 6. Sort remaining terms for order-independence
/// 7. Compute SimHash over the final token set
pub fn fingerprint(query: &str) -> QueryFingerprint {
    let lower = query.to_lowercase();

    // Tokenize: split on whitespace and punctuation (keep hyphens in words)
    let tokens: Vec<String> = lower
        .split(|c: char| c.is_whitespace() || (c.is_ascii_punctuation() && c != '-'))
        .filter(|t| !t.is_empty())
        .map(|t| t.trim_matches('-').to_string())
        .filter(|t| !t.is_empty())
        .collect();

    // Detect comparison pattern
    let has_vs = tokens.iter().any(|t| VS_OPERATORS.contains(&t.as_str()));
    let (is_comparison, entities) = if has_vs {
        extract_comparison_entities(&tokens)
    } else {
        (false, Vec::new())
    };

    // Remove stop words and comparison operators
    let meaningful: Vec<String> = tokens
        .iter()
        .filter(|t| !STOP_WORDS.contains(&t.as_str()))
        .filter(|t| !VS_OPERATORS.contains(&t.as_str()))
        .map(|t| lightweight_stem(t))
        .collect();

    // Build canonical form
    let canonical = if is_comparison && entities.len() >= 2 {
        // For comparison queries, sort entities alphabetically
        let mut sorted_entities = entities.clone();
        sorted_entities.sort();
        format!("{} vs", sorted_entities.join(" "))
    } else {
        // For regular queries, sort all terms
        let mut sorted: Vec<String> = meaningful.clone();
        sorted.sort();
        sorted.dedup();
        sorted.join(" ")
    };

    let simhash = compute_simhash(&meaningful);

    QueryFingerprint {
        canonical,
        simhash,
        is_comparison,
        entities,
    }
}

/// Check if two queries are near-duplicates based on SimHash distance.
///
/// Returns true if the Hamming distance between fingerprints is <= threshold.
/// Default threshold of 8 catches ~87% similar queries.
pub fn is_near_duplicate(a: &QueryFingerprint, b: &QueryFingerprint, threshold: u32) -> bool {
    if a.canonical == b.canonical {
        return true;
    }
    hamming_distance(a.simhash, b.simhash) <= threshold
}

/// Compute Hamming distance between two 64-bit hashes.
fn hamming_distance(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}

/// Extract comparison entities from tokens containing a VS operator.
///
/// Handles patterns like:
/// - "rust vs go" → entities: ["go", "rust"]
/// - "comparing rust and go" → entities: ["go", "rust"]
/// - "rust compared to go" → entities: ["go", "rust"]
fn extract_comparison_entities(tokens: &[String]) -> (bool, Vec<String>) {
    // Find the position of the comparison operator
    let vs_pos = tokens
        .iter()
        .position(|t| VS_OPERATORS.contains(&t.as_str()));

    match vs_pos {
        Some(pos) => {
            let left: Vec<String> = tokens[..pos]
                .iter()
                .filter(|t| !STOP_WORDS.contains(&t.as_str()))
                .filter(|t| !VS_OPERATORS.contains(&t.as_str()))
                .map(|t| lightweight_stem(t))
                .collect();

            let right: Vec<String> = tokens[pos + 1..]
                .iter()
                .filter(|t| !STOP_WORDS.contains(&t.as_str()))
                .filter(|t| !VS_OPERATORS.contains(&t.as_str()))
                .map(|t| lightweight_stem(t))
                .collect();

            // For "comparing X and Y" patterns: operator is first token,
            // split the raw tokens (before stop-word removal) on "and"
            if left.is_empty() {
                let raw_right = &tokens[pos + 1..];
                let and_pos = raw_right.iter().position(|t| t == "and" || t == "&");
                if let Some(ap) = and_pos {
                    let entity_a: String = raw_right[..ap]
                        .iter()
                        .filter(|t| !STOP_WORDS.contains(&t.as_str()))
                        .map(|t| lightweight_stem(t))
                        .collect::<Vec<_>>()
                        .join(" ");
                    let entity_b: String = raw_right[ap + 1..]
                        .iter()
                        .filter(|t| !STOP_WORDS.contains(&t.as_str()))
                        .filter(|t| !VS_OPERATORS.contains(&t.as_str()))
                        .map(|t| lightweight_stem(t))
                        .collect::<Vec<_>>()
                        .join(" ");
                    if !entity_a.is_empty() && !entity_b.is_empty() {
                        let mut entities = vec![entity_a, entity_b];
                        entities.sort();
                        return (true, entities);
                    }
                }
                // Fallback: if right side has 2+ meaningful tokens, split in half
                if right.len() >= 2 {
                    let mid = right.len() / 2;
                    let entity_a = right[..mid].join(" ");
                    let entity_b = right[mid..].join(" ");
                    let mut entities = vec![entity_a, entity_b];
                    entities.sort();
                    return (true, entities);
                }
                return (false, Vec::new());
            }

            if left.is_empty() || right.is_empty() {
                return (false, Vec::new());
            }

            let left_entity = left.join(" ");
            let right_entity = right.join(" ");

            let mut entities = vec![left_entity, right_entity];
            entities.sort();
            (true, entities)
        }
        None => (false, Vec::new()),
    }
}

/// Lightweight stemming — strip common English suffixes.
///
/// This is intentionally aggressive to maximize cache hits.
/// Not linguistically perfect, but good enough for query dedup.
fn lightweight_stem(word: &str) -> String {
    let w = word.to_lowercase();

    // Don't stem very short words
    if w.len() <= 3 {
        return w;
    }

    // Strip -ing (but keep at least 3 chars)
    if w.ends_with("ing") && w.len() > 6 {
        return w[..w.len() - 3].to_string();
    }

    // Strip -tion/-sion (keep at least 3 chars of stem)
    if (w.ends_with("tion") || w.ends_with("sion")) && w.len() >= 7 {
        return w[..w.len() - 4].to_string();
    }

    // Strip -ed (but not -eed, -ied)
    if w.ends_with("ed") && !w.ends_with("eed") && !w.ends_with("ied") && w.len() > 4 {
        return w[..w.len() - 2].to_string();
    }

    // Strip trailing -s (but not -ss, -us, -is)
    if w.ends_with('s')
        && !w.ends_with("ss")
        && !w.ends_with("us")
        && !w.ends_with("is")
        && w.len() > 3
    {
        return w[..w.len() - 1].to_string();
    }

    w
}

/// Compute a 64-bit SimHash over a set of tokens.
///
/// Each token gets a 64-bit FNV-1a hash, then we sum bit weights
/// across all tokens. Final fingerprint: bit i = 1 if weight[i] > 0.
fn compute_simhash(tokens: &[String]) -> u64 {
    if tokens.is_empty() {
        return 0;
    }

    let mut weights = [0i32; 64];

    for token in tokens {
        let hash = fnv1a_64(token);
        for (i, weight) in weights.iter_mut().enumerate() {
            if (hash >> i) & 1 == 1 {
                *weight += 1;
            } else {
                *weight -= 1;
            }
        }
    }

    let mut fingerprint: u64 = 0;
    for (i, &w) in weights.iter().enumerate() {
        if w > 0 {
            fingerprint |= 1u64 << i;
        }
    }
    fingerprint
}

/// FNV-1a hash (64-bit) for a string.
fn fnv1a_64(s: &str) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x00000100000001B3;

    let mut hash = FNV_OFFSET;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Generate a cache key from a query fingerprint.
///
/// Suitable for use as a cache lookup key that's resistant to
/// query reformulation.
pub fn cache_key(fp: &QueryFingerprint) -> String {
    format!("qfp:{}", fp.canonical)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_vs_go_order_invariant() {
        let a = fingerprint("rust vs go");
        let b = fingerprint("go vs rust");
        assert_eq!(a.canonical, b.canonical);
        assert!(a.is_comparison);
        assert!(b.is_comparison);
    }

    #[test]
    fn comparing_detected_as_vs() {
        let a = fingerprint("rust vs go");
        let b = fingerprint("comparing rust and go");
        // Both should be detected as comparison queries
        assert!(a.is_comparison);
        assert!(b.is_comparison);
        // Both should have the same sorted entities
        assert_eq!(a.entities, b.entities);
    }

    #[test]
    fn stop_words_removed() {
        let a = fingerprint("what is machine learning");
        let b = fingerprint("machine learning");
        assert_eq!(a.canonical, b.canonical);
    }

    #[test]
    fn case_insensitive() {
        let a = fingerprint("JavaScript Frameworks");
        let b = fingerprint("javascript frameworks");
        assert_eq!(a.canonical, b.canonical);
    }

    #[test]
    fn near_duplicate_detection() {
        let a = fingerprint("best rust web frameworks 2024");
        let b = fingerprint("top rust web framework 2024");
        // These share most terms after stemming — distance should be moderate
        let distance = hamming_distance(a.simhash, b.simhash);
        assert!(
            is_near_duplicate(&a, &b, 16),
            "Expected near-duplicate: a={}, b={}, distance={}",
            a.canonical,
            b.canonical,
            distance,
        );
    }

    #[test]
    fn different_queries_not_duplicates() {
        let a = fingerprint("rust programming language");
        let b = fingerprint("cooking italian pasta recipes");
        assert!(
            !is_near_duplicate(&a, &b, 5),
            "Unrelated queries should not be duplicates"
        );
    }

    #[test]
    fn lightweight_stem_strips_suffixes() {
        assert_eq!(lightweight_stem("programming"), "programm");
        assert_eq!(lightweight_stem("languages"), "language");
        assert_eq!(lightweight_stem("compared"), "compar");
        assert_eq!(lightweight_stem("education"), "educa"); // 9 chars, strips -tion
        assert_eq!(lightweight_stem("action"), "action"); // too short to stem safely
    }

    #[test]
    fn lightweight_stem_preserves_short_words() {
        assert_eq!(lightweight_stem("go"), "go");
        assert_eq!(lightweight_stem("is"), "is");
        assert_eq!(lightweight_stem("the"), "the");
    }

    #[test]
    fn cache_key_deterministic() {
        let a = cache_key(&fingerprint("rust vs go"));
        let b = cache_key(&fingerprint("go vs rust"));
        assert_eq!(a, b, "Same query should produce same cache key");
    }

    #[test]
    fn empty_query() {
        let fp = fingerprint("");
        assert_eq!(fp.canonical, "");
        assert!(!fp.is_comparison);
    }

    #[test]
    fn simhash_deterministic() {
        let a = fingerprint("rust programming");
        let b = fingerprint("rust programming");
        assert_eq!(a.simhash, b.simhash);
    }
}
