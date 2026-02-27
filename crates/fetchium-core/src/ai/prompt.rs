//! System prompt templates for AI synthesis (PRD §23).

/// Full synthesis system prompt for multi-source research queries.
///
/// Instructs the model to cite every claim using `[N]` notation.
pub fn synthesis_system_prompt(query: &str, source_count: usize) -> String {
    format!(
        r#"You are a research synthesis assistant for Fetchium. Your task is to provide a clear, accurate, and well-cited answer to the user's query.

RULES:
1. Base your answer ONLY on the provided sources. Never fabricate information.
2. Cite every factual claim using [N] notation where N is the source number.
3. If sources disagree, note the contradiction and cite both sides.
4. If no source adequately answers the query, say so explicitly.
5. Be concise but thorough. Prefer specificity over vagueness.
6. Structure your answer with clear paragraphs. Use bullet points for lists.
7. End with a "Sources" section listing [N] URL pairs used.

You have {source_count} sources available. The user's query is: "{query}"

Respond with your synthesized answer now."#
    )
}

/// Lighter prompt for simple factual queries.
pub fn factual_system_prompt(query: &str) -> String {
    format!(
        r#"Answer the following question concisely based on the provided sources. Cite every fact with [N] where N is the source number. If unsure, say so.

Question: "{query}""#
    )
}

/// Prompt for when Ollama is unavailable (fallback mode).
pub fn fallback_prompt(query: &str, sources_count: usize) -> String {
    format!(
        "AI synthesis unavailable (Ollama not running). Here are {sources_count} search results for: \"{query}\""
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthesis_prompt_contains_query() {
        let prompt = synthesis_system_prompt("what is Rust", 5);
        assert!(prompt.contains("what is Rust"));
        assert!(prompt.contains("5 sources"));
        assert!(prompt.contains("[N]"));
    }

    #[test]
    fn factual_prompt_contains_query() {
        let prompt = factual_system_prompt("what is a borrow checker");
        assert!(prompt.contains("borrow checker"));
        assert!(prompt.contains("[N]"));
    }
}
