//! Autoprompt — Exa-style local query rewriting for better backend results.
//!
//! Transforms raw user queries into optimized search queries without API calls.
//! Applied before dispatch to backends in the search orchestrator.

/// Result of autoprompt query rewriting.
#[derive(Debug, Clone)]
pub struct AutopromptResult {
    /// The rewritten query (may equal original if no changes needed).
    pub rewritten: String,
    /// The original query, unchanged.
    pub original: String,
    /// Whether any transformation was applied.
    pub changed: bool,
}

/// Rewrite a query for better search engine results.
///
/// Applies these transformations in order:
/// 1. Question form normalization ("what is X" → "X")
/// 2. Filler word removal ("please help me with X" → "X")
/// 3. Inline abbreviation expansion ("ML models" → "machine learning models")
/// 4. Length guard (truncate if > 150 chars)
pub fn autoprompt(query: &str) -> AutopromptResult {
    let original = query.to_string();
    let mut q = query.trim().to_string();

    if q.is_empty() {
        return AutopromptResult {
            rewritten: q,
            original,
            changed: false,
        };
    }

    // Skip rewriting for non-ASCII queries (multilingual) — our patterns are English-only
    if q.chars().any(|c| c.is_alphabetic() && !c.is_ascii()) {
        return AutopromptResult {
            rewritten: q,
            original,
            changed: false,
        };
    }

    let before = q.clone();

    // Step 1: Question form normalization
    q = normalize_question(&q);

    // Step 2: Filler word removal
    q = remove_fillers(&q);

    // Step 3: Inline abbreviation expansion
    q = expand_abbreviations(&q);

    // Step 4: Length guard
    if q.len() > 150 {
        q = q.chars().take(150).collect::<String>().trim().to_string();
    }

    // Clean up extra whitespace
    q = q.split_whitespace().collect::<Vec<_>>().join(" ");

    let changed = q != before;
    AutopromptResult {
        rewritten: q,
        original,
        changed,
    }
}

/// Strip question prefixes to extract key terms.
///
/// Only strips prefixes for short-to-medium queries (≤8 words after stripping).
/// Long queries already have enough signal — stripping "what is the" from a
/// 15-word query can hurt more than help.
fn normalize_question(query: &str) -> String {
    let lower = query.to_lowercase();
    let word_count = query.split_whitespace().count();

    // Don't rewrite long queries — they have enough keyword signal already
    if word_count > 10 {
        // Just strip trailing question mark
        if query.ends_with('?') {
            return query[..query.len() - 1].trim().to_string();
        }
        return query.to_string();
    }

    // Patterns to strip (ordered longest-first to avoid partial matches)
    const QUESTION_PREFIXES: &[&str] = &[
        "can someone explain ",
        "can you explain ",
        "could you explain ",
        "please explain ",
        "how do i ",
        "how do you ",
        "how does ",
        "how can i ",
        "how can you ",
        "how to ",
        "what are the ",
        "what is the ",
        "what are ",
        "what is a ",
        "what is an ",
        "what is ",
        "what does ",
        "why does ",
        "why is ",
        "why do ",
        "where can i find ",
        "where can i ",
        "where is ",
        "when did ",
        "when was ",
        "who is ",
        "who are ",
        "who was ",
        "tell me about ",
        "explain ",
        "define ",
        "describe ",
    ];

    for prefix in QUESTION_PREFIXES {
        if lower.starts_with(prefix) {
            let rest = &query[prefix.len()..];
            let rest = rest.trim_end_matches('?').trim();
            if !rest.is_empty() && rest.split_whitespace().count() >= 1 {
                return rest.to_string();
            }
        }
    }

    // Remove trailing question mark
    if query.ends_with('?') {
        return query[..query.len() - 1].trim().to_string();
    }

    query.to_string()
}

/// Remove conversational filler words that hurt search quality.
fn remove_fillers(query: &str) -> String {
    const FILLER_PHRASES: &[&str] = &[
        "please help me ",
        "help me ",
        "i want to ",
        "i need to ",
        "i would like to ",
        "i'm trying to ",
        "i am trying to ",
        "i'm looking for ",
        "i am looking for ",
        "can anyone ",
        "does anyone know ",
        "i was wondering ",
    ];

    let lower = query.to_lowercase();
    let mut result = query.to_string();

    for phrase in FILLER_PHRASES {
        if lower.starts_with(phrase) {
            result = query[phrase.len()..].to_string();
            break;
        }
    }

    // Remove individual filler words that don't carry search value
    const FILLER_WORDS: &[&str] = &[
        "basically", "actually", "really", "just", "simply", "literally",
    ];
    let words: Vec<&str> = result.split_whitespace().collect();
    if words.len() > 2 {
        let filtered: Vec<&str> = words
            .into_iter()
            .filter(|w| {
                let wl = w.to_lowercase();
                !FILLER_WORDS.contains(&wl.as_str())
            })
            .collect();
        if !filtered.is_empty() {
            result = filtered.join(" ");
        }
    }

    result
}

/// Expand abbreviations inline for terms that search engines don't handle well.
///
/// Only expands niche abbreviations. Well-known terms like SQL, HTTP, CSS, API,
/// REST, DNS, etc. are understood by all search engines and should NOT be expanded
/// (expansion dilutes the query).
fn expand_abbreviations(query: &str) -> String {
    // Only abbreviations that benefit from expansion — niche terms search engines
    // may not associate well with their full forms.
    const ACRONYMS: &[(&str, &str)] = &[
        ("ml", "machine learning"),
        ("dl", "deep learning"),
        ("nlp", "natural language processing"),
        ("cv", "computer vision"),
        ("llm", "large language model"),
        ("rl", "reinforcement learning"),
        ("nn", "neural network"),
        ("cnn", "convolutional neural network"),
        ("rnn", "recurrent neural network"),
        ("k8s", "kubernetes"),
        ("ux", "user experience"),
        ("iot", "internet of things"),
        ("saas", "software as a service"),
    ];

    let words: Vec<&str> = query.split_whitespace().collect();
    let mut result = Vec::with_capacity(words.len());
    let mut expanded = false;

    for word in &words {
        let clean = word
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase();
        let mut found = false;
        for (abbr, full) in ACRONYMS {
            if clean == *abbr {
                // Only expand if the word is standalone (not part of a longer term)
                // and the full form isn't already in the query
                let query_lower = query.to_lowercase();
                if !query_lower.contains(full) {
                    result.push(full.to_string());
                    found = true;
                    expanded = true;
                    break;
                }
            }
        }
        if !found {
            result.push(word.to_string());
        }
    }

    if expanded {
        result.join(" ")
    } else {
        query.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn question_normalization() {
        let r = autoprompt("what is a python decorator");
        assert_eq!(r.rewritten, "python decorator");
        assert!(r.changed);
    }

    #[test]
    fn question_how_to() {
        let r = autoprompt("how to deploy ML models to production");
        assert_eq!(r.rewritten, "deploy machine learning models to production");
        assert!(r.changed);
    }

    #[test]
    fn filler_removal() {
        let r = autoprompt("please help me fix this JS error");
        assert!(r.rewritten.contains("JS"));
        assert!(!r.rewritten.contains("please"));
        assert!(!r.rewritten.contains("help me"));
    }

    #[test]
    fn abbreviation_expansion() {
        let r = autoprompt("ML model training best practices");
        assert!(r.rewritten.contains("machine learning"));
        assert!(r.changed);
    }

    #[test]
    fn no_change_needed() {
        let r = autoprompt("rust async runtime comparison");
        assert_eq!(r.rewritten, "rust async runtime comparison");
        assert!(!r.changed);
    }

    #[test]
    fn multilingual_passthrough() {
        let r = autoprompt("最新の生成aiニュース 2026");
        assert_eq!(r.rewritten, "最新の生成aiニュース 2026");
        assert!(!r.changed);
    }

    #[test]
    fn empty_query() {
        let r = autoprompt("");
        assert_eq!(r.rewritten, "");
        assert!(!r.changed);
    }

    #[test]
    fn trailing_question_mark() {
        let r = autoprompt("python decorator syntax?");
        assert_eq!(r.rewritten, "python decorator syntax");
        assert!(r.changed);
    }

    #[test]
    fn combined_transforms() {
        let r = autoprompt("what is NLP and how does it work?");
        // Should normalize question + expand NLP
        assert!(r.rewritten.contains("natural language processing"));
        assert!(!r.rewritten.contains("what is"));
    }

    #[test]
    fn filler_words_removed() {
        let r = autoprompt("I'm trying to basically understand kubernetes");
        assert!(r.rewritten.contains("kubernetes"));
        assert!(!r.rewritten.contains("basically"));
    }

    #[test]
    fn k8s_expansion() {
        let r = autoprompt("k8s pod networking");
        assert!(r.rewritten.contains("kubernetes"));
    }

    #[test]
    fn doesnt_double_expand() {
        // If the full form is already in the query, don't expand
        let r = autoprompt("machine learning ML models");
        // "ML" shouldn't expand since "machine learning" is already present
        assert_eq!(r.rewritten, "machine learning ML models");
    }
}
