//! YouTube Intelligence System CLI command handler.

use crate::cli::Format;
use anyhow::Result;
use colored::Colorize;
use hsx_core::config::HsxConfig;
use hsx_core::http::client::HttpClient;
use hsx_core::youtube::pipeline;
use hsx_core::youtube::types::YouTubePipelineConfig;

/// Run the YouTube subcommand.
pub async fn run(args: crate::cli::YouTubeArgs, config: &HsxConfig, format: Format) -> Result<()> {
    let http = HttpClient::new(config)?;

    match args.action {
        crate::cli::YouTubeAction::Search(search_args) => {
            run_search(&search_args, config, &http, format).await
        }
        crate::cli::YouTubeAction::Analyze(analyze_args) => {
            run_analyze(&analyze_args, config, &http, format).await
        }
        crate::cli::YouTubeAction::Transcript(transcript_args) => {
            run_transcript(&transcript_args, config, &http, format).await
        }
        crate::cli::YouTubeAction::Research(research_args) => {
            run_research(&research_args, config, &http, format).await
        }
        crate::cli::YouTubeAction::Compare(compare_args) => {
            run_compare(&compare_args, config, &http, format).await
        }
    }
}

async fn run_search(
    args: &crate::cli::YtSearchArgs,
    config: &HsxConfig,
    http: &HttpClient,
    format: Format,
) -> Result<()> {
    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_message(format!("Searching YouTube for '{}'...", args.query));
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let pipeline_config = YouTubePipelineConfig {
        query: args.query.clone(),
        max_videos: args.max_results,
        fetch_transcript: false,
        fetch_comments: false,
        fact_check: false,
        build_timeline: false,
        build_learning_path: false,
        generate_teaching: false,
        token_budget: 4000,
    };

    let result = pipeline::run_youtube_pipeline(&pipeline_config, config, http).await?;
    spinner.finish_and_clear();

    if format == Format::Json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("{}", pipeline::format_result_markdown(&result));
    }

    Ok(())
}

async fn run_analyze(
    args: &crate::cli::YtAnalyzeArgs,
    config: &HsxConfig,
    http: &HttpClient,
    format: Format,
) -> Result<()> {
    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_message(format!("Analyzing video: {}...", args.url));
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let result = pipeline::analyze_single_video(
        &args.url,
        config,
        http,
        args.comments,
        args.transcript,
        args.teaching,
    )
    .await?;
    spinner.finish_and_clear();

    if format == Format::Json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("{}", pipeline::format_result_markdown(&result));
    }

    Ok(())
}

async fn run_transcript(
    args: &crate::cli::YtTranscriptArgs,
    config: &HsxConfig,
    http: &HttpClient,
    format: Format,
) -> Result<()> {
    use hsx_core::youtube::universal::{detect_platform, fetch_universal_transcript};

    let platform = detect_platform(&args.url);
    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_message(format!("Extracting transcript from {platform}..."));
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    // Universal transcript extraction: YouTube fast-path or yt-dlp for any other platform
    let transcript = fetch_universal_transcript(&args.url, http, config).await?;
    spinner.finish_and_clear();

    if format == Format::Json {
        println!("{}", serde_json::to_string_pretty(&transcript)?);
    } else {
        // Quality indicator: ★★★ = excellent, ★★☆ = fair, ★☆☆ = poor
        let quality_stars = match (transcript.quality_score * 10.0) as u32 {
            8..=10 => "★★★ Excellent",
            5..=7 => "★★☆ Fair",
            _ => "★☆☆ Poor (garbled ASR or non-speech video)",
        };
        println!("{}", format!("{platform} Transcript").bold());
        println!("URL: {}", args.url.dimmed());
        println!(
            "Language: {} | Source: {:?}",
            transcript.language, transcript.source
        );
        println!(
            "Quality: {} ({:.0}%)",
            quality_stars,
            transcript.quality_score * 100.0
        );
        println!(
            "Words: {} | Speakers: {} | Key moments: {}\n",
            transcript.word_count,
            transcript.speakers.len(),
            transcript.key_moments.len()
        );

        if args.chapters {
            // Try to get chapters from YouTube metadata (only works for YouTube)
            let maybe_video_id = hsx_core::multimodal::video::extract_video_id(&args.url).ok();
            let chapters_shown = if let Some(ref vid) = maybe_video_id {
                if let Ok(meta) =
                    hsx_core::youtube::metadata::fetch_metadata(vid, http, config).await
                {
                    if !meta.chapters.is_empty() {
                        println!("{}", "Chapters:".bold());
                        let aligned = hsx_core::youtube::transcript::align_to_chapters(
                            &transcript.entries,
                            &meta.chapters,
                        );
                        for (title, entries) in &aligned {
                            println!("\n{}", format!("## {title}").cyan());
                            let text: String = entries
                                .iter()
                                .map(|e| e.text.as_str())
                                .collect::<Vec<_>>()
                                .join(" ");
                            println!("{text}");
                        }
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            };

            if !chapters_shown {
                println!("{}", transcript.full_text);
            }
        } else {
            println!("{}", transcript.full_text);
        }

        // Print key moments
        if !transcript.key_moments.is_empty() {
            println!("\n{}", "Key Moments:".bold());
            for moment in &transcript.key_moments {
                let ts = format_timestamp(moment.timestamp_ms);
                println!(
                    "  {} [{:?}] {}",
                    ts.yellow(),
                    moment.moment_type,
                    moment.text
                );
            }
        }
    }

    Ok(())
}

async fn run_research(
    args: &crate::cli::YtResearchArgs,
    config: &HsxConfig,
    http: &HttpClient,
    format: Format,
) -> Result<()> {
    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_message(format!("Researching '{}' on YouTube...", args.query));
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let pipeline_config = YouTubePipelineConfig {
        query: args.query.clone(),
        max_videos: args.max_videos,
        fetch_transcript: true,
        fetch_comments: true,
        fact_check: args.fact_check,
        build_timeline: args.timeline,
        build_learning_path: args.learning_path,
        generate_teaching: false,
        token_budget: 8000,
    };

    let result = pipeline::run_youtube_pipeline(&pipeline_config, config, http).await?;
    spinner.finish_and_clear();

    if format == Format::Json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        let md = pipeline::format_result_markdown(&result);
        println!("{md}");

        // Write to file if specified
        if let Some(ref path) = args.output {
            std::fs::write(path, &md)?;
            println!("\n{}", format!("Report saved to {path}").green());
        }
    }

    Ok(())
}

async fn run_compare(
    args: &crate::cli::YtCompareArgs,
    config: &HsxConfig,
    http: &HttpClient,
    format: Format,
) -> Result<()> {
    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_message("Comparing videos...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    // Analyze both videos in parallel
    let http1 = http.clone();
    let http2 = http.clone();
    let config1 = config.clone();
    let config2 = config.clone();
    let url1 = args.urls[0].clone();
    let url2 = args.urls[1].clone();

    let (result1, result2) = tokio::join!(
        pipeline::analyze_single_video(&url1, &config1, &http1, true, true, false),
        pipeline::analyze_single_video(&url2, &config2, &http2, true, true, false),
    );

    spinner.finish_and_clear();

    let r1 = result1?;
    let r2 = result2?;

    if format == Format::Json {
        let combined = serde_json::json!({
            "video_a": r1,
            "video_b": r2,
        });
        println!("{}", serde_json::to_string_pretty(&combined)?);
    } else {
        println!("{}", "YouTube Video Comparison".bold());
        println!("{}", "═".repeat(60));

        // Side-by-side comparison
        if let (Some(v1), Some(v2)) = (r1.videos.first(), r2.videos.first()) {
            println!("\n{:<30} | {:<30}", "Video A".bold(), "Video B".bold());
            println!("{}", "─".repeat(63));
            println!(
                "{:<30} | {:<30}",
                truncate_str(&v1.metadata.title, 30),
                truncate_str(&v2.metadata.title, 30)
            );
            println!(
                "{:<30} | {:<30}",
                v1.metadata.channel.name, v2.metadata.channel.name
            );
            println!(
                "{:<30} | {:<30}",
                format!("{} views", v1.metadata.view_count),
                format!("{} views", v2.metadata.view_count)
            );
            println!(
                "{:<30} | {:<30}",
                format!("{} likes", v1.metadata.like_count),
                format!("{} likes", v2.metadata.like_count)
            );
            println!(
                "{:<30} | {:<30}",
                format!("Credibility: {:.0}%", v1.credibility.score * 100.0),
                format!("Credibility: {:.0}%", v2.credibility.score * 100.0)
            );

            if let (Some(ref t1), Some(ref t2)) = (&v1.transcript, &v2.transcript) {
                println!(
                    "{:<30} | {:<30}",
                    format!("{} words", t1.word_count),
                    format!("{} words", t2.word_count)
                );
            }

            if let (Some(ref c1), Some(ref c2)) = (&v1.comments, &v2.comments) {
                println!(
                    "{:<30} | {:<30}",
                    format!("Sentiment: {:.0}%+", c1.overall_sentiment.positive * 100.0),
                    format!("Sentiment: {:.0}%+", c2.overall_sentiment.positive * 100.0),
                );
            }
        }

        // Rankings comparison
        if let (Some(rank1), Some(rank2)) = (r1.rankings.first(), r2.rankings.first()) {
            println!("\n{}", "VideoFusion Scores:".bold());
            println!(
                "  Video A: {:.3} | Video B: {:.3}",
                rank1.final_score, rank2.final_score
            );
            println!(
                "  Educational: {:.2} vs {:.2}",
                rank1.educational_score, rank2.educational_score
            );
            println!(
                "  Clickbait: {:.2} vs {:.2}",
                rank1.clickbait_score, rank2.clickbait_score
            );
        }
    }

    Ok(())
}

fn format_timestamp(ms: u32) -> String {
    let secs = ms / 1000;
    let m = secs / 60;
    let s = secs % 60;
    format!("{m}:{s:02}")
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}
