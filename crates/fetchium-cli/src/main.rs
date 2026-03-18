//! Fetchium CLI — entry point.

mod cli;
mod commands;
mod output;
mod tui;

use clap::Parser;
use cli::{Cli, Commands};
use std::time::Instant;

#[tokio::main(worker_threads = 4)]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let filter = if cli.verbose {
        "fetchium=debug,fetchium_core=debug"
    } else if cli.quiet {
        "fetchium=error,fetchium_core=error"
    } else {
        "fetchium=info,fetchium_core=warn"
    };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| filter.into()),
        )
        .with_target(false)
        .with_writer(std::io::stderr) // logs → stderr so stdout stays clean for JSON/piping
        .init();

    // Load config
    let mut config = match &cli.config {
        Some(path) => {
            fetchium_core::config::FetchiumConfig::load_from(Some(std::path::Path::new(path)))
        }
        None => fetchium_core::config::FetchiumConfig::load(),
    };

    // Apply CLI flag overrides
    if cli.no_cache {
        config.cache.enabled = false;
    }
    if cli.verbose {
        config.general.verbose = true;
    }

    // Capture global flags before moving cli
    let format = cli.format;
    let quiet = cli.quiet;
    let show_timing = cli.time;
    let cmd_timer = Instant::now();

    // Dispatch command
    let result = match cli.command {
        Commands::Search(args) => commands::search::run(args, &config, format, quiet).await,
        Commands::Fetch(args) | Commands::View(args) => {
            commands::fetch::run(args, &config, format).await
        }
        Commands::Research(args) => commands::research::run(args, &config).await,
        Commands::Ai(args) => commands::ai::run(args, &config).await,
        Commands::Deep(args) => commands::deep::run(args, &config).await,
        Commands::AgentSearch(args) => commands::agent_search::run(args, &config).await,
        Commands::AgentFetch(args) => commands::agent_fetch::run(args, &config).await,
        Commands::AgentResearch(args) => commands::agent_research::run(args, &config).await,
        Commands::Doctor => commands::doctor::run(&config).await,
        Commands::Config(args) => commands::config::run(args, &config).await,
        Commands::Cache(args) => commands::cache::run(args, &config).await,
        Commands::Serve(args) => commands::serve::run(args, &config).await,
        Commands::Compare(args) => commands::compare::run(args, &config, format).await,
        Commands::Monitor(args) => commands::monitor::run(args, &config).await,
        Commands::Index(args) => commands::index::run(args, &config, format).await,
        Commands::Intelligence { sub } => commands::intelligence::run(&config, sub)
            .await
            .map_err(|e| anyhow::anyhow!("{e}")),
        Commands::Plugin { sub } => commands::plugin::run(sub),
        Commands::Workspace { sub } => commands::workspace::run(sub),
        Commands::Subscribe { sub } => commands::subscribe::run(sub),
        Commands::Radar(args) => commands::radar::run(args.limit),
        Commands::Digest(args) => {
            commands::digest::run(&args.period, &args.topics, args.output.as_deref())
        }
        Commands::Tui => tui::run_tui(),
        Commands::Completions { shell } => {
            commands::completions::run(shell);
            Ok(())
        }
        Commands::YouTube(args) => commands::youtube::run(args, &config, format).await,
        Commands::Social(args) => commands::social::run(args, &config, format).await,
        Commands::Provider { action } => commands::provider::run(action, &config).await,
        Commands::Setup(args) => commands::setup::run(args, &config).await,
        Commands::Twitter(args) => commands::twitter::run(args, &config, format).await,
        Commands::Reddit(args) => commands::reddit::run(args, &config, format).await,
        Commands::Hackernews(args) => commands::hackernews::run(args, &config, format).await,
        Commands::Facebook(args) => commands::facebook::run(args, &config, format).await,
        Commands::Tiktok(args) => commands::tiktok::run(args, &config, format).await,
        Commands::Transcribe(args) => commands::transcribe::run(args, &config).await,
        Commands::Summarize(args) => commands::summarize::run(args, &config).await,
    };

    // Global timing footer (shown for all commands when --time flag is set)
    if show_timing {
        let elapsed = cmd_timer.elapsed();
        let ms = elapsed.as_millis();
        eprintln!("\n⏱  {}ms", ms);
    }

    result
}
