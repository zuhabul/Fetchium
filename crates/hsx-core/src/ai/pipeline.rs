//! Full AI preview pipeline: search → extract → sandwich → Ollama → output (PRD §23 Mode D).

use crate::ai::ollama::OllamaClient;
use crate::ai::prompt::{factual_system_prompt, synthesis_system_prompt};
use crate::ai::router::{route_model, select_model};
use crate::ai::sandwich::{assemble_context, sandwich_layout};
use crate::ai::types::{AiConfig, AiPreviewResult, ChatMessage, RankedSource};
use crate::config::HsxConfig;
use crate::error::HsxError;
use crate::extract::pipeline::extract;
use crate::http::client::HttpClient;
use crate::search::orchestrator::{OrchestratorConfig, SearchOrchestrator};
use std::io::Write;

/// Execute the full AI preview pipeline for a query.
///
/// # Pipeline steps
/// 1. Multi-backend search via `SearchOrchestrator`
/// 2. Fetch and extract content for top `max_sources` results
/// 3. Assemble sandwich-layout context (Ms-PoE)
/// 4. Route to best available Ollama model by complexity
/// 5. Call Ollama (streaming or not)
/// 6. Return the answer with metadata
#[allow(clippy::too_many_arguments)]
pub async fn run_ai_pipeline(
    query: &str,
    model_override: Option<&str>,
    token_budget: usize,
    max_sources: usize,
    streaming: bool,
    ai_config: &AiConfig,
    hsx_config: &HsxConfig,
    http_client: &HttpClient,
) -> Result<AiPreviewResult, HsxError> {
    // Step 1: Search
    let orch_config = OrchestratorConfig::from_hsx_config(hsx_config, max_sources as u32);
    let orchestrator = SearchOrchestrator::new(http_client.clone(), orch_config);
    let search_results = orchestrator
        .search(query, Some(max_sources as u32))
        .await?;

    let top_n = search_results.len().min(max_sources);
    let per_source_budget = (token_budget / top_n.max(1)).max(200);

    // Step 2: Fetch and extract content with bounded concurrency (3) to save memory
    use futures::stream::{self, StreamExt};
    
    let mut fetch_stream = stream::iter(search_results.into_iter().take(top_n).enumerate())
        .map(|(orig_idx, item)| {
            let client = http_client.clone();
            let budget = per_source_budget;
            async move {
                let html = client.fetch_text(&item.url).await.unwrap_or_default();
                let extracted = if html.is_empty() {
                    None
                } else {
                    let ext = extract(&html, &item.url);
                    let max_chars = budget * 4;
                    let content = if ext.text.len() > max_chars {
                        ext.text[..max_chars].to_string()
                    } else {
                        ext.text
                    };
                    Some((ext.title, content))
                };
                (orig_idx, item, extracted)
            }
        })
        .buffer_unordered(3); // Cap concurrent HTML buffering locally

    let mut ranked_sources: Vec<RankedSource> = Vec::new();
    while let Some((orig_idx, item, extracted)) = fetch_stream.next().await {
        if let Some((title, content)) = extracted {
            if !content.trim().is_empty() {
                ranked_sources.push(RankedSource {
                    index: orig_idx,
                    content,
                    confidence: item.score.unwrap_or(0.5),
                    url: item.url.clone(),
                    title,
                });
            }
        }
    }

    // Step 3: Sandwich layout + context assembly
    let ordered = sandwich_layout(ranked_sources);
    let (context, _source_map) = assemble_context(&ordered, token_budget);
    let sources_used = ordered.len();

    // Step 4: Check Ollama availability
    let ollama = OllamaClient::new(ai_config);

    if !ollama.is_available().await {
        return Ok(AiPreviewResult {
            answer: format_fallback(&ordered, query),
            model_used: "none (Ollama not running)".into(),
            sources_used,
            streaming: false,
            fallback: true,
        });
    }

    // Step 5: Route to best model
    let available_models = ollama
        .list_models()
        .await
        .unwrap_or_default();

    let tier = route_model(query, sources_used);
    let model_name = select_model(&available_models, tier, model_override).ok_or_else(|| {
        HsxError::AiUnavailable(
            "No models installed in Ollama. Run `ollama pull deepseek-r1:7b` to install one."
                .into(),
        )
    })?;

    // Step 6: Build chat messages
    let system_prompt = if sources_used > 3 {
        synthesis_system_prompt(query, sources_used)
    } else {
        factual_system_prompt(query)
    };

    let messages = vec![
        ChatMessage {
            role: "system".into(),
            content: system_prompt,
        },
        ChatMessage {
            role: "user".into(),
            content: format!("Sources:\n\n{}\n\nAnswer the query: \"{}\"", context, query),
        },
    ];

    // Step 7: Call Ollama
    let answer = if streaming {
        ollama
            .chat_stream(
                &model_name,
                &messages,
                ai_config.temperature,
                |chunk| {
                    print!("{chunk}");
                    let _ = std::io::stdout().flush();
                },
            )
            .await?
    } else {
        ollama
            .chat(&model_name, &messages, ai_config.temperature)
            .await?
    };

    Ok(AiPreviewResult {
        answer,
        model_used: model_name,
        sources_used,
        streaming,
        fallback: false,
    })
}

/// Format search results as a fallback when Ollama is not available.
fn format_fallback(sources: &[RankedSource], query: &str) -> String {
    let mut out = format!(
        "AI synthesis unavailable (Ollama not running).\n\
         Install with: https://ollama.ai | Then run: ollama pull deepseek-r1:7b\n\n\
         Search results for \"{}\":\n\n",
        query
    );
    for (i, s) in sources.iter().enumerate() {
        let snippet: String = s.content.chars().take(200).collect();
        out.push_str(&format!(
            "[{}] {} (confidence: {:.0}%)\n    {}\n    {}...\n\n",
            i + 1,
            s.title,
            s.confidence * 100.0,
            s.url,
            snippet
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_fallback_is_non_empty() {
        let sources = vec![RankedSource {
            index: 0,
            content: "Some content here".into(),
            confidence: 0.8,
            url: "https://example.com".into(),
            title: "Example".into(),
        }];
        let result = format_fallback(&sources, "test query");
        assert!(result.contains("test query"));
        assert!(result.contains("Example"));
        assert!(result.contains("https://example.com"));
    }
}
