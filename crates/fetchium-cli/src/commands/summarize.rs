//! `fetchium summarize` — AI-powered URL/text summarization.

use crate::cli::SummarizeArgs;
use anyhow::Result;
use colored::Colorize;
use fetchium_core::config::HsxConfig;
use fetchium_core::summarize::{SummarizeConfig, SummaryLength};
use std::time::Instant;

/// Run the `fetchium summarize` subcommand.
pub async fn run(args: SummarizeArgs, config: &HsxConfig) -> Result<()> {
    let start = Instant::now();

    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_message("Summarizing...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let length = match args.length.as_deref() {
        Some("short") | Some("s") => SummaryLength::Short,
        Some("long") | Some("l") => SummaryLength::Long,
        _ => SummaryLength::Medium,
    };

    let summarize_config = SummarizeConfig {
        length,
        model: None,
    };

    let result = fetchium_core::summarize::summarize(&args.input, &summarize_config, config).await;
    spinner.finish_and_clear();

    match result {
        Ok(summary) => {
            if summary.summary.is_empty() {
                eprintln!("No content to summarize.");
                return Ok(());
            }

            let header = if summary.ai_used {
                "Summary"
            } else {
                "Summary (heuristic)"
            };
            println!("\n{}", header.bold().cyan());
            println!("{}\n", "=".repeat(50).dimmed());

            if let Some(title) = &summary.source_title {
                println!("{}: {}\n", "Source".bold(), title);
            }

            println!("{}", summary.summary);
        }
        Err(e) => {
            eprintln!("Summarization failed: {e}");
        }
    }

    let elapsed = start.elapsed();
    eprintln!("\nCompleted in {:.1}s", elapsed.as_secs_f64());

    Ok(())
}
