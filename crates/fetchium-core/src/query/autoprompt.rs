//! Autoprompt — Exa-style local query rewriting for better backend results.
//!
//! Transforms raw user queries into optimized search queries without API calls.
//! Applied before dispatch to backends in the search orchestrator.

use crate::query::crosslingual::{expand_crosslingual, Language};

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

    let crosslingual = expand_crosslingual(&q);
    if is_non_latin_bridge_query(&crosslingual) {
        let rewritten = crosslingual
            .expansions
            .first()
            .map(|expansion| expansion.query.trim().to_string())
            .unwrap_or_else(|| q.clone());
        return AutopromptResult {
            changed: rewritten != q,
            rewritten,
            original,
        };
    }

    if is_latin_foreign_query(&crosslingual) {
        let rewritten = bridge_crosslingual_query(&q, &crosslingual);
        return AutopromptResult {
            changed: rewritten != q,
            rewritten,
            original,
        };
    }

    // Skip rewriting for non-ASCII non-Latin queries — our patterns are English-only
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

    // Step 4: Normalize health-sensitive comparison/treatment queries into facets.
    q = normalize_medical_query(&q);

    // Step 5: Condense long natural-language queries into lexical anchors.
    // Skip health-sensitive rewrites so medical facets survive into retrieval.
    if !is_health_sensitive_query(&q) {
        q = condense_long_query(&q);
    }

    // Step 6: Length guard
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
        if let Some(stripped) = query.strip_suffix('?') {
            return stripped.trim().to_string();
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
    if let Some(stripped) = query.strip_suffix('?') {
        return stripped.trim().to_string();
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
        "basically",
        "actually",
        "really",
        "just",
        "simply",
        "literally",
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

fn condense_long_query(query: &str) -> String {
    let words: Vec<&str> = query.split_whitespace().collect();
    if words.len() <= 12 {
        return query.to_string();
    }
    let lower = query.to_lowercase();
    let preserve_howto =
        lower.contains("how to") || lower.contains("guide") || lower.contains("step by step");

    const STOPWORDS: &[&str] = &[
        "a", "an", "the", "and", "or", "but", "to", "for", "of", "with", "by", "in", "on", "at",
        "is", "are", "was", "were", "be", "been", "being", "this", "that", "these", "those",
        "what", "which", "who", "when", "where", "why", "how", "using", "into", "from", "step",
        "steps", "guide", "best",
    ];

    let mut anchors = Vec::new();
    for word in words {
        let clean = word
            .trim_matches(|c: char| !c.is_alphanumeric() && c != '/')
            .to_lowercase();
        if clean.len() < 2 || STOPWORDS.contains(&clean.as_str()) {
            continue;
        }
        if !anchors.contains(&clean) {
            anchors.push(clean);
        }
    }

    if anchors.len() <= 10 {
        let condensed = anchors.join(" ");
        return if preserve_howto {
            format!("how to {condensed}")
        } else {
            condensed
        };
    }

    let tail_start = anchors.len().saturating_sub(4);
    let mut reduced = Vec::with_capacity(10);
    for word in anchors
        .iter()
        .take(6)
        .chain(anchors.iter().skip(tail_start))
    {
        if !reduced.contains(word) {
            reduced.push(word.clone());
        }
    }
    let condensed = reduced.join(" ");
    if preserve_howto {
        format!("how to {condensed}")
    } else {
        condensed
    }
}

fn normalize_medical_query(query: &str) -> String {
    if !is_health_sensitive_query(query) {
        return expand_medical_phrases(query);
    }

    let lower = query.to_lowercase();
    let mut facets = Vec::new();

    if lower.contains("blood pressure") || lower.contains("hypertension") {
        facets.push("hypertension".to_string());
    }

    if lower.contains("sleep apnea") {
        facets.push("obstructive sleep apnea".to_string());
    }

    if lower.contains("medication")
        || lower.contains("medicine")
        || lower.contains("drug")
        || lower.contains("blood pressure medication")
    {
        facets.push("antihypertensive".to_string());
        facets.push("drug classes".to_string());
    }

    if lower.contains("comparison")
        || lower.contains("compare")
        || lower.contains("vs")
        || lower.contains("versus")
    {
        facets.push("comparison".to_string());
    }

    if lower.contains("side effects") || lower.contains("adverse effects") {
        facets.push("adverse effects".to_string());
    }

    if lower.contains("blood pressure medication")
        && (lower.contains("comparison") || lower.contains("side effects"))
    {
        facets.push("ace inhibitor".to_string());
        facets.push("arb".to_string());
        facets.push("beta blocker".to_string());
        facets.push("diuretic".to_string());
        facets.push("calcium channel blocker".to_string());
    }

    if facets.is_empty() {
        return expand_medical_phrases(query);
    }

    if is_strict_medical_query(&lower) {
        return facets.join(" ");
    }

    let mut parts = vec![query.to_string()];
    for facet in facets {
        if !contains_phrase_ci(&lower, &facet) {
            parts.push(facet);
        }
    }

    parts.join(" ")
}

fn expand_medical_phrases(query: &str) -> String {
    let lower = query.to_lowercase();
    let mut result = query.to_string();

    if lower.contains("blood pressure medication") && !lower.contains("antihypertensive") {
        result.push_str(" antihypertensive");
    }
    if lower.contains("blood pressure") && !lower.contains("hypertension") {
        result.push_str(" hypertension");
    }
    if (lower.contains("medication comparison") || lower.contains("drug comparison"))
        && !lower.contains("drug classes")
    {
        result.push_str(" drug classes");
    }
    if lower.contains("side effects") && !lower.contains("adverse effects") {
        result.push_str(" adverse effects");
    }
    if lower.contains("blood pressure medication")
        && lower.contains("comparison")
        && !lower.contains("ace inhibitor")
    {
        result.push_str(" ace inhibitor arb beta blocker diuretic calcium channel blocker");
    }

    result
}

fn is_health_sensitive_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    [
        "medication",
        "medicine",
        "drug",
        "drugs",
        "dose",
        "dosage",
        "side effect",
        "side effects",
        "symptom",
        "symptoms",
        "treatment",
        "treatments",
        "diagnosis",
        "disease",
        "blood pressure",
        "hypertension",
        "diabetes",
        "vaccine",
        "vaccination",
        "therapy",
        "supplement",
        "supplements",
        "supplementation",
        "creatine",
        "pain",
        "cancer",
    ]
    .iter()
    .any(|pattern| lower.contains(pattern))
}

fn is_strict_medical_query(lower_query: &str) -> bool {
    [
        "medication",
        "medicine",
        "drug",
        "drugs",
        "dose",
        "dosage",
        "side effect",
        "side effects",
        "treatment",
        "treatments",
        "therapy",
        "compare",
        "comparison",
        "versus",
        "vs",
    ]
    .iter()
    .any(|pattern| lower_query.contains(pattern))
}

fn contains_phrase_ci(lower_query: &str, phrase: &str) -> bool {
    lower_query.contains(&phrase.to_lowercase())
}

fn is_latin_foreign_query(result: &crate::query::crosslingual::CrossLingualResult) -> bool {
    matches!(
        result.detected_language,
        Language::French | Language::Spanish | Language::German | Language::Portuguese
    ) && !result.expansions.is_empty()
}

fn is_non_latin_bridge_query(result: &crate::query::crosslingual::CrossLingualResult) -> bool {
    matches!(result.detected_language, Language::Bengali) && !result.expansions.is_empty()
}

fn bridge_crosslingual_query(
    query: &str,
    result: &crate::query::crosslingual::CrossLingualResult,
) -> String {
    let Some(expansion) = result.expansions.first() else {
        return query.to_string();
    };
    let query_lower = query.to_lowercase();
    let expansion_lower = expansion.query.to_lowercase();
    if expansion_lower == query_lower || query_lower.contains(&expansion_lower) {
        return query.to_string();
    }
    format!("{query} {}", expansion.query.trim())
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

    #[test]
    fn condenses_long_technical_query() {
        let r = autoprompt(
            "step by step guide to deploying a machine learning model to production using Docker Kubernetes and CI CD pipelines",
        );
        assert!(r.rewritten.starts_with("how to "));
        assert!(r.rewritten.contains("machine learning"));
        assert!(r.rewritten.contains("docker"));
        assert!(r.rewritten.contains("kubernetes"));
        assert!(r.rewritten.contains("ci"));
        assert!(r.rewritten.contains("cd"));
        assert!(!r.rewritten.contains("step by step"));
    }

    #[test]
    fn expands_medical_phrases() {
        let r = autoprompt("blood pressure medication side effects comparison");
        assert!(r.rewritten.contains("antihypertensive"));
        assert!(r.rewritten.contains("hypertension"));
        assert!(r.rewritten.contains("drug classes"));
        assert!(r.rewritten.contains("adverse effects"));
        assert!(r.rewritten.contains("ace inhibitor"));
        assert!(r.rewritten.contains("beta blocker"));
    }

    #[test]
    fn normalize_medical_query_builds_facets_for_comparison() {
        let normalized =
            normalize_medical_query("blood pressure medication side effects comparison");
        assert!(!normalized.contains("blood pressure medication"));
        assert!(normalized.contains("hypertension"));
        assert!(normalized.contains("antihypertensive"));
        assert!(normalized.contains("drug classes"));
        assert!(normalized.contains("adverse effects"));
        assert!(normalized.contains("calcium channel blocker"));
    }

    #[test]
    fn normalize_medical_query_leaves_non_health_queries_alone() {
        let normalized = normalize_medical_query("vector database comparison pinecone vs weaviate");
        assert_eq!(
            normalized,
            "vector database comparison pinecone vs weaviate"
        );
    }

    #[test]
    fn bridge_crosslingual_query_adds_english_anchor_terms() {
        let r = autoprompt("comment apprendre le français rapidement");
        assert!(
            r.rewritten.contains("learn french fast"),
            "rewritten={}",
            r.rewritten
        );
    }

    #[test]
    fn normalize_medical_query_adds_sleep_apnea_facet() {
        let normalized = normalize_medical_query("sleep apnea causes diagnosis treatment options");
        assert!(normalized.contains("obstructive sleep apnea"));
    }

    #[test]
    fn bengali_query_rewrites_to_english_bridge() {
        let r = autoprompt("মেশিন লার্নিং কি");
        assert_eq!(r.rewritten, "what is machine learning");
    }
}
