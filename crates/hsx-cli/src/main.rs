//! HyperSearchX CLI — entry point.

mod cli;
mod commands;
mod output;

use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let filter = if cli.verbose {
        "hsx=debug,hsx_core=debug"
    } else if cli.quiet {
        "hsx=error,hsx_core=error"
    } else {
        "hsx=info,hsx_core=warn"
    };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| filter.into()),
        )
        .with_target(false)
        .init();

    // Load config
    let mut config = match &cli.config {
        Some(path) => hsx_core::config::HsxConfig::load_from(Some(std::path::Path::new(path))),
        None => hsx_core::config::HsxConfig::load(),
    };

    // Apply CLI flag overrides
    if cli.no_cache {
        config.cache.enabled = false;
    }
    if cli.verbose {
        config.general.verbose = true;
    }

    // Dispatch command
    match cli.command {
        Commands::Search(args) => commands::search::run(args, &config).await,
        Commands::Fetch(args) | Commands::View(args) => commands::fetch::run(args, &config).await,
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
    }
}
