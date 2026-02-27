//! `fetchium x` / `fetchium twitter` — X (Twitter) Intelligence (search, trends, sentiment, monitor, fetch).

use crate::cli::Format;
use anyhow::Result;
use colored::Colorize;
use fetchium_core::config::HsxConfig;
use fetchium_core::http::client::HttpClient;
use fetchium_core::social::twitter::{
    oembed, pipeline as twitter_pipeline, realtime, search, sentiment, trends,
    types::TwitterPipelineConfig,
};
use std::time::Instant;

/// Run the `fetchium twitter` subcommand.
pub async fn run(args: crate::cli::TwitterArgs, config: &HsxConfig, format: Format) -> Result<()> {
    let start = Instant::now();
    let http = HttpClient::new(config)?;

    match args.action {
        crate::cli::TwitterAction::Search { query, max } => {
            let spinner = make_spinner(&format!("Searching X (Twitter): {}...", query));

            let cfg = TwitterPipelineConfig {
                query: query.clone(),
                max_tweets: max,
                fetch_trends: false,
                searxng_url: config.search.searxng_url.clone(),
                ..Default::default()
            };

            let result = twitter_pipeline::run_twitter_pipeline(&cfg, config, &http).await;
            spinner.finish_and_clear();

            match result {
                Ok(r) => {
                    if matches!(format, Format::Json) {
                        println!("{}", serde_json::to_string_pretty(&r)?);
                    } else {
                        println!("{}", twitter_pipeline::format_markdown(&r));
                        if r.tweets.is_empty() {
                            println!(
                                "{}",
                                "Note: No tweets found. Nitter instances may be rate-limited."
                                    .yellow()
                            );
                        }
                    }
                }
                Err(e) => eprintln!("{} {e}", "X error:".red()),
            }
            println!("\nCompleted in {:.1}s", start.elapsed().as_secs_f64());
        }
        crate::cli::TwitterAction::Trends { country } => {
            let spinner = make_spinner(&format!("Fetching trends for {}...", country));
            let result = trends::fetch_trends(&country, &http).await;
            spinner.finish_and_clear();

            match result {
                Ok(trending) => {
                    println!("\n{}\n", "Trending on X (Twitter)".bold().cyan());
                    for trend in trending.iter().take(25) {
                        let volume = trend
                            .tweet_volume
                            .map(|v| format!(" ({} tweets)", v))
                            .unwrap_or_default();
                        println!("  {}. {}{}", trend.rank, trend.name.bold(), volume);
                    }
                    if trending.is_empty() {
                        println!("No trends found for '{}'.", country);
                    }
                }
                Err(e) => eprintln!("{} {e}", "Trends error:".red()),
            }
        }
        crate::cli::TwitterAction::Sentiment { query, max } => {
            let spinner = make_spinner(&format!("Analyzing sentiment: {}...", query));

            let cfg = TwitterPipelineConfig {
                query: query.clone(),
                max_tweets: max,
                searxng_url: config.search.searxng_url.clone(),
                ..Default::default()
            };

            let tweets = search::search_tweets(&query, max, &cfg, &http).await;
            spinner.finish_and_clear();

            match tweets {
                Ok(tweets) => {
                    let mut positive = 0usize;
                    let mut negative = 0usize;
                    let mut neutral = 0usize;
                    let mut total_score = 0.0f64;

                    for tweet in &tweets {
                        let s = sentiment::analyze_sentiment(&tweet.text);
                        total_score += s.score;
                        match s.label {
                            sentiment::Sentiment::Positive => positive += 1,
                            sentiment::Sentiment::Negative => negative += 1,
                            _ => neutral += 1,
                        }
                    }

                    let total = tweets.len().max(1);
                    let avg_score = total_score / total as f64;

                    println!("\n{}\n", "Sentiment Analysis".bold().cyan());
                    println!("Query: {}", query.bold());
                    println!("Tweets analyzed: {}", total);
                    println!("Average sentiment: {:.2}", avg_score);
                    println!(
                        "\n  {} Positive: {} ({:.0}%)",
                        "+".green(),
                        positive,
                        positive as f64 / total as f64 * 100.0
                    );
                    println!(
                        "  {} Negative: {} ({:.0}%)",
                        "-".red(),
                        negative,
                        negative as f64 / total as f64 * 100.0
                    );
                    println!(
                        "  {} Neutral:  {} ({:.0}%)",
                        "~".dimmed(),
                        neutral,
                        neutral as f64 / total as f64 * 100.0
                    );
                }
                Err(e) => eprintln!("{} {e}", "X error:".red()),
            }
        }
        crate::cli::TwitterAction::Fetch { url } => {
            let spinner = make_spinner("Fetching tweet...");
            let result = oembed::fetch_oembed(&url, &http).await;
            spinner.finish_and_clear();

            match result {
                Ok(tweet) => {
                    println!("\n{}", "Tweet".bold().cyan());
                    println!("@{}: {}", tweet.author.username.bold(), tweet.text);
                    println!("URL: {}", tweet.url);
                }
                Err(e) => eprintln!("{} {e}", "Fetch error:".red()),
            }
        }
        crate::cli::TwitterAction::Monitor { query, interval } => {
            println!(
                "{}",
                format!(
                    "Monitoring X (Twitter) for: {} (every {}s)",
                    query, interval
                )
                .cyan()
            );
            println!("Press Ctrl+C to stop.\n");

            let rt_config = realtime::RealtimeConfig {
                query,
                interval_secs: interval,
                sentiment: true,
                max_per_poll: 20,
            };
            let mut rx = realtime::monitor_stream(rt_config, http).await;

            while let Some(update) = rx.recv().await {
                let sentiment_str = update
                    .sentiment
                    .map(|s| format!(" [{}]", s.label))
                    .unwrap_or_default();
                println!(
                    "[{}] @{}: {}{}",
                    chrono::Utc::now().format("%H:%M:%S"),
                    update.tweet.author.username,
                    update.tweet.text.chars().take(150).collect::<String>(),
                    sentiment_str,
                );
            }
        }
        crate::cli::TwitterAction::Research { query, max } => {
            // Delegate to social module with twitter-only flag
            let social_args = crate::cli::SocialArgs {
                query,
                extra_query: None,
                unified: false,
                twitter: true,
                reddit: false,
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
        crate::cli::TwitterAction::Profile { username } => {
            let spinner = make_spinner(&format!("Fetching profile: @{}...", username));

            let cfg = TwitterPipelineConfig {
                query: format!("from:{}", username),
                max_tweets: 5,
                searxng_url: config.search.searxng_url.clone(),
                ..Default::default()
            };

            let tweets = search::search_tweets(&format!("from:{}", username), 5, &cfg, &http).await;
            spinner.finish_and_clear();

            match tweets {
                Ok(tweets) => {
                    println!("\n{}", format!("@{}", username).bold().cyan());
                    println!("{}", "=".repeat(50).dimmed());
                    if let Some(tweet) = tweets.first() {
                        println!("Display name: {}", tweet.author.display_name);
                        if let Some(f) = tweet.author.followers {
                            println!("Followers: {}", f);
                        }
                        println!("\nRecent tweets:");
                        for (i, t) in tweets.iter().enumerate().take(5) {
                            println!(
                                "  {}. {}",
                                i + 1,
                                t.text.chars().take(120).collect::<String>()
                            );
                        }
                    } else {
                        println!("No tweets found for @{}", username);
                    }
                }
                Err(e) => eprintln!("{} {e}", "X error:".red()),
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
