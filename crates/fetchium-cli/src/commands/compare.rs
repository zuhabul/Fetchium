//! `fetchium compare` — side-by-side comparison of two or more items.

use anyhow::{Context, Result};
use fetchium_core::compare::parser::parse_comparison_query;
use fetchium_core::compare::{build_comparison, build_comparison_ai};
use fetchium_core::config::HsxConfig;
use fetchium_core::extract::pipeline;
use fetchium_core::http::client::HttpClient;
use fetchium_core::search::orchestrator::{OrchestratorConfig, SearchOrchestrator};
use fetchium_core::token::qatbe::extract_with_budget;
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
    let orchestrator = SearchOrchestrator::new(http.clone(), orch_config);

    // Gather text snippets for each item via web search + QATBE extraction.
    let mut item_data: Vec<(String, String, Vec<String>)> = Vec::new();

    for item in &parsed.items {
        spinner.set_message(format!("Researching: {}...", item));
        // Use review-targeted query for better results
        let search_query = format!("{} features pros cons review", item);
        let results = orchestrator
            .search(&search_query, Some(args.max_sources as u32))
            .await
            .unwrap_or_default();

        let mut combined_text = String::new();
        let mut sources: Vec<String> = Vec::new();

        for result in results.iter().take(args.max_sources) {
            if let Ok(fetch_result) = http.fetch(&result.url).await {
                let extracted = pipeline::extract(&fetch_result.body, &fetch_result.url);
                let qatbe = extract_with_budget(&extracted, item, args.budget as u32);
                for seg in &qatbe.segments {
                    if let Some(text) = seg.content.as_str() {
                        combined_text.push_str(text);
                        combined_text.push('\n');
                    }
                }
                sources.push(result.url.clone());
            }
        }

        item_data.push((item.clone(), combined_text, sources));
    }

    spinner.set_message("Building comparison table...");

    let comparison = if args.ai {
        build_comparison_ai(&parsed, &item_data, config).await
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
