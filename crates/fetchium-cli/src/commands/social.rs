//! Social Media Intelligence System CLI command handler.
//!
//! All platforms use a single flat interface:
//!
//! ```text
//! hsx social "query"                    # unified (all platforms, default)
//! hsx social "query" --twitter          # Twitter/X only
//! hsx social "query" --reddit           # Reddit only
//! hsx social "query" --tiktok           # TikTok only
//! hsx social "query" --hackernews       # Hacker News only
//! hsx social "query" --facebook         # Facebook only
//! hsx social "query" --unified --ideas  # Unified + content ideas
//! hsx social "query" --reddit --subreddits r/ML,r/AI  # Reddit subreddits
//! ```

use anyhow::Result;
use colored::Colorize;
use fetchium_core::config::HsxConfig;
use fetchium_core::http::client::HttpClient;
use fetchium_core::social::{
    facebook::{analysis as fb_analysis, pipeline as fb_pipeline, types::FacebookPipelineConfig},
    hackernews,
    reddit::{pipeline as reddit_pipeline, types::RedditPipelineConfig},
    tiktok::{pipeline as tiktok_pipeline, types::TikTokPipelineConfig},
    twitter::{pipeline as twitter_pipeline, types::TwitterPipelineConfig},
    types::{SocialPipelineConfig, SocialPlatform},
    unified::engine::{format_markdown as unified_format, run_social_pipeline},
};

use crate::cli::{Format, SocialArgs};

/// Run the `hsx social` command.
pub async fn run(args: SocialArgs, config: &HsxConfig, format: Format) -> Result<()> {
    let http = HttpClient::new(config)?;

    // Determine which specific platforms were requested.
    let any_specific = args.twitter
        || args.reddit
        || args.tiktok
        || args.hackernews
        || args.facebook
        || args.youtube;

    // Unified mode: either explicitly requested or nothing specific chosen.
    if args.unified || !any_specific {
        let platforms = build_platform_list(&args);
        return run_unified(&args, platforms, config, &http, format).await;
    }

    // Individual platform mode — run each selected platform.
    if args.twitter {
        run_twitter(
            args.query.clone(),
            args.max,
            args.trends,
            config,
            &http,
            format,
        )
        .await?;
    }
    if args.reddit {
        run_reddit(
            args.query.clone(),
            args.subreddits.clone(),
            args.max,
            config,
            &http,
            format,
        )
        .await?;
    }
    if args.tiktok {
        run_tiktok(args.query.clone(), args.max, config, &http, format).await?;
    }
    if args.hackernews {
        run_hackernews(args.query.clone(), args.max, config, &http, format).await?;
    }
    if args.facebook {
        run_facebook(
            args.query.clone(),
            args.max,
            args.token.clone(),
            config,
            &http,
            format,
        )
        .await?;
    }
    if args.youtube {
        run_youtube_social(
            args.query.clone(),
            args.max,
            args.deep,
            config,
            &http,
            format,
        )
        .await?;
    }

    Ok(())
}

// ─── Twitter ──────────────────────────────────────────────────────

async fn run_twitter(
    query: String,
    max: usize,
    trends: bool,
    config: &HsxConfig,
    http: &HttpClient,
    format: Format,
) -> Result<()> {
    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_message(format!("Fetching Twitter/X data for '{query}'..."));
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let cfg = TwitterPipelineConfig {
        query: query.clone(),
        max_tweets: max,
        fetch_trends: trends,
        searxng_url: config.search.searxng_url.clone(),
        ..Default::default()
    };

    let result = twitter_pipeline::run_twitter_pipeline(&cfg, config, http).await;
    spinner.finish_and_clear();

    match result {
        Ok(r) => {
            if matches!(format, Format::Json) {
                println!("{}", serde_json::to_string_pretty(&r)?);
            } else {
                let md = twitter_pipeline::format_markdown(&r);
                println!("{md}");
                if r.tweets.is_empty() {
                    println!(
                        "{}",
                        "Note: No tweets found. Nitter instances may be rate-limited — \
                         this is a known limitation of free Twitter/X scraping."
                            .yellow()
                    );
                }
            }
        }
        Err(e) => eprintln!("{} {e}", "Twitter error:".red()),
    }
    Ok(())
}

// ─── Reddit ───────────────────────────────────────────────────────

async fn run_reddit(
    query: String,
    subreddits: Vec<String>,
    max: usize,
    config: &HsxConfig,
    http: &HttpClient,
    format: Format,
) -> Result<()> {
    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_message(format!("Fetching Reddit data for '{query}'..."));
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let cfg = RedditPipelineConfig {
        query: query.clone(),
        subreddits,
        max_posts: max,
        ..Default::default()
    };

    let result = reddit_pipeline::run_reddit_pipeline(&cfg, config, http).await;
    spinner.finish_and_clear();

    match result {
        Ok(r) => {
            if matches!(format, Format::Json) {
                println!("{}", serde_json::to_string_pretty(&r)?);
            } else {
                println!("{}", reddit_pipeline::format_markdown(&r));
            }
        }
        Err(e) => eprintln!("{} {e}", "Reddit error:".red()),
    }
    Ok(())
}

// ─── TikTok ───────────────────────────────────────────────────────

async fn run_tiktok(
    query: String,
    max: usize,
    config: &HsxConfig,
    http: &HttpClient,
    format: Format,
) -> Result<()> {
    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_message(format!("Fetching TikTok data for '{query}'..."));
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let cfg = TikTokPipelineConfig {
        query: query.clone(),
        max_videos: max,
        fetch_trends: true,
        ..Default::default()
    };

    let result = tiktok_pipeline::run_tiktok_pipeline(&cfg, config, http).await;
    spinner.finish_and_clear();

    match result {
        Ok(r) => {
            if matches!(format, Format::Json) {
                println!("{}", serde_json::to_string_pretty(&r)?);
            } else {
                println!("{}", tiktok_pipeline::format_markdown(&r));
            }
        }
        Err(e) => eprintln!("{} {e}", "TikTok error:".red()),
    }
    Ok(())
}

// ─── HackerNews ───────────────────────────────────────────────────

async fn run_hackernews(
    query: String,
    max: usize,
    _config: &HsxConfig,
    http: &HttpClient,
    format: Format,
) -> Result<()> {
    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_message(format!("Searching Hacker News for '{query}'..."));
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let stories = hackernews::search_stories(&query, max, http, 15).await;
    spinner.finish_and_clear();

    match stories {
        Ok(stories) => {
            if matches!(format, Format::Json) {
                println!("{}", serde_json::to_string_pretty(&stories)?);
            } else {
                println!(
                    "{} — {} results\n",
                    "Hacker News".bold().yellow(),
                    stories.len()
                );
                for story in &stories {
                    let url = story
                        .url
                        .as_deref()
                        .unwrap_or("https://news.ycombinator.com");
                    println!(
                        "  {} {} | ↑{} | 💬{} | by {}",
                        "▸".cyan(),
                        story.title.bold(),
                        story.score,
                        story.descendants,
                        story.author
                    );
                    println!("    {}", url.dimmed());
                }
                if stories.is_empty() {
                    println!("No results found for '{query}'.");
                }
            }
        }
        Err(e) => eprintln!("{} {e}", "HackerNews error:".red()),
    }
    Ok(())
}

// ─── YouTube (social mode) ────────────────────────────────────────

async fn run_youtube_social(
    query: String,
    max: usize,
    deep: bool,
    config: &HsxConfig,
    http: &HttpClient,
    format: Format,
) -> Result<()> {
    use fetchium_core::youtube::{pipeline as yt_pipeline, types::YouTubePipelineConfig};

    let spinner = indicatif::ProgressBar::new_spinner();
    if deep {
        spinner.set_message(format!(
            "Searching YouTube (deep mode: fetching transcripts) for '{query}'..."
        ));
    } else {
        spinner.set_message(format!("Searching YouTube for '{query}'..."));
    }
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let cfg = YouTubePipelineConfig {
        query: query.clone(),
        max_videos: max,
        fetch_transcript: deep, // enable transcripts in deep mode
        fetch_comments: false,
        fact_check: false,
        ..Default::default()
    };

    let result = yt_pipeline::run_youtube_pipeline(&cfg, config, http).await;
    spinner.finish_and_clear();

    match result {
        Ok(r) => {
            if matches!(format, Format::Json) {
                println!("{}", serde_json::to_string_pretty(&r)?);
            } else {
                println!(
                    "{} — {} results\n",
                    "YouTube".bold().red(),
                    r.rankings.len()
                );
                for (ranking, analysis) in r.rankings.iter().take(max).zip(r.videos.iter()) {
                    println!(
                        "  {} {} (score: {:.2})",
                        "▸".cyan(),
                        ranking.title.bold(),
                        ranking.final_score
                    );
                    println!(
                        "    https://youtube.com/watch?v={}",
                        ranking.video_id.dimmed()
                    );
                    // Show channel name when available
                    if !analysis.metadata.channel.name.is_empty() {
                        println!("    Channel: {}", analysis.metadata.channel.name.dimmed());
                    }
                    // Show transcript excerpt in deep mode
                    if deep {
                        if let Some(ref t) = analysis.transcript {
                            let excerpt: String = t.full_text.chars().take(200).collect();
                            if !excerpt.is_empty() {
                                println!("    Transcript: {}...", excerpt.dimmed());
                            }
                            if !t.key_moments.is_empty() {
                                println!(
                                    "    Key moments: {}",
                                    t.key_moments
                                        .iter()
                                        .take(3)
                                        .map(|m| m.text.chars().take(60).collect::<String>())
                                        .collect::<Vec<_>>()
                                        .join(" | ")
                                        .dimmed()
                                );
                            }
                        }
                    }
                }
                if r.rankings.is_empty() {
                    println!("No results found for '{query}'.");
                }
            }
        }
        Err(e) => eprintln!("{} {e}", "YouTube error:".red()),
    }
    Ok(())
}

// ─── Facebook ─────────────────────────────────────────────────────

async fn run_facebook(
    query: String,
    max: usize,
    token: Option<String>,
    config: &HsxConfig,
    http: &HttpClient,
    format: Format,
) -> Result<()> {
    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_message(format!("Fetching Facebook data for '{query}'..."));
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let graph_token = token.or_else(|| config.facebook_graph_token());

    let cfg = FacebookPipelineConfig {
        query: query.clone(),
        max_results: max,
        graph_api_token: graph_token.clone(),
        timeout_secs: 20,
        searxng_url: config.search.searxng_url.clone(),
    };

    let result = fb_pipeline::run_facebook_pipeline(&cfg, http).await;
    spinner.finish_and_clear();

    match result {
        Ok(r) => {
            if matches!(format, Format::Json) {
                println!("{}", serde_json::to_string_pretty(&r)?);
            } else {
                let md = fb_analysis::format_markdown(&r);
                println!("{md}");

                if r.posts.is_empty() && r.pages.is_empty() && graph_token.is_none() {
                    eprintln!(
                        "\n{} For richer Facebook data, set up a Graph API token:\n  \
                        1. Visit {} and create a free app\n  \
                        2. Pass: {}",
                        "Tip:".yellow().bold(),
                        "https://developers.facebook.com".cyan(),
                        "--token APP_ID|APP_SECRET".yellow()
                    );
                }
            }
        }
        Err(e) => eprintln!("{} {e}", "Facebook error:".red()),
    }
    Ok(())
}

// ─── Unified ──────────────────────────────────────────────────────

async fn run_unified(
    args: &SocialArgs,
    platforms: Vec<SocialPlatform>,
    config: &HsxConfig,
    http: &HttpClient,
    format: Format,
) -> Result<()> {
    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_message(format!(
        "Running unified social research for '{}'...",
        args.query
    ));
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let cfg = SocialPipelineConfig {
        query: args.query.clone(),
        platforms,
        max_posts_per_platform: args.max,
        include_trends: true, // always include trends in unified mode
        generate_ideas: args.ideas,
        deep_analysis: args.deep,
        timeout_secs: 30,
    };

    let result = run_social_pipeline(&cfg, config, http).await;
    spinner.finish_and_clear();

    match result {
        Ok(r) => {
            if matches!(format, Format::Json) {
                println!("{}", serde_json::to_string_pretty(&r)?);
            } else {
                let md = unified_format(&r);
                println!("{md}");
                if let Some(path) = &args.output {
                    std::fs::write(path, &md)?;
                    println!("{} {path}", "Saved report to".green());
                }
            }
        }
        Err(e) => eprintln!("{} {e}", "Unified research error:".red()),
    }
    Ok(())
}

// ─── Helpers ──────────────────────────────────────────────────────

fn build_platform_list(args: &SocialArgs) -> Vec<SocialPlatform> {
    let mut platforms = Vec::new();
    if args.twitter {
        platforms.push(SocialPlatform::Twitter);
    }
    if args.reddit {
        platforms.push(SocialPlatform::Reddit);
    }
    if args.tiktok {
        platforms.push(SocialPlatform::TikTok);
    }
    if args.hackernews {
        platforms.push(SocialPlatform::HackerNews);
    }
    if args.youtube {
        platforms.push(SocialPlatform::YouTube);
    }
    if args.facebook {
        platforms.push(SocialPlatform::Facebook);
    }

    if platforms.is_empty() {
        // Default: all platforms
        vec![
            SocialPlatform::Twitter,
            SocialPlatform::Reddit,
            SocialPlatform::TikTok,
            SocialPlatform::HackerNews,
            SocialPlatform::YouTube,
            SocialPlatform::Facebook,
        ]
    } else {
        platforms
    }
}
