//! Smart Snippet Extraction (SSE) — query-aware passage extraction.
//!
//! Extracts the most relevant passages from a document for display as
//! search result snippets. Uses sliding-window BM25 scoring with term
//! proximity bonuses and positional decay.

use std::collections::HashSet;

#[allow(unused_imports)]
use crate::types::ResultItem;

/// Configuration for the snippet extraction engine.
#[derive(Debug, Clone)]
pub struct SnippetConfig {
    /// Number of words in each sliding window.
    pub window_size: usize,
    /// Maximum number of snippets to return.
    pub max_snippets: usize,
    /// Prefix inserted before highlighted query terms.
    pub highlight_prefix: &'static str,
    /// Suffix inserted after highlighted query terms.
    pub highlight_suffix: &'static str,
}

impl Default for SnippetConfig {
    fn default() -> Self {
        Self {
            window_size: 30,
            max_snippets: 3,
            highlight_prefix: "**",
            highlight_suffix: "**",
        }
    }
}

/// A passage extracted from the source text with its relevance score.
#[derive(Debug, Clone)]
pub struct ScoredSnippet {
    /// The extracted passage text.
    pub text: String,
    /// Composite relevance score (higher is better).
    pub score: f64,
    /// Starting word index in the original document.
    pub start_word: usize,
    /// Ending word index (exclusive) in the original document.
    pub end_word: usize,
}

/// Extract the most relevant snippets from `text` for the given `query`.
///
/// Returns up to `config.max_snippets` non-overlapping passages sorted
/// by descending relevance score.
pub fn extract_best_snippets(
    text: &str,
    query: &str,
    config: &SnippetConfig,
) -> Vec<ScoredSnippet> {
    if text.is_empty() || query.is_empty() || config.window_size == 0 {
        return Vec::new();
    }

    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return Vec::new();
    }

    let query_terms: HashSet<&str> = query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 2)
        .collect();
    if query_terms.is_empty() {
        return Vec::new();
    }

    let total = words.len();
    let win = config.window_size.min(total);
    let num_windows = total.saturating_sub(win) + 1;
    let mut snippets: Vec<ScoredSnippet> = Vec::new();

    for start in 0..num_windows {
        let end = (start + win).min(total);
        let ww = &words[start..end];
        let raw = score_window(ww, &query_terms);
        let pos_factor = 1.0 + 0.1 * (1.0 - start as f64 / num_windows.max(1) as f64);
        let score = raw * pos_factor;

        if score > 0.0 {
            snippets.push(ScoredSnippet {
                text: ww.join(" "),
                score,
                start_word: start,
                end_word: end,
            });
        }
    }

    snippets.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    merge_overlapping(&mut snippets);
    snippets.truncate(config.max_snippets);
    snippets
}

/// Highlight query terms in a snippet by wrapping them with prefix/suffix markers.
///
/// Matching is case-insensitive. Punctuation attached to words is preserved.
pub fn highlight_terms(snippet: &str, query: &str, prefix: &str, suffix: &str) -> String {
    let query_terms: HashSet<String> = query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 2)
        .map(|t| t.to_lowercase())
        .collect();
    if query_terms.is_empty() {
        return snippet.to_string();
    }
    snippet
        .split_whitespace()
        .map(|word| {
            let clean: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
            if query_terms.contains(&clean.to_lowercase()) {
                format!("{prefix}{word}{suffix}")
            } else {
                word.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Score a word window: term frequency, density, and proximity.
fn score_window(words: &[&str], query_terms: &HashSet<&str>) -> f64 {
    if words.is_empty() || query_terms.is_empty() {
        return 0.0;
    }

    let clean: Vec<String> = words
        .iter()
        .map(|w| {
            w.to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect()
        })
        .collect();

    // Distinct query terms present in this window
    let tf = query_terms
        .iter()
        .filter(|qt| clean.contains(&qt.to_lowercase()))
        .count() as f64;
    if tf == 0.0 {
        return 0.0;
    }

    // Density: total query-term hits / window size
    let hits = clean
        .iter()
        .filter(|w| query_terms.iter().any(|qt| qt.to_lowercase() == **w))
        .count() as f64;
    let density = hits / words.len() as f64;

    // Proximity: bonus when matched terms appear close together
    let positions: Vec<usize> = clean
        .iter()
        .enumerate()
        .filter(|(_, w)| query_terms.iter().any(|qt| qt.to_lowercase() == **w))
        .map(|(i, _)| i)
        .collect();

    let proximity = if positions.len() >= 2 {
        let gap: usize = positions.windows(2).map(|p| p[1] - p[0]).sum();
        let avg = gap as f64 / (positions.len() - 1) as f64;
        (1.0 / avg).min(1.0)
    } else {
        0.0
    };

    let coverage = tf / query_terms.len() as f64;
    coverage * 2.0 + density * 1.5 + proximity
}

/// Remove overlapping snippets, keeping higher-scored ones.
fn merge_overlapping(snippets: &mut Vec<ScoredSnippet>) {
    let mut accepted: Vec<ScoredSnippet> = Vec::new();
    for s in snippets.drain(..) {
        let overlaps = accepted
            .iter()
            .any(|e| s.start_word < e.end_word && s.end_word > e.start_word);
        if !overlaps {
            accepted.push(s);
        }
    }
    *snippets = accepted;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(window: usize, max: usize) -> SnippetConfig {
        SnippetConfig {
            window_size: window,
            max_snippets: max,
            ..Default::default()
        }
    }

    #[test]
    fn empty_text_returns_empty() {
        assert!(extract_best_snippets("", "rust", &Default::default()).is_empty());
    }

    #[test]
    fn empty_query_returns_empty() {
        assert!(extract_best_snippets("Some text.", "", &Default::default()).is_empty());
    }

    #[test]
    fn single_match_returns_correct_window() {
        let text = "The cat sat on the mat and the Rust language is fast and safe for systems";
        let snips = extract_best_snippets(text, "Rust language", &cfg(5, 1));
        assert!(!snips.is_empty());
        assert!(snips[0].text.to_lowercase().contains("rust"));
        assert!(snips[0].score > 0.0);
    }

    #[test]
    fn multiple_matches_returns_ranked_windows() {
        let text = "Rust is a systems programming language. \
                    Unrelated filler text about cooking recipes and pasta. \
                    More about Rust safety and memory management in systems.";
        let snips = extract_best_snippets(text, "Rust systems", &cfg(6, 3));
        assert!(snips.len() >= 2, "found {}", snips.len());
        for pair in snips.windows(2) {
            assert!(pair[0].score >= pair[1].score);
        }
    }

    #[test]
    fn query_term_density_affects_score() {
        let text = "Rust Rust programming Rust language systems Rust code \
                    filler words here that mean nothing at all \
                    one mention of Rust in a sea of unrelated words everywhere";
        let snips = extract_best_snippets(text, "Rust", &cfg(8, 2));
        assert!(snips.len() >= 2, "found {}", snips.len());
        assert!(
            snips[0].score > snips[1].score,
            "{} > {}",
            snips[0].score,
            snips[1].score
        );
    }

    #[test]
    fn overlapping_windows_merged() {
        let text = "Rust is a programming language for systems that is fast and safe";
        let snips = extract_best_snippets(text, "Rust programming systems", &cfg(10, 5));
        for i in 0..snips.len() {
            for j in (i + 1)..snips.len() {
                let (a, b) = (&snips[i], &snips[j]);
                assert!(
                    !(a.start_word < b.end_word && a.end_word > b.start_word),
                    "snippets {i} and {j} overlap"
                );
            }
        }
    }

    #[test]
    fn highlight_wraps_correct_terms() {
        let h = highlight_terms(
            "Rust is a fast systems language for programming",
            "Rust programming",
            "<<",
            ">>",
        );
        assert!(h.contains("<<Rust>>"), "{h}");
        assert!(h.contains("<<programming>>"), "{h}");
        assert!(!h.contains("<<is>>"), "{h}");
        assert!(!h.contains("<<fast>>"), "{h}");
    }

    #[test]
    fn position_bonus_favors_early_results() {
        let text = "Rust systems programming is great \
                    filler filler filler filler filler filler filler filler filler filler \
                    filler filler filler filler filler filler filler filler filler filler \
                    Rust systems programming is great";
        let snips = extract_best_snippets(text, "Rust systems programming", &cfg(5, 2));
        assert!(snips.len() >= 2, "found {}", snips.len());
        assert!(
            snips[0].start_word < snips[1].start_word,
            "early={} late={}",
            snips[0].start_word,
            snips[1].start_word
        );
    }

    #[test]
    fn max_snippets_respected() {
        let text = "Rust systems are great. Also Rust programming is fun. And Rust memory safety is important.";
        let snips = extract_best_snippets(text, "Rust", &cfg(5, 1));
        assert!(snips.len() <= 1, "got {}", snips.len());
    }

    #[test]
    fn highlight_case_insensitive() {
        let h = highlight_terms(
            "rust is GREAT for SYSTEMS programming",
            "Rust systems",
            "[",
            "]",
        );
        assert!(h.contains("[rust]"), "{h}");
        assert!(h.contains("[SYSTEMS]"), "{h}");
    }

    #[test]
    fn score_window_empty_inputs() {
        let empty: HashSet<&str> = HashSet::new();
        assert_eq!(score_window(&[], &empty), 0.0);
        let terms: HashSet<&str> = ["rust"].into_iter().collect();
        assert_eq!(score_window(&[], &terms), 0.0);
        assert_eq!(score_window(&["hello", "world"], &empty), 0.0);
    }
}
