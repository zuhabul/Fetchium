//! `hsx ai` — AI-powered synthesis using local Ollama LLM (Mode D, PRD §10).

use crate::cli::AiArgs;
use hsx_core::ai::{run_ai_pipeline, AiConfig};
use hsx_core::config::HsxConfig;
use hsx_core::http::client::HttpClient;
use indicatif::{ProgressBar, ProgressStyle};
use console::style;
use std::time::Duration;

pub async fn run(args: AiArgs, config: &HsxConfig) -> anyhow::Result<()> {
    let ai_config = AiConfig::from_hsx_config(config);
    let http_client = HttpClient::new(config)?;
    let streaming = !args.no_stream;

    // Show spinner during search + extraction phase (not during streaming)
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message(format!("Searching: {}...", &args.query));
    spinner.enable_steady_tick(Duration::from_millis(80));

    if streaming {
        // Clear spinner before streaming begins so tokens display cleanly
        spinner.finish_and_clear();
        println!("{}", style("─".repeat(60)).dim());
        println!("{} {}", style("AI Answer:").bold().cyan(), style(&args.query).dim());
        println!("{}", style("─".repeat(60)).dim());
    }

    let result = run_ai_pipeline(
        &args.query,
        args.model.as_deref(),
        args.budget,
        args.max_sources,
        streaming,
        &ai_config,
        config,
        &http_client,
    )
    .await?;

    if !streaming {
        spinner.finish_and_clear();
        println!("{}", style("─".repeat(60)).dim());
        println!("{} {}", style("AI Answer:").bold().cyan(), style(&args.query).dim());
        println!("{}", style("─".repeat(60)).dim());
        println!("{}", result.answer);
    } else if !result.fallback {
        // Add newline after streaming output
        println!();
    }

    println!("{}", style("─".repeat(60)).dim());
    println!(
        "{} {} │ {} {} │ {} {}",
        style("Model:").dim(),   style(&result.model_used).cyan(),
        style("Sources:").dim(), style(result.sources_used).cyan(),
        style("Fallback:").dim(), style(if result.fallback { "yes" } else { "no" }).cyan(),
    );

    if result.fallback {
        eprintln!(
            "\n{} Install Ollama (https://ollama.ai) and run `ollama pull deepseek-r1:7b` for AI synthesis.",
            style("Tip:").yellow().bold()
        );
    }

    Ok(())
}
