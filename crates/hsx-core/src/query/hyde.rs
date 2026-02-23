//! HyDE — Hypothetical Document Embeddings (Phase 5, PRD §21-22).
//!
//! For ambiguous short queries, generates a hypothetical answer via Ollama,
//! then embeds *that* text instead of the raw query. The embedding better
//! captures the semantic space of the desired document.
//!
//! Activation criteria (AND):
//! 1. Query confidence < 0.6 (detected by intent classifier)
//! 2. Query length < 6 words
//! 3. `embeddings` feature is enabled
//! 4. An LLM (Ollama) is reachable

#[cfg(feature = "embeddings")]
use tracing::{debug, info};

/// Prompt template for HyDE document generation.
const HYDE_PROMPT: &str = "Write a short, factual paragraph that would be a perfect answer to \
                            this question. Do not include any preamble or meta-commentary. \
                            Just write the answer directly.\n\nQuestion: {query}\n\nAnswer:";

/// Determine whether to apply HyDE for a query.
///
/// Returns `true` when the query is short and ambiguous (heuristic only).
pub fn should_use_hyde(query: &str, intent_confidence: f64) -> bool {
    let word_count = query.split_whitespace().count();
    intent_confidence < 0.6 && word_count < 6
}

/// Build the HyDE prompt for a given query.
pub fn hyde_prompt(query: &str) -> String {
    HYDE_PROMPT.replace("{query}", query)
}

/// Feature-gated: embed a query using HyDE when available.
///
/// When the `embeddings` feature is disabled, returns the empty vec (caller should
/// fall back to BM25-only ranking).
#[cfg(feature = "embeddings")]
pub async fn hyde_embed(
    query: &str,
    ollama_response: Option<&str>,
) -> Result<Vec<f32>, crate::error::HsxError> {
    // If we have an LLM-generated hypothetical document, embed that instead
    let text_to_embed = ollama_response.unwrap_or(query);

    debug!(
        query = query,
        using_hypothetical = ollama_response.is_some(),
        "HyDE embed"
    );

    crate::embeddings::embed(text_to_embed)
}

/// Smart embed: use HyDE for ambiguous queries, direct embed otherwise.
///
/// `ollama_response` is the optional LLM-generated hypothetical answer.
#[cfg(feature = "embeddings")]
pub async fn smart_embed(
    query: &str,
    intent_confidence: f64,
    ollama_response: Option<&str>,
) -> Result<Vec<f32>, crate::error::HsxError> {
    if should_use_hyde(query, intent_confidence) && ollama_response.is_some() {
        info!(query = query, "Applying HyDE for ambiguous query");
        hyde_embed(query, ollama_response).await
    } else {
        debug!(query = query, "Direct query embedding (no HyDE)");
        crate::embeddings::embed(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_low_confidence_uses_hyde() {
        assert!(should_use_hyde("Apple", 0.3));
    }

    #[test]
    fn long_query_no_hyde() {
        assert!(!should_use_hyde("What is the difference between Rust and Go for systems programming", 0.4));
    }

    #[test]
    fn high_confidence_no_hyde() {
        assert!(!should_use_hyde("Paris", 0.9));
    }

    #[test]
    fn hyde_prompt_contains_query() {
        let prompt = hyde_prompt("What is Rust?");
        assert!(prompt.contains("What is Rust?"));
        assert!(prompt.contains("Answer:"));
    }
}
