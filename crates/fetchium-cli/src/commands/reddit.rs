//! `fetchium reddit` — Reddit Intelligence (search, hot, top, research, fetch).

use crate::cli::{Format, RedditArgs};
use anyhow::Result;
use colored::Colorize;
use fetchium_core::config::HsxConfig;
use fetchium_core::http::client::HttpClient;
use std::time::Instant;

/// Run the `fetchium reddit` subcommand.
pub async fn run(args: RedditArgs, config: &HsxConfig, format: Format) -> Result<()> {
    let start = Instant::now();
    let http = HttpClient::new(config)?;

    match args.action {
        crate::cli::RedditAction::Search {
            query,
            max,
            subreddits,
        } => {
            let spinner = make_spinner(&format!("Searching Reddit: {}...", query));
            let social_args = crate::cli::SocialArgs {
                query,
                unified: false,
                twitter: false,
                reddit: true,
                tiktok: false,
                hackernews: false,
                facebook: false,
                youtube: false,
                max,
                subreddits,
                trends: false,
                ideas: false,
                deep: false,
                token: None,
                output: None,
            };
            spinner.finish_and_clear();
            crate::commands::social::run(social_args, config, format).await?;
        }
        crate::cli::RedditAction::Research { query, max } => {
            let social_args = crate::cli::SocialArgs {
                query,
                unified: false,
                twitter: false,
                reddit: true,
                tiktok: false,
                hackernews: false,
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
        crate::cli::RedditAction::Hot { subreddit, max } => {
            let spinner = make_spinner(&format!("Fetching r/{} hot posts...", subreddit));
            let url = format!(
                "https://www.reddit.com/r/{}/hot.json?limit={}&raw_json=1",
                subreddit, max
            );
            let body = http.fetch_text(&url).await?;
            spinner.finish_and_clear();

            println!(
                "\n{}\n",
                format!("r/{} — Hot Posts", subreddit).bold().cyan()
            );

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(children) = json["data"]["children"].as_array() {
                    for (i, child) in children.iter().enumerate().take(max) {
                        let data = &child["data"];
                        let title = data["title"].as_str().unwrap_or("Untitled");
                        let score = data["score"].as_i64().unwrap_or(0);
                        let comments = data["num_comments"].as_i64().unwrap_or(0);
                        let author = data["author"].as_str().unwrap_or("deleted");
                        println!(
                            "{}. {} ({}pts, {}c) — u/{}",
                            i + 1,
                            title.bold(),
                            score,
                            comments,
                            author,
                        );
                    }
                }
            }
            println!("\nCompleted in {:.1}s", start.elapsed().as_secs_f64());
        }
        crate::cli::RedditAction::Top {
            subreddit,
            period,
            max,
        } => {
            let spinner = make_spinner(&format!("Fetching r/{} top ({})...", subreddit, period));
            let url = format!(
                "https://www.reddit.com/r/{}/top.json?t={}&limit={}&raw_json=1",
                subreddit, period, max
            );
            let body = http.fetch_text(&url).await?;
            spinner.finish_and_clear();

            println!(
                "\n{}\n",
                format!("r/{} — Top ({})", subreddit, period).bold().cyan()
            );

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(children) = json["data"]["children"].as_array() {
                    for (i, child) in children.iter().enumerate().take(max) {
                        let data = &child["data"];
                        let title = data["title"].as_str().unwrap_or("Untitled");
                        let score = data["score"].as_i64().unwrap_or(0);
                        let comments = data["num_comments"].as_i64().unwrap_or(0);
                        println!("{}. {} ({}pts, {}c)", i + 1, title.bold(), score, comments);
                    }
                }
            }
            println!("\nCompleted in {:.1}s", start.elapsed().as_secs_f64());
        }
        crate::cli::RedditAction::Fetch { url } => {
            let spinner = make_spinner("Fetching Reddit post...");
            let json_url = if url.ends_with(".json") {
                url.clone()
            } else {
                format!("{}.json", url)
            };
            let body = http.fetch_text(&json_url).await?;
            spinner.finish_and_clear();

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(arr) = json.as_array() {
                    if let Some(post) = arr.first() {
                        if let Some(data) = post["data"]["children"][0]["data"].as_object() {
                            println!(
                                "\n{}",
                                data.get("title")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Post")
                                    .bold()
                                    .cyan()
                            );
                            if let Some(text) = data.get("selftext").and_then(|v| v.as_str()) {
                                if !text.is_empty() {
                                    println!("\n{}", text);
                                }
                            }
                            println!("\n---");
                            println!(
                                "Score: {} | Comments: {}",
                                data.get("score").and_then(|v| v.as_i64()).unwrap_or(0),
                                data.get("num_comments")
                                    .and_then(|v| v.as_i64())
                                    .unwrap_or(0)
                            );
                        }
                    }
                }
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
