//! Cross-Lingual Query Bridge (CLQB) — detects language and adds cross-lingual expansions.
//!
//! Automatically detects when a query contains non-English technical terms
//! and adds English equivalents (and vice versa). Supports common programming
//! and technical terms across 6 languages.
//!
//! Example: "algorithme de tri" → also searches "sorting algorithm"

use std::collections::HashMap;

/// A cross-lingual expansion of a query.
#[derive(Debug, Clone)]
pub struct CrossLingualExpansion {
    /// The expanded query text.
    pub query: String,
    /// Detected source language.
    pub source_lang: Language,
    /// Target language of this expansion.
    pub target_lang: Language,
    /// Confidence in the translation (0.0–1.0).
    pub confidence: f64,
}

/// Supported languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    Spanish,
    French,
    German,
    Portuguese,
    Japanese,
    Unknown,
}

/// Result of cross-lingual analysis.
#[derive(Debug, Clone)]
pub struct CrossLingualResult {
    /// Original query.
    pub original: String,
    /// Detected language of the original query.
    pub detected_language: Language,
    /// Cross-lingual expansions (empty if already English with no foreign terms).
    pub expansions: Vec<CrossLingualExpansion>,
}

/// Analyze a query and generate cross-lingual expansions.
pub fn expand_crosslingual(query: &str) -> CrossLingualResult {
    let detected = detect_language(query);
    let mut expansions = Vec::new();

    if detected != Language::English && detected != Language::Unknown {
        // Translate foreign query to English
        if let Some(english) = translate_to_english(query, detected) {
            expansions.push(CrossLingualExpansion {
                query: english,
                source_lang: detected,
                target_lang: Language::English,
                confidence: 0.8,
            });
        }
    }

    // Also check for individual foreign technical terms in otherwise English queries
    let term_translations = translate_terms(query);
    for (original_term, english_term, lang, confidence) in term_translations {
        let expanded = query.replace(&original_term, &english_term);
        if expanded != query && !expansions.iter().any(|e| e.query == expanded) {
            expansions.push(CrossLingualExpansion {
                query: expanded,
                source_lang: lang,
                target_lang: Language::English,
                confidence,
            });
        }
    }

    // Sort by confidence descending, limit to 3
    expansions.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    expansions.truncate(3);

    CrossLingualResult {
        original: query.to_string(),
        detected_language: detected,
        expansions,
    }
}

// ─── Language detection ──────────────────────────────────────

fn detect_language(query: &str) -> Language {
    let lower = query.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();

    // Check for language-specific markers
    let scores = [
        (Language::French, french_score(&words, &lower)),
        (Language::Spanish, spanish_score(&words, &lower)),
        (Language::German, german_score(&words, &lower)),
        (Language::Portuguese, portuguese_score(&words, &lower)),
        (Language::Japanese, japanese_score(&lower)),
    ];

    let (best_lang, best_score) = scores
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();

    if *best_score > 0.3 {
        *best_lang
    } else {
        // Default to English if no strong signal
        if words.iter().all(|w| {
            w.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        }) {
            Language::English
        } else {
            Language::Unknown
        }
    }
}

fn french_score(words: &[&str], _query: &str) -> f64 {
    const MARKERS: &[&str] = &[
        "de",
        "le",
        "la",
        "les",
        "un",
        "une",
        "du",
        "des",
        "en",
        "est",
        "sont",
        "avec",
        "pour",
        "dans",
        "sur",
        "par",
        "comme",
        "algorithme",
        "recherche",
        "tri",
        "fonction",
    ];
    let hits = words.iter().filter(|w| MARKERS.contains(w)).count();
    (hits as f64 / words.len().max(1) as f64).min(1.0)
}

fn spanish_score(words: &[&str], _query: &str) -> f64 {
    const MARKERS: &[&str] = &[
        "de",
        "el",
        "la",
        "los",
        "las",
        "un",
        "una",
        "en",
        "es",
        "son",
        "con",
        "para",
        "por",
        "como",
        "algoritmo",
        "buscar",
        "ordenar",
        "funcion",
    ];
    let hits = words.iter().filter(|w| MARKERS.contains(w)).count();
    (hits as f64 / words.len().max(1) as f64).min(1.0)
}

fn german_score(words: &[&str], _query: &str) -> f64 {
    const MARKERS: &[&str] = &[
        "der",
        "die",
        "das",
        "ein",
        "eine",
        "ist",
        "sind",
        "mit",
        "und",
        "von",
        "fur",
        "auf",
        "bei",
        "nach",
        "algorithmus",
        "suche",
        "sortierung",
        "funktion",
    ];
    let hits = words.iter().filter(|w| MARKERS.contains(w)).count();
    (hits as f64 / words.len().max(1) as f64).min(1.0)
}

fn portuguese_score(words: &[&str], _query: &str) -> f64 {
    const MARKERS: &[&str] = &[
        "de",
        "o",
        "a",
        "os",
        "as",
        "um",
        "uma",
        "em",
        "com",
        "para",
        "por",
        "como",
        "algoritmo",
        "busca",
        "ordenacao",
    ];
    let hits = words.iter().filter(|w| MARKERS.contains(w)).count();
    (hits as f64 / words.len().max(1) as f64).min(1.0)
}

fn japanese_score(query: &str) -> f64 {
    let jp_chars = query
        .chars()
        .filter(|c| {
            let cp = *c as u32;
            // Hiragana, Katakana, or CJK Unified Ideographs
            (0x3040..=0x309F).contains(&cp)
                || (0x30A0..=0x30FF).contains(&cp)
                || (0x4E00..=0x9FFF).contains(&cp)
        })
        .count();
    let total = query.chars().count().max(1);
    jp_chars as f64 / total as f64
}

// ─── Translation ─────────────────────────────────────────────

fn translate_to_english(query: &str, lang: Language) -> Option<String> {
    let dict = get_phrase_dict(lang);
    let lower = query.to_lowercase();

    // Try full phrase match first
    for (foreign, english) in &dict {
        if lower.contains(*foreign) {
            return Some(lower.replace(*foreign, english));
        }
    }

    // Try word-by-word translation
    let word_dict = get_word_dict(lang);
    let words: Vec<&str> = lower.split_whitespace().collect();
    let translated: Vec<String> = words
        .iter()
        .map(|w| {
            word_dict
                .get(*w)
                .map(|t| t.to_string())
                .unwrap_or_else(|| w.to_string())
        })
        .collect();

    let result = translated.join(" ");
    if result != lower {
        Some(result)
    } else {
        None
    }
}

fn translate_terms(query: &str) -> Vec<(String, String, Language, f64)> {
    let mut results = Vec::new();
    let lower = query.to_lowercase();

    for lang in [
        Language::French,
        Language::Spanish,
        Language::German,
        Language::Portuguese,
    ] {
        let dict = get_word_dict(lang);
        for (foreign, english) in &dict {
            if lower.contains(foreign) && *foreign != *english {
                results.push((foreign.to_string(), english.to_string(), lang, 0.7));
            }
        }
    }

    results
}

fn get_phrase_dict(lang: Language) -> Vec<(&'static str, &'static str)> {
    match lang {
        Language::French => vec![
            ("algorithme de tri", "sorting algorithm"),
            ("recherche binaire", "binary search"),
            ("apprentissage automatique", "machine learning"),
            ("intelligence artificielle", "artificial intelligence"),
            ("base de donnees", "database"),
            ("reseau de neurones", "neural network"),
            ("structure de donnees", "data structure"),
        ],
        Language::Spanish => vec![
            ("algoritmo de ordenamiento", "sorting algorithm"),
            ("busqueda binaria", "binary search"),
            ("aprendizaje automatico", "machine learning"),
            ("inteligencia artificial", "artificial intelligence"),
            ("base de datos", "database"),
            ("red neuronal", "neural network"),
            ("estructura de datos", "data structure"),
        ],
        Language::German => vec![
            ("sortieralgorithmus", "sorting algorithm"),
            ("binaere suche", "binary search"),
            ("maschinelles lernen", "machine learning"),
            ("kuenstliche intelligenz", "artificial intelligence"),
            ("neuronales netz", "neural network"),
            ("datenstruktur", "data structure"),
        ],
        _ => vec![],
    }
}

fn get_word_dict(lang: Language) -> HashMap<&'static str, &'static str> {
    match lang {
        Language::French => [
            ("algorithme", "algorithm"),
            ("tri", "sorting"),
            ("recherche", "search"),
            ("fonction", "function"),
            ("variable", "variable"),
            ("boucle", "loop"),
            ("tableau", "array"),
            ("liste", "list"),
            ("arbre", "tree"),
            ("graphe", "graph"),
            ("pile", "stack"),
            ("file", "queue"),
            ("rapide", "fast"),
            ("lent", "slow"),
            ("erreur", "error"),
        ]
        .into_iter()
        .collect(),
        Language::Spanish => [
            ("algoritmo", "algorithm"),
            ("ordenar", "sort"),
            ("buscar", "search"),
            ("funcion", "function"),
            ("variable", "variable"),
            ("bucle", "loop"),
            ("arreglo", "array"),
            ("lista", "list"),
            ("arbol", "tree"),
            ("grafo", "graph"),
            ("pila", "stack"),
            ("cola", "queue"),
            ("rapido", "fast"),
            ("lento", "slow"),
            ("error", "error"),
        ]
        .into_iter()
        .collect(),
        Language::German => [
            ("algorithmus", "algorithm"),
            ("sortierung", "sorting"),
            ("suche", "search"),
            ("funktion", "function"),
            ("variable", "variable"),
            ("schleife", "loop"),
            ("datenbank", "database"),
            ("liste", "list"),
            ("baum", "tree"),
            ("schnell", "fast"),
            ("langsam", "slow"),
            ("fehler", "error"),
        ]
        .into_iter()
        .collect(),
        Language::Portuguese => [
            ("algoritmo", "algorithm"),
            ("ordenacao", "sorting"),
            ("busca", "search"),
            ("funcao", "function"),
            ("variavel", "variable"),
            ("laco", "loop"),
            ("lista", "list"),
            ("arvore", "tree"),
            ("grafo", "graph"),
            ("rapido", "fast"),
            ("lento", "slow"),
            ("erro", "error"),
        ]
        .into_iter()
        .collect(),
        _ => HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_query_no_expansions() {
        let result = expand_crosslingual("sorting algorithm implementation");
        assert_eq!(result.detected_language, Language::English);
        assert!(result.expansions.is_empty());
    }

    #[test]
    fn french_query_expanded() {
        let result = expand_crosslingual("algorithme de tri rapide en python");
        assert_eq!(result.detected_language, Language::French);
        assert!(!result.expansions.is_empty());
        let has_english = result
            .expansions
            .iter()
            .any(|e| e.target_lang == Language::English);
        assert!(has_english);
    }

    #[test]
    fn spanish_query_expanded() {
        let result = expand_crosslingual("algoritmo de ordenamiento en lista");
        assert_eq!(result.detected_language, Language::Spanish);
        assert!(!result.expansions.is_empty());
    }

    #[test]
    fn german_query_expanded() {
        let result = expand_crosslingual("sortierung algorithmus mit liste und baum");
        assert_eq!(result.detected_language, Language::German);
        assert!(!result.expansions.is_empty());
    }

    #[test]
    fn empty_query() {
        let result = expand_crosslingual("");
        assert!(result.expansions.is_empty());
    }

    #[test]
    fn max_three_expansions() {
        let result = expand_crosslingual("algorithme de tri rapide avec boucle et tableau");
        assert!(result.expansions.len() <= 3);
    }

    #[test]
    fn japanese_detected() {
        let lang = detect_language("ソートアルゴリズム");
        assert_eq!(lang, Language::Japanese);
    }

    #[test]
    fn mixed_language_term_translation() {
        // English query with a foreign technical term
        let translations = translate_terms("implement algorithme in python");
        assert!(!translations.is_empty());
        let has_algorithm = translations
            .iter()
            .any(|(_, eng, _, _)| eng.contains("algorithm"));
        assert!(has_algorithm);
    }

    #[test]
    fn confidence_sorted_descending() {
        let result = expand_crosslingual("algorithme de tri avec recherche");
        if result.expansions.len() >= 2 {
            for i in 0..result.expansions.len() - 1 {
                assert!(result.expansions[i].confidence >= result.expansions[i + 1].confidence);
            }
        }
    }
}
