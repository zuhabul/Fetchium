//! Answer Extraction Engine (AXE) — extracts direct answers from search results.
//!
//! For factual queries like "What is the capital of France?", this engine
//! extracts candidate answers from result snippets using pattern matching
//! and cross-result voting.

use crate::types::ResultItem;
use std::collections::HashMap;

/// Configuration for the answer extractor.
#[derive(Debug, Clone)]
pub struct AnswerConfig {
    /// Minimum number of results that must agree on an answer.
    pub min_agreement: usize,
    /// Confidence threshold below which answers are not returned.
    pub min_confidence: f64,
    /// Maximum number of candidate answers to return.
    pub max_answers: usize,
}

impl Default for AnswerConfig {
    fn default() -> Self {
        Self {
            min_agreement: 1,
            min_confidence: 0.3,
            max_answers: 3,
        }
    }
}

/// A candidate answer extracted from results.
#[derive(Debug, Clone)]
pub struct ExtractedAnswer {
    /// The answer text.
    pub answer: String,
    /// Confidence score (0.0–1.0).
    pub confidence: f64,
    /// Number of results that support this answer.
    pub supporting_sources: usize,
    /// Extraction method used.
    pub method: ExtractionMethod,
}

/// How the answer was extracted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractionMethod {
    /// "X is Y" pattern matching.
    DefinitionPattern,
    /// Cross-result entity voting.
    EntityVoting,
    /// Direct snippet extraction.
    SnippetExtraction,
}

/// Extract direct answers from search results for a given query.
pub fn extract_answers(
    query: &str,
    results: &[ResultItem],
    config: &AnswerConfig,
) -> Vec<ExtractedAnswer> {
    if results.is_empty() || query.is_empty() {
        return vec![];
    }

    let mut candidates: Vec<ExtractedAnswer> = Vec::new();

    // Method 1: Definition patterns ("X is Y", "X are Y")
    let definitions = extract_definitions(query, results);
    candidates.extend(definitions);

    // Method 2: Entity voting (common entities across results)
    let entities = entity_voting(query, results);
    candidates.extend(entities);

    // Filter by confidence and agreement
    candidates.retain(|a| {
        a.confidence >= config.min_confidence && a.supporting_sources >= config.min_agreement
    });

    // Sort by confidence descending
    candidates.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Deduplicate similar answers
    deduplicate_answers(&mut candidates);

    candidates.truncate(config.max_answers);
    candidates
}

/// Check if a query is answerable (factual question).
pub fn is_answerable_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();

    // Check for question patterns
    if let Some(first) = words.first() {
        if QUESTION_WORDS.contains(first) {
            return true;
        }
    }

    // Check for "define X", "meaning of X" patterns
    lower.starts_with("define ")
        || lower.contains("meaning of")
        || lower.contains("definition of")
        || lower.ends_with('?')
}

// ─── Definition pattern extraction ────────────────────────

fn extract_definitions(query: &str, results: &[ResultItem]) -> Vec<ExtractedAnswer> {
    let query_lower = query.to_lowercase();
    let query_subject = extract_subject(&query_lower);
    let mut answers = Vec::new();

    for result in results {
        let snippet_lower = result.snippet.to_lowercase();
        let sentences = split_sentences(&snippet_lower);

        for sentence in &sentences {
            // Look for "Subject is/are Definition" patterns
            if let Some(definition) = extract_is_pattern(sentence, &query_subject) {
                let clean = clean_answer(&definition);
                if !clean.is_empty() && clean.len() < 200 {
                    answers.push(ExtractedAnswer {
                        answer: clean,
                        confidence: 0.7,
                        supporting_sources: 1,
                        method: ExtractionMethod::DefinitionPattern,
                    });
                }
            }
        }
    }

    // Merge identical definitions
    merge_answers(&mut answers);
    answers
}

fn extract_subject(query: &str) -> String {
    let mut subject = query.to_string();

    // Strip question words
    for prefix in &[
        "what is ",
        "what are ",
        "who is ",
        "who are ",
        "where is ",
        "when is ",
        "define ",
    ] {
        if subject.starts_with(prefix) {
            subject = subject[prefix.len()..].to_string();
            break;
        }
    }

    // Strip trailing "?"
    if subject.ends_with('?') {
        subject.pop();
    }

    subject.trim().to_string()
}

fn extract_is_pattern(sentence: &str, subject: &str) -> Option<String> {
    if subject.is_empty() {
        return None;
    }

    // Look for "subject is/are definition"
    for connector in &[" is ", " are ", " was ", " refers to ", " means "] {
        if let Some(pos) = sentence.find(connector) {
            let before = &sentence[..pos];
            // Check if subject appears before the connector
            if before.contains(subject) || subject.contains(before.trim()) {
                let after = &sentence[pos + connector.len()..];
                let definition = after.trim().to_string();
                if !definition.is_empty() {
                    return Some(definition);
                }
            }
        }
    }
    None
}

// ─── Entity voting ────────────────────────────────────────

fn entity_voting(query: &str, results: &[ResultItem]) -> Vec<ExtractedAnswer> {
    let query_terms: Vec<String> = query
        .to_lowercase()
        .split_whitespace()
        .filter(|w| w.len() >= 3)
        .map(|w| w.to_string())
        .collect();

    // Extract candidate entities from snippets (capitalized phrases, numbers, etc.)
    let mut entity_counts: HashMap<String, usize> = HashMap::new();

    for result in results {
        let entities = extract_entities_from_snippet(&result.snippet, &query_terms);
        for entity in entities {
            *entity_counts.entry(entity).or_insert(0) += 1;
        }
    }

    // Convert to answers, requiring multiple sources
    let total = results.len().max(1);
    entity_counts
        .into_iter()
        .filter(|(_, count)| *count >= 1)
        .map(|(entity, count)| {
            let confidence = (count as f64 / total as f64).min(0.95);
            ExtractedAnswer {
                answer: entity,
                confidence,
                supporting_sources: count,
                method: ExtractionMethod::EntityVoting,
            }
        })
        .collect()
}

fn extract_entities_from_snippet(snippet: &str, query_terms: &[String]) -> Vec<String> {
    let mut entities = Vec::new();

    // Extract numbers with units (e.g., "384,400 km", "100°C")
    let words: Vec<&str> = snippet.split_whitespace().collect();
    for window in words.windows(2) {
        let first = window[0];
        let second = window[1];
        if first.chars().any(|c| c.is_ascii_digit()) && second.len() <= 10 {
            let candidate = format!("{} {}", first, second);
            if !has_overlap(&candidate.to_lowercase(), query_terms) {
                entities.push(candidate);
            }
        }
    }

    // Extract capitalized phrases (potential named entities)
    let mut current_phrase = Vec::new();
    for word in &words {
        if word.starts_with(|c: char| c.is_uppercase()) && word.len() >= 2 {
            current_phrase.push(*word);
        } else {
            if !current_phrase.is_empty() {
                let phrase = current_phrase.join(" ");
                if phrase.len() >= 3 && !has_overlap(&phrase.to_lowercase(), query_terms) {
                    entities.push(phrase);
                }
            }
            current_phrase.clear();
        }
    }
    if !current_phrase.is_empty() {
        let phrase = current_phrase.join(" ");
        if phrase.len() >= 3 && !has_overlap(&phrase.to_lowercase(), query_terms) {
            entities.push(phrase);
        }
    }

    entities
}

fn has_overlap(text: &str, terms: &[String]) -> bool {
    terms.iter().any(|t| text.contains(t.as_str()))
}

// ─── Helpers ─────────────────────────────────────────────

fn split_sentences(text: &str) -> Vec<String> {
    text.split(['.', '!', '?'])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn clean_answer(answer: &str) -> String {
    let trimmed = answer.trim();
    // Strip trailing punctuation
    let stripped = trimmed.trim_end_matches(['.', ',', ';']);
    stripped.to_string()
}

fn merge_answers(answers: &mut Vec<ExtractedAnswer>) {
    let mut merged: Vec<ExtractedAnswer> = Vec::new();

    for answer in answers.drain(..) {
        let lower = answer.answer.to_lowercase();
        if let Some(existing) = merged.iter_mut().find(|a| a.answer.to_lowercase() == lower) {
            existing.supporting_sources += answer.supporting_sources;
            existing.confidence = (existing.confidence + answer.confidence) / 2.0;
        } else {
            merged.push(answer);
        }
    }

    *answers = merged;
}

fn deduplicate_answers(answers: &mut Vec<ExtractedAnswer>) {
    let mut seen: Vec<String> = Vec::new();
    answers.retain(|a| {
        let lower = a.answer.to_lowercase();
        if seen
            .iter()
            .any(|s| s == &lower || s.contains(&lower) || lower.contains(s.as_str()))
        {
            false
        } else {
            seen.push(lower);
            true
        }
    });
}

const QUESTION_WORDS: &[&str] = &["what", "who", "where", "when", "which", "how", "define"];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BackendId;

    fn make_item(title: &str, snippet: &str) -> ResultItem {
        ResultItem {
            title: title.into(),
            url: format!(
                "https://{}.example.com",
                title.to_lowercase().replace(' ', "-")
            ),
            snippet: snippet.into(),
            rank: 0,
            backend: BackendId::DuckDuckGo,
            score: Some(0.5),
            published_date: None,
        }
    }

    #[test]
    fn empty_input_no_answers() {
        let answers = extract_answers("", &[], &AnswerConfig::default());
        assert!(answers.is_empty());
    }

    #[test]
    fn definition_pattern_extraction() {
        let results = vec![
            make_item(
                "Rust",
                "Rust is a systems programming language focused on safety and performance.",
            ),
            make_item(
                "About Rust",
                "Rust is a multi-paradigm systems programming language.",
            ),
        ];
        let answers = extract_answers("What is Rust?", &results, &AnswerConfig::default());
        assert!(
            !answers.is_empty(),
            "should extract definition for 'what is' query"
        );
    }

    #[test]
    fn answerable_query_detection() {
        assert!(is_answerable_query("What is the capital of France?"));
        assert!(is_answerable_query("Who invented the telephone?"));
        assert!(is_answerable_query("define algorithm"));
        assert!(!is_answerable_query("rust programming"));
    }

    #[test]
    fn subject_extraction() {
        assert_eq!(extract_subject("what is rust"), "rust");
        assert_eq!(extract_subject("who is alan turing"), "alan turing");
        assert_eq!(extract_subject("define algorithm"), "algorithm");
    }

    #[test]
    fn entity_voting_finds_common_entities() {
        let results = vec![
            make_item(
                "France Capital",
                "The capital of France is Paris, a major European city.",
            ),
            make_item(
                "Paris Info",
                "Paris is the capital and largest city of France.",
            ),
            make_item(
                "French Cities",
                "Paris serves as the capital of France since centuries.",
            ),
        ];
        let answers = extract_answers("capital of France", &results, &AnswerConfig::default());

        let has_paris = answers.iter().any(|a| {
            let lower = a.answer.to_lowercase();
            lower.contains("paris")
        });
        assert!(
            has_paris,
            "Paris should be extracted as answer, got: {:?}",
            answers
        );
    }

    #[test]
    fn confidence_sorted_descending() {
        let results = vec![
            make_item("A", "Rust is a systems programming language for safety."),
            make_item("B", "Rust is a compiled language for performance."),
        ];
        let answers = extract_answers("What is Rust?", &results, &AnswerConfig::default());
        if answers.len() >= 2 {
            for i in 0..answers.len() - 1 {
                assert!(answers[i].confidence >= answers[i + 1].confidence);
            }
        }
    }

    #[test]
    fn max_answers_respected() {
        let config = AnswerConfig {
            max_answers: 1,
            min_confidence: 0.0,
            ..AnswerConfig::default()
        };
        let results = vec![make_item(
            "A",
            "Rust is safe. Rust is fast. Rust is concurrent.",
        )];
        let answers = extract_answers("What is Rust?", &results, &config);
        assert!(answers.len() <= 1);
    }

    #[test]
    fn is_pattern_extraction() {
        let def = extract_is_pattern("rust is a systems programming language", "rust");
        assert!(def.is_some());
        assert!(def.unwrap().contains("systems programming language"));
    }

    #[test]
    fn clean_answer_strips_punctuation() {
        assert_eq!(clean_answer("  hello world.  "), "hello world");
        assert_eq!(clean_answer("answer,"), "answer");
    }
}
