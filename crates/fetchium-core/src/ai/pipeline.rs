//! Full AI preview pipeline: search → extract → sandwich → AI provider → output (PRD §23 Mode D).

use crate::ai::prompt::{factual_system_prompt, synthesis_system_prompt};
use crate::ai::provider_client::chat_with_fallback;
use crate::ai::sandwich::{assemble_context, sandwich_layout};
use crate::ai::setup::{format_setup_guide, DeviceSpec};
use crate::ai::types::{AiConfig, AiPreviewResult, ChatMessage, RankedSource};
use crate::config::HsxConfig;
use crate::error::HsxError;
use crate::extract::pipeline::extract;
use crate::http::client::HttpClient;
use crate::search::orchestrator::{OrchestratorConfig, SearchOrchestrator};
use std::collections::HashSet;
use std::io::Write;
use std::time::Instant;

/// Execute the full AI preview pipeline for a query.
///
/// # Pipeline steps
/// 1. Multi-backend search via `SearchOrchestrator`
/// 2. Fetch and extract content for top `max_sources` results
///    (skipped in `fast` mode — uses search snippets directly)
/// 3. Assemble sandwich-layout context (Ms-PoE)
/// 4. Route to best available AI provider by complexity
/// 5. Call AI provider (streaming or not)
/// 6. Return the answer with metadata and per-phase timing
#[allow(clippy::too_many_arguments)]
pub async fn run_ai_pipeline(
    query: &str,
    model_override: Option<&str>,
    token_budget: usize,
    max_sources: usize,
    streaming: bool,
    fast: bool,
    ai_config: &AiConfig,
    fetchium_config: &HsxConfig,
    http_client: &HttpClient,
) -> Result<AiPreviewResult, HsxError> {
    let wall_start = Instant::now();

    // Step 1: Search
    let search_start = Instant::now();
    let orch_config = OrchestratorConfig::from_fetchium_config(fetchium_config, max_sources as u32);
    let orchestrator = SearchOrchestrator::new(http_client.clone(), orch_config);
    let mut search_results = orchestrator.search(query, Some(max_sources as u32)).await?;
    let quality = crate::rank::quality::assess_quality(&search_results, query);
    if matches!(
        quality.confidence,
        crate::rank::quality::ConfidenceLevel::Low | crate::rank::quality::ConfidenceLevel::VeryLow
    ) {
        let mut seen_urls: HashSet<String> = search_results.iter().map(|r| r.url.clone()).collect();
        for cq in generate_corrective_queries(query).into_iter().take(2) {
            if let Ok(extra) = orchestrator.search(&cq, Some(6)).await {
                for r in extra {
                    if seen_urls.insert(r.url.clone()) {
                        search_results.push(r);
                    }
                }
            }
        }
        search_results = crate::rank::rerank(search_results, query);
        search_results.truncate(max_sources);
    }
    let search_ms = search_start.elapsed().as_millis() as u64;

    let top_n = search_results.len().min(max_sources);
    let per_source_budget = (token_budget / top_n.max(1)).max(200);

    // Step 2: Build ranked sources
    // Fast mode: use search snippets directly (no HTTP fetch, eliminates 3-8s of latency).
    // Full mode: fetch full page HTML + CEP extraction with high concurrency + per-source timeout.
    let fetch_start = Instant::now();
    let ranked_sources: Vec<RankedSource> = if fast {
        // Use snippets directly — zero network latency
        search_results
            .into_iter()
            .take(top_n)
            .enumerate()
            .filter(|(_, item)| !item.snippet.is_empty())
            .map(|(idx, item)| RankedSource {
                index: idx,
                content: item.snippet.clone(),
                confidence: item.score.unwrap_or(0.5),
                url: item.url.clone(),
                title: item.title.clone(),
            })
            .collect()
    } else {
        // Full extraction: fetch HTML concurrently (8 at a time) with 6s per-source timeout
        use futures::stream::{self, StreamExt};
        use std::time::Duration;
        use tokio::time::timeout;

        let mut fetch_stream = stream::iter(search_results.into_iter().take(top_n).enumerate())
            .map(|(orig_idx, item)| {
                let client = http_client.clone();
                let budget = per_source_budget;
                async move {
                    let html =
                        match timeout(Duration::from_secs(6), client.fetch_text(&item.url)).await {
                            Ok(Ok(s)) => s,
                            _ => String::new(),
                        };

                    // Skip binary content (images, PDFs, etc.) that slipped through
                    let is_binary = html.len() > 4
                        && (html.starts_with("\u{fffd}")
                            || html.starts_with("JFIF")
                            || html.starts_with("%PDF")
                            || html
                                .as_bytes()
                                .iter()
                                .take(512)
                                .filter(|b| **b == 0)
                                .count()
                                > 5);

                    let extracted = if html.is_empty() || is_binary {
                        None
                    } else {
                        let ext = extract(&html, &item.url);
                        let max_chars = budget * 4;
                        let content = if ext.text.chars().count() > max_chars {
                            ext.text.chars().take(max_chars).collect::<String>()
                        } else {
                            ext.text
                        };
                        if content.trim().is_empty() {
                            None
                        } else {
                            Some((ext.title, content))
                        }
                    };
                    (orig_idx, item, extracted)
                }
            })
            .buffer_unordered(8); // Up from 3 → 8 for maximum parallel fetching

        let mut sources: Vec<RankedSource> = Vec::new();
        while let Some((orig_idx, item, extracted)) = fetch_stream.next().await {
            if let Some((title, content)) = extracted {
                sources.push(RankedSource {
                    index: orig_idx,
                    content,
                    confidence: item.score.unwrap_or(0.5),
                    url: item.url.clone(),
                    title,
                });
            }
        }
        sources
    };
    let fetch_ms = fetch_start.elapsed().as_millis() as u64;

    // Step 3: Sandwich layout + context assembly
    let ordered = sandwich_layout(ranked_sources);
    let (context, _source_map) = assemble_context(&ordered, token_budget);
    let sources_used = ordered.len();

    // Step 4: Build chat messages
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

    // Step 5: Call the configured provider chain
    let chain = ai_config.providers.resolved_chain();
    let chain_empty = chain.is_empty();

    // In fast mode with GeminiCli: use gemini-2.5-flash (no thinking, ~5x faster)
    // unless the user explicitly specified a model override.
    let fast_model_override: Option<String> = if fast && model_override.is_none() {
        let uses_gemini_cli = chain
            .first()
            .map(|k| *k == crate::ai::providers::ProviderKind::GeminiCli)
            .unwrap_or(false);
        if uses_gemini_cli {
            Some("gemini-2.5-flash".into())
        } else {
            None
        }
    } else {
        None
    };
    let effective_model = model_override
        .map(|s| s.to_string())
        .or(fast_model_override);

    let ai_start = Instant::now();
    let mut on_token = |chunk: &str| {
        if streaming {
            print!("{chunk}");
            let _ = std::io::stdout().flush();
        }
    };

    match chat_with_fallback(
        &messages,
        effective_model.as_deref(),
        ai_config,
        &ai_config.providers,
        streaming,
        &mut on_token,
    )
    .await
    {
        Ok(result) => {
            let grounding = grounding_score(&result.content, &context);
            if grounding < 0.12 {
                let spec = DeviceSpec::detect();
                return Ok(AiPreviewResult {
                    answer: format_fallback(&ordered, query, &spec),
                    model_used: format!("{}/{}", result.provider.slug(), result.model_used),
                    sources_used,
                    streaming: false,
                    fallback: true,
                    total_ms: wall_start.elapsed().as_millis() as u64,
                    search_ms,
                    fetch_ms,
                    ai_ms: ai_start.elapsed().as_millis() as u64,
                    fast_mode: fast,
                });
            }
            let ai_ms = ai_start.elapsed().as_millis() as u64;
            Ok(AiPreviewResult {
                answer: result.content,
                model_used: format!("{}/{}", result.provider.slug(), result.model_used),
                sources_used,
                streaming,
                fallback: false,
                total_ms: wall_start.elapsed().as_millis() as u64,
                search_ms,
                fetch_ms,
                ai_ms,
                fast_mode: fast,
            })
        }
        Err(_) if chain_empty || chain == vec![crate::ai::providers::ProviderKind::Ollama] => {
            // Pure Ollama fallback — show setup guide
            let spec = DeviceSpec::detect();
            Ok(AiPreviewResult {
                answer: format_fallback(&ordered, query, &spec),
                model_used: "none (no provider available)".into(),
                sources_used,
                streaming: false,
                fallback: true,
                total_ms: wall_start.elapsed().as_millis() as u64,
                search_ms,
                fetch_ms,
                ai_ms: 0,
                fast_mode: fast,
            })
        }
        Err(e) => Err(e),
    }
}

fn generate_corrective_queries(query: &str) -> Vec<String> {
    let q = query.trim();
    let lower = q.to_lowercase();
    let mut out = Vec::new();
    if !lower.contains("overview") {
        out.push(format!("{q} overview"));
    }
    if !lower.contains("explained") {
        out.push(format!("{q} explained"));
    }
    out
}

fn grounding_score(answer: &str, source_context: &str) -> f64 {
    let src = source_context.to_lowercase();
    let mut total = 0usize;
    let mut supported = 0usize;
    for sentence in answer.split(['.', '!', '?']) {
        let terms = sentence
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| t.len() >= 4)
            .map(|t| t.to_lowercase())
            .collect::<Vec<_>>();
        if terms.is_empty() {
            continue;
        }
        total += 1;
        let hits = terms.iter().filter(|t| src.contains(t.as_str())).count();
        if hits >= 2 {
            supported += 1;
        }
    }
    if total == 0 {
        0.0
    } else {
        supported as f64 / total as f64
    }
}

/// Format search results as a fallback when Ollama is not available.
fn format_fallback(sources: &[RankedSource], query: &str, spec: &DeviceSpec) -> String {
    let mut out = format!(
        "Search results for \"{}\" (AI synthesis unavailable — Ollama not running)\n\n",
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
    out.push_str(&format_setup_guide(spec));
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
        let spec = DeviceSpec {
            total_ram_gb: 16.0,
            cpu_cores: 8,
            is_apple_silicon: false,
            usable_ram_gb: 11.5,
        };
        let result = format_fallback(&sources, "test query", &spec);
        assert!(result.contains("test query"));
        assert!(result.contains("Example"));
        assert!(result.contains("https://example.com"));
        assert!(result.contains("ollama pull"));
    }

    #[test]
    fn grounding_score_higher_when_supported() {
        let src =
            "Rust is a systems programming language with memory safety and zero cost abstractions.";
        let good = "Rust is a systems programming language with memory safety.";
        let bad = "Rust was invented on Mars by aliens.";
        assert!(grounding_score(good, src) > grounding_score(bad, src));
    }
}
