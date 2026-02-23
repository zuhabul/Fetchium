//! Sandwich layout context assembly (Ms-PoE inspired) for mitigating "lost in the middle" (PRD §23).
//!
//! The "lost in the middle" problem: LLMs attend most strongly to the
//! beginning and end of the context window, paying less attention to the middle.
//!
//! **Sandwich layout** places the HIGHEST-confidence sources at the start and end,
//! and LOWER-confidence sources in the middle where attention is weakest.
//!
//! Example with 6 sources ranked by confidence (1 = best):
//! - Input:    [1, 2, 3, 4, 5, 6]
//! - Sandwich: [1, 3, 5, 6, 4, 2]
//!   - Start: 1 (best)
//!   - Middle: 3, 5, 6 (weakest attention)
//!   - End: 4, 2 (second best)

use crate::ai::types::RankedSource;

/// Reorder sources using the sandwich layout.
///
/// Returns an empty vec if input is empty. Sources with ≤2 items are returned as-is.
pub fn sandwich_layout(mut sources: Vec<RankedSource>) -> Vec<RankedSource> {
    if sources.len() <= 2 {
        return sources;
    }

    // Sort descending by confidence (best first).
    sources.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let n = sources.len();
    let mut result: Vec<Option<RankedSource>> = (0..n).map(|_| None).collect();
    let mut front = 0usize;
    let mut back = n - 1;

    for (i, source) in sources.into_iter().enumerate() {
        if i % 2 == 0 {
            // High confidence → front (start of context window)
            result[front] = Some(source);
            front += 1;
        } else {
            // Slightly lower → back (end of context window)
            result[back] = Some(source);
            back = back.saturating_sub(1);
        }
    }

    result.into_iter().flatten().collect()
}

/// Assemble the context string from sandwich-ordered sources within a token budget.
///
/// Returns `(context_text, source_map)` where `source_map[i]` is the original
/// source index for position `i`.
pub fn assemble_context(sources: &[RankedSource], token_budget: usize) -> (String, Vec<usize>) {
    let mut context = String::new();
    let mut source_map = Vec::new();
    let mut tokens_used = 0usize;

    for (pos, source) in sources.iter().enumerate() {
        // Accurate token estimate via tiktoken
        let source_tokens = crate::extract::layer1::estimate_tokens(&source.content) as usize + 20; // +20 for header overhead

        if tokens_used + source_tokens > token_budget {
            let remaining_tokens = token_budget.saturating_sub(tokens_used);
            let remaining_chars = (remaining_tokens * 4).saturating_sub(80);
            if remaining_chars == 0 {
                break;
            }
            let truncated = &source.content[..remaining_chars.min(source.content.len())];
            context.push_str(&format!(
                "[Source {}] {}\nURL: {}\n{}\n\n",
                pos + 1,
                source.title,
                source.url,
                truncated,
            ));
            source_map.push(source.index);
            break;
        }

        context.push_str(&format!(
            "[Source {}] {}\nURL: {}\n{}\n\n",
            pos + 1,
            source.title,
            source.url,
            source.content,
        ));
        source_map.push(source.index);
        tokens_used += source_tokens;
    }

    (context, source_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_source(idx: usize, confidence: f64, content: &str) -> RankedSource {
        RankedSource {
            index: idx,
            content: content.into(),
            confidence,
            url: format!("https://example{idx}.com"),
            title: format!("Source {idx}"),
        }
    }

    #[test]
    fn empty_input_returns_empty() {
        let result = sandwich_layout(vec![]);
        assert!(result.is_empty());
    }

    #[test]
    fn single_source_passthrough() {
        let s = vec![make_source(0, 0.9, "content")];
        let result = sandwich_layout(s);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn sandwich_places_best_first() {
        let sources = vec![
            make_source(0, 0.5, "c"),
            make_source(1, 0.9, "a"),
            make_source(2, 0.7, "b"),
            make_source(3, 0.3, "d"),
        ];
        let ordered = sandwich_layout(sources);
        // Best (0.9) should be first
        assert!((ordered[0].confidence - 0.9).abs() < 1e-9);
    }

    #[test]
    fn assemble_context_respects_budget() {
        let sources: Vec<RankedSource> = (0..5)
            .map(|i| make_source(i, 0.8, &"x".repeat(400)))
            .collect();
        let (ctx, map) = assemble_context(&sources, 200); // very tight budget
        // Context should be shorter than a full dump
        assert!(ctx.len() < 5 * 400);
        assert!(!map.is_empty());
    }

    #[test]
    fn assemble_context_includes_source_header() {
        let sources = vec![make_source(0, 0.9, "hello world content")];
        let (ctx, _) = assemble_context(&sources, 10_000);
        assert!(ctx.contains("[Source 1]"));
        assert!(ctx.contains("https://example0.com"));
    }
}
