//! `fetchium compare` — side-by-side comparison of two or more items.

use anyhow::{Context, Result};
use fetchium_core::compare::parser::parse_comparison_query;
use fetchium_core::compare::{build_comparison, build_comparison_ai_unified};
use fetchium_core::config::HsxConfig;
use fetchium_core::http::client::HttpClient;
use fetchium_core::search::orchestrator::{OrchestratorConfig, SearchOrchestrator};
use std::sync::Arc;
use std::time::Instant;

use crate::cli::{CompareArgs, Format};

pub async fn run(args: CompareArgs, config: &HsxConfig, format: Format) -> Result<()> {
    let start = Instant::now();
    let parsed = parse_comparison_query(&args.query);

    if parsed.items.len() < 2 {
        eprintln!(
            "Could not detect items to compare in {:?}. \
             Use \"A vs B\" or \"compare A and B\" syntax.",
            args.query
        );
        std::process::exit(1);
    }

    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_style(
        indicatif::ProgressStyle::default_spinner()
            .template("{spinner:.cyan} [{elapsed_precise}] {msg}")
            .unwrap(),
    );
    spinner.set_message(format!("Comparing: {}", parsed.items.join(" vs ")));
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let http = HttpClient::new(config).context("Failed to build HTTP client")?;
    let orch_config = OrchestratorConfig::from_hsx_config(config, args.max_sources as u32);
    let orchestrator = Arc::new(SearchOrchestrator::new(http.clone(), orch_config));

    // Build comparison context from original query for disambiguation
    let comparison_context = parsed.items.join(" vs ");

    // Collect search snippets from multiple context-aware queries in parallel
    spinner.set_message(format!("Searching: {}...", comparison_context));

    let mut search_handles = Vec::new();
    // Query 1: Direct comparison query
    {
        let orch = orchestrator.clone();
        let ctx = comparison_context.clone();
        let max = args.max_sources as u32;
        search_handles.push(tokio::spawn(async move {
            orch.search(&format!("{ctx} comparison"), Some(max * 2))
                .await
                .unwrap_or_default()
        }));
    }
    // Query 2: Per-item queries in parallel
    for item in &parsed.items {
        let orch = orchestrator.clone();
        let item = item.clone();
        let ctx = comparison_context.clone();
        let max = args.max_sources as u32;
        search_handles.push(tokio::spawn(async move {
            orch.search(
                &format!("{item} {ctx} features performance ecosystem"),
                Some(max),
            )
            .await
            .unwrap_or_default()
        }));
    }

    // Collect all search results
    let mut all_results = Vec::new();
    for handle in search_handles {
        if let Ok(results) = handle.await {
            all_results.extend(results);
        }
    }

    // Deduplicate by URL
    let mut seen_urls = std::collections::HashSet::new();
    all_results.retain(|r| seen_urls.insert(r.url.clone()));

    // Pre-filter: keep only results that mention at least one compared item
    let items_lower: Vec<String> = parsed.items.iter().map(|i| i.to_lowercase()).collect();
    all_results.retain(|r| {
        let title = r.title.to_lowercase();
        let snippet = r.snippet.to_lowercase();
        items_lower
            .iter()
            .any(|i| title.contains(i) || snippet.contains(i))
            || title.contains("comparison")
            || title.contains(" vs ")
    });

    // Build snippet context from search results (fast, no page fetching needed)
    let mut snippet_text = String::new();
    let mut sources: Vec<String> = Vec::new();
    for (i, result) in all_results.iter().enumerate().take(args.max_sources * 2) {
        snippet_text.push_str(&format!("[{}] {}\n", i + 1, result.title));
        if !result.snippet.is_empty() {
            snippet_text.push_str(&result.snippet);
            snippet_text.push('\n');
        }
        sources.push(result.url.clone());
    }

    spinner.set_message("Building comparison table...");

    // Build per-item data for heuristic fallback
    let item_data: Vec<(String, String, Vec<String>)> = parsed
        .items
        .iter()
        .map(|item| {
            let item_lower = item.to_lowercase();
            let relevant: String = all_results
                .iter()
                .filter(|r| {
                    let t = r.title.to_lowercase();
                    let s = r.snippet.to_lowercase();
                    t.contains(&item_lower) || s.contains(&item_lower)
                })
                .map(|r| format!("{}\n{}\n", r.title, r.snippet))
                .collect();
            (item.clone(), relevant, sources.clone())
        })
        .collect();

    // Use AI by default when available (--no-ai to disable)
    let use_ai = args.ai
        || config.ai.providers.fallback_chain.iter().any(|p| {
            let lower = p.to_lowercase();
            lower.contains("gemini")
                || lower.contains("openai")
                || lower.contains("anthropic")
                || lower.contains("antigravity")
        });

    let comparison = if use_ai {
        build_comparison_ai_unified(&parsed, &snippet_text, &sources, config).await
    } else {
        build_comparison(&parsed, &item_data)
    };

    spinner.finish_and_clear();

    let output = match format {
        Format::Json => serde_json::to_string_pretty(&comparison)?,
        _ => comparison.markdown_table.clone(),
    };

    if let Some(path) = &args.output {
        std::fs::write(path, &output)?;
        eprintln!("Comparison written to {path}");
    } else {
        println!("{output}");
    }

    let elapsed = start.elapsed();
    eprintln!("\nCompleted in {:.1}s", elapsed.as_secs_f64());

    Ok(())
}
