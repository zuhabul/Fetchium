//! `fetchium facebook` — Facebook Intelligence (search, fetch).

use crate::cli::{FacebookArgs, Format};
use anyhow::Result;
use colored::Colorize;
use fetchium_core::config::FetchiumConfig;
use fetchium_core::http::client::HttpClient;
use std::time::Instant;

/// Run the `fetchium facebook` subcommand.
pub async fn run(args: FacebookArgs, config: &FetchiumConfig, format: Format) -> Result<()> {
    let start = Instant::now();

    match args.action {
        crate::cli::FacebookAction::Search { query, max } => {
            let social_args = crate::cli::SocialArgs {
                query,
                extra_query: None,
                unified: false,
                twitter: false,
                reddit: false,
                tiktok: false,
                hackernews: false,
                facebook: true,
                youtube: false,
                max,
                subreddits: vec![],
                trends: false,
                ideas: false,
                deep: false,
                token: None,
                output: None,
            };
            crate::commands::social::run(social_args, config, format).await?;
        }
        crate::cli::FacebookAction::Fetch { url } => {
            let http = HttpClient::new(config)?;
            let spinner = indicatif::ProgressBar::new_spinner();
            spinner.set_message("Fetching Facebook page...");
            spinner.enable_steady_tick(std::time::Duration::from_millis(80));
            let html = http.fetch_text(&url).await?;
            spinner.finish_and_clear();

            let extracted = fetchium_core::extract::pipeline::extract(&html, &url);
            println!("\n{}", extracted.title.bold().cyan());
            println!("{}", extracted.text.chars().take(2000).collect::<String>());
            println!("\nCompleted in {:.1}s", start.elapsed().as_secs_f64());
        }
    }

    Ok(())
}
