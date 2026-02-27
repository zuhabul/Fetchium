//! `fetchium transcribe` — Transcribe audio/video from any URL.

use crate::cli::TranscribeArgs;
use anyhow::Result;
use colored::Colorize;
use fetchium_core::config::HsxConfig;
use fetchium_core::http::client::HttpClient;
use std::time::Instant;

/// Run the `fetchium transcribe` subcommand.
pub async fn run(args: TranscribeArgs, config: &HsxConfig) -> Result<()> {
    let start = Instant::now();
    let http = HttpClient::new(config)?;

    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_message(format!("Transcribing: {}...", &args.url));
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    // Detect if YouTube URL
    let is_youtube = args.url.contains("youtube.com") || args.url.contains("youtu.be");

    if is_youtube {
        // Delegate to YouTube transcript system
        let yt_args = crate::cli::YtTranscriptArgs {
            url: args.url.clone(),
            chapters: args.chapters,
        };
        spinner.finish_and_clear();
        let yt_wrapper = crate::cli::YouTubeArgs {
            action: crate::cli::YouTubeAction::Transcript(yt_args),
        };
        crate::commands::youtube::run(yt_wrapper, config, crate::cli::Format::Markdown).await?;
    } else {
        // Generic URL: fetch and extract text content
        let html = http.fetch_text(&args.url).await?;
        let extracted = fetchium_core::extract::pipeline::extract(&html, &args.url);
        spinner.finish_and_clear();

        println!("\n{}", "Transcription".bold().cyan());
        println!("{}", "=".repeat(50).dimmed());
        println!("\n{}", extracted.text);
    }

    let elapsed = start.elapsed();
    eprintln!("\nCompleted in {:.1}s", elapsed.as_secs_f64());

    Ok(())
}
