//! `hsx ai` — AI-powered synthesis using configured provider (Mode D, PRD §10).

use crate::cli::AiArgs;
use console::style;
use hsx_core::ai::{best_model_name, run_ai_pipeline, AiConfig, DeviceSpec};
use hsx_core::config::HsxConfig;
use hsx_core::http::client::HttpClient;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub async fn run(args: AiArgs, config: &HsxConfig) -> anyhow::Result<()> {
    let ai_config = AiConfig::from_hsx_config(config);
    let http_client = HttpClient::new(config)?;
    let streaming = !args.no_stream;

    // Show the configured provider chain in verbose mode
    let chain = ai_config.providers.resolved_chain();
    let provider_hint = if chain.len() == 1 {
        chain[0].display_name().to_string()
    } else {
        let names: Vec<_> = chain.iter().map(|k| k.display_name()).collect();
        format!("{} (fallback chain)", names.join(" → "))
    };

    // Spinner during search + extraction (not shown during streaming output)
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message(format!("Searching via {} ...", provider_hint));
    spinner.enable_steady_tick(Duration::from_millis(80));

    if streaming {
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
        // Show multi-provider setup guidance
        let spec = DeviceSpec::detect();
        let best_ollama = best_model_name(spec.usable_ram_gb);
        eprintln!(
            "\n{} No AI provider available. Quick options:",
            style("Setup:").yellow().bold()
        );
        eprintln!();
        eprintln!(
            "  {} {} (free, fast, needs API key)",
            style("Option 1 — Google Gemini:").bold(),
            style("gemini-2.0-flash").green()
        );
        eprintln!("    1. Get key: https://aistudio.google.com/app/apikey");
        eprintln!("    2. Run:     hsx provider setup gemini");
        eprintln!();
        eprintln!(
            "  {} {} (local, private, no API key)",
            style("Option 2 — Ollama:").bold(),
            style(best_ollama).green()
        );
        eprintln!("    1. Install: curl -fsSL https://ollama.ai/install.sh | sh");
        eprintln!("    2. Run:     ollama serve && ollama pull {best_ollama}");
        eprintln!("    3. Run:     hsx provider setup ollama");
        eprintln!();
        eprintln!("  See all options: {}", style("hsx provider list").cyan());
        eprintln!("  Full guide:      {}", style("hsx doctor").cyan());
    }

    Ok(())
}
