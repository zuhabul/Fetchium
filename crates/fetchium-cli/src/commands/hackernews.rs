//! `fetchium hackernews` — Hacker News Intelligence (search, top, new, fetch).

use crate::cli::{Format, HackernewsArgs};
use anyhow::Result;
use colored::Colorize;
use fetchium_core::config::HsxConfig;
use fetchium_core::http::client::HttpClient;
use fetchium_core::social::hackernews;
use std::time::Instant;

/// Run the `fetchium hackernews` subcommand.
pub async fn run(args: HackernewsArgs, config: &HsxConfig, format: Format) -> Result<()> {
    let start = Instant::now();
    let http = HttpClient::new(config)?;

    match args.action {
        crate::cli::HackernewsAction::Search { query, max } => {
            let spinner = make_spinner(&format!("Searching HN: {}...", query));
            let stories = hackernews::search_stories(&query, max, &http, 15).await;
            spinner.finish_and_clear();

            match stories {
                Ok(stories) => {
                    if matches!(format, Format::Json) {
                        println!("{}", serde_json::to_string_pretty(&stories)?);
                    } else {
                        println!(
                            "\n{} — {} results\n",
                            "Hacker News Search".bold().yellow(),
                            stories.len()
                        );
                        for story in &stories {
                            let url = story
                                .url
                                .as_deref()
                                .unwrap_or("https://news.ycombinator.com");
                            println!(
                                "  {} {} | {}pts | {}c | by {}",
                                "▸".cyan(),
                                story.title.bold(),
                                story.score,
                                story.descendants,
                                story.author
                            );
                            println!("    {}", url.dimmed());
                        }
                        if stories.is_empty() {
                            println!("No results found for '{}'.", query);
                        }
                    }
                }
                Err(e) => eprintln!("{} {e}", "HackerNews error:".red()),
            }
            println!("\nCompleted in {:.1}s", start.elapsed().as_secs_f64());
        }
        crate::cli::HackernewsAction::Research { query, max } => {
            let social_args = crate::cli::SocialArgs {
                query,
                extra_query: None,
                unified: false,
                twitter: false,
                reddit: false,
                tiktok: false,
                hackernews: true,
                facebook: false,
                youtube: false,
                max,
                subreddits: vec![],
                trends: false,
                ideas: false,
                deep: true,
                token: None,
                output: None,
            };
            crate::commands::social::run(social_args, config, format).await?;
        }
        crate::cli::HackernewsAction::Top { max } => {
            let spinner = make_spinner("Fetching HN top stories...");
            let url = "https://hacker-news.firebaseio.com/v0/topstories.json";
            let body = http.fetch_text(url).await?;
            spinner.finish_and_clear();

            println!("\n{}\n", "Hacker News — Top Stories".bold().yellow());

            if let Ok(ids) = serde_json::from_str::<Vec<u64>>(&body) {
                let mut handles = Vec::new();
                for id in ids.iter().take(max) {
                    let item_url =
                        format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);
                    let client = http.clone();
                    handles.push(tokio::spawn(async move {
                        client.fetch_text(&item_url).await.ok()
                    }));
                }
                for (i, handle) in handles.into_iter().enumerate() {
                    if let Ok(Some(item_body)) = handle.await {
                        if let Ok(item) = serde_json::from_str::<serde_json::Value>(&item_body) {
                            let title = item["title"].as_str().unwrap_or("Untitled");
                            let score = item["score"].as_i64().unwrap_or(0);
                            let descendants = item["descendants"].as_i64().unwrap_or(0);
                            println!(
                                "{}. {} ({}pts, {}c)",
                                i + 1,
                                title.bold(),
                                score,
                                descendants
                            );
                        }
                    }
                }
            }
            println!("\nCompleted in {:.1}s", start.elapsed().as_secs_f64());
        }
        crate::cli::HackernewsAction::New { max } => {
            let spinner = make_spinner("Fetching HN new stories...");
            let url = "https://hacker-news.firebaseio.com/v0/newstories.json";
            let body = http.fetch_text(url).await?;
            spinner.finish_and_clear();

            println!("\n{}\n", "Hacker News — Newest Stories".bold().yellow());

            if let Ok(ids) = serde_json::from_str::<Vec<u64>>(&body) {
                let mut handles = Vec::new();
                for id in ids.iter().take(max) {
                    let item_url =
                        format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);
                    let client = http.clone();
                    handles.push(tokio::spawn(async move {
                        client.fetch_text(&item_url).await.ok()
                    }));
                }
                for (i, handle) in handles.into_iter().enumerate() {
                    if let Ok(Some(item_body)) = handle.await {
                        if let Ok(item) = serde_json::from_str::<serde_json::Value>(&item_body) {
                            let title = item["title"].as_str().unwrap_or("Untitled");
                            let score = item["score"].as_i64().unwrap_or(0);
                            println!("{}. {} ({}pts)", i + 1, title, score);
                        }
                    }
                }
            }
            println!("\nCompleted in {:.1}s", start.elapsed().as_secs_f64());
        }
        crate::cli::HackernewsAction::Fetch { url } => {
            let spinner = make_spinner("Fetching HN story...");
            // Extract item ID from HN URL or use raw ID
            let id = url.split("id=").last().unwrap_or(&url).trim();
            let api_url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);
            let body = http.fetch_text(&api_url).await?;
            spinner.finish_and_clear();

            if let Ok(item) = serde_json::from_str::<serde_json::Value>(&body) {
                println!(
                    "\n{}",
                    item["title"].as_str().unwrap_or("Story").bold().cyan()
                );
                if let Some(text) = item["text"].as_str() {
                    println!("\n{}", text);
                }
                if let Some(story_url) = item["url"].as_str() {
                    println!("\nURL: {}", story_url);
                }
                println!("\n---");
                println!(
                    "Score: {} | Comments: {} | By: {}",
                    item["score"].as_i64().unwrap_or(0),
                    item["descendants"].as_i64().unwrap_or(0),
                    item["by"].as_str().unwrap_or("unknown")
                );
            }
        }
    }

    Ok(())
}

fn make_spinner(msg: &str) -> indicatif::ProgressBar {
    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_message(msg.to_string());
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));
    spinner
}
