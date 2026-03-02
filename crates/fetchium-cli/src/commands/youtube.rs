//! YouTube Intelligence System CLI command handler.

use crate::cli::Format;
use anyhow::{Context, Result};
use colored::Colorize;
use fetchium_core::config::HsxConfig;
use fetchium_core::http::client::HttpClient;
use fetchium_core::youtube::pipeline;
use fetchium_core::youtube::types::{TranscriptEntry, YouTubePipelineConfig};
use serde::Serialize;
use std::time::Duration;

/// Run the YouTube subcommand.
pub async fn run(args: crate::cli::YouTubeArgs, config: &HsxConfig, format: Format) -> Result<()> {
    let http = HttpClient::new(config)?;

    match args.action {
        crate::cli::YouTubeAction::Search(search_args) => {
            run_search(&search_args, config, &http, format).await
        }
        crate::cli::YouTubeAction::Video(video_args) => {
            run_video(&video_args, config, &http, format).await
        }
        crate::cli::YouTubeAction::Channel(channel_args) => {
            run_channel(&channel_args, format).await
        }
        crate::cli::YouTubeAction::Playlist(playlist_args) => {
            run_playlist(&playlist_args, format).await
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

async fn run_video(
    args: &crate::cli::YtVideoArgs,
    config: &HsxConfig,
    http: &HttpClient,
    format: Format,
) -> Result<()> {
    let video_id = if args.input.starts_with("http://") || args.input.starts_with("https://") {
        fetchium_core::multimodal::video::extract_video_id(&args.input)
            .with_context(|| format!("Invalid YouTube URL: {}", args.input))?
    } else {
        args.input.clone()
    };
    let meta = fetchium_core::youtube::metadata::fetch_metadata(&video_id, http, config).await?;

    if format == Format::Json {
        println!("{}", serde_json::to_string_pretty(&meta)?);
    } else {
        println!("{}", "YouTube Video Metadata".bold());
        println!("ID: {}", meta.video_id);
        println!("Title: {}", meta.title);
        println!("Channel: {}", meta.channel.name);
        println!("Views: {}", meta.view_count);
        println!("Likes: {}", meta.like_count);
        println!("Published: {}", meta.published);
        println!("Duration: {}s", meta.duration_secs);
    }
    Ok(())
}

#[derive(Debug, Serialize)]
struct ChannelResult {
    id: Option<String>,
    name: String,
    handle: Option<String>,
    description: Option<String>,
    follower_count: Option<u64>,
    videos: Vec<String>,
}

async fn run_channel(args: &crate::cli::YtChannelArgs, format: Format) -> Result<()> {
    let url = normalize_channel_input_to_url(&args.input);
    let mut meta = fetch_channel_metadata_ytdlp(&url).await?;
    if args.videos {
        meta.videos = fetch_flat_video_ids(&format!("{url}/videos"), args.max_results).await?;
    }

    if format == Format::Json {
        println!("{}", serde_json::to_string_pretty(&meta)?);
    } else {
        println!("{}", "YouTube Channel".bold());
        println!("Name: {}", meta.name);
        if let Some(id) = &meta.id {
            println!("ID: {id}");
        }
        if let Some(handle) = &meta.handle {
            println!("Handle: {handle}");
        }
        if let Some(subs) = meta.follower_count {
            println!("Subscribers: {subs}");
        }
        if let Some(desc) = &meta.description {
            println!("Description: {}", truncate_str(desc, 240));
        }
        if args.videos {
            println!("\nVideo IDs ({}):", meta.videos.len());
            for vid in &meta.videos {
                println!("  - {vid}");
            }
        }
    }
    Ok(())
}

#[derive(Debug, Serialize)]
struct PlaylistResult {
    id: Option<String>,
    video_ids: Vec<String>,
}

async fn run_playlist(args: &crate::cli::YtPlaylistArgs, format: Format) -> Result<()> {
    let url = normalize_playlist_input_to_url(&args.input);
    let out = PlaylistResult {
        id: extract_playlist_id(&args.input),
        video_ids: fetch_flat_video_ids(&url, args.max_results).await?,
    };
    if format == Format::Json {
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("{}", "YouTube Playlist".bold());
        if let Some(id) = &out.id {
            println!("ID: {id}");
        }
        println!("Video IDs ({}):", out.video_ids.len());
        for vid in &out.video_ids {
            println!("  - {vid}");
        }
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
    use fetchium_core::youtube::universal::{detect_platform, fetch_universal_transcript};

    let platform = detect_platform(&args.url);
    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_message(format!("Extracting transcript from {platform}..."));
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    // Universal transcript extraction: YouTube fast-path or yt-dlp for any other platform
    let transcript = fetch_universal_transcript(&args.url, http, config).await?;
    spinner.finish_and_clear();

    if format == Format::Json {
        if args.chunks {
            let chunks = chunk_transcript(&transcript.entries, args.chunk_size);
            let out = serde_json::json!({
                "language": transcript.language,
                "source": transcript.source,
                "quality_score": transcript.quality_score,
                "word_count": transcript.word_count,
                "text": transcript.full_text,
                "chunks": chunks,
            });
            println!("{}", serde_json::to_string_pretty(&out)?);
        } else if args.text {
            println!("{}", transcript.full_text);
        } else {
            println!("{}", serde_json::to_string_pretty(&transcript)?);
        }
    } else {
        if args.text {
            println!("{}", transcript.full_text);
            return Ok(());
        }
        if args.chunks {
            println!("{}", format!("{platform} Transcript Chunks").bold());
            for c in chunk_transcript(&transcript.entries, args.chunk_size) {
                println!(
                    "\n[{} - {}]\n{}",
                    format_timestamp(c.start_ms),
                    format_timestamp(c.end_ms),
                    c.text
                );
            }
            return Ok(());
        }

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
            let maybe_video_id = fetchium_core::multimodal::video::extract_video_id(&args.url).ok();
            let chapters_shown = if let Some(ref vid) = maybe_video_id {
                if let Ok(meta) =
                    fetchium_core::youtube::metadata::fetch_metadata(vid, http, config).await
                {
                    if !meta.chapters.is_empty() {
                        println!("{}", "Chapters:".bold());
                        let aligned = fetchium_core::youtube::transcript::align_to_chapters(
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

#[derive(Debug, Clone, Serialize)]
struct TranscriptChunk {
    start_ms: u32,
    end_ms: u32,
    text: String,
}

fn chunk_transcript(entries: &[TranscriptEntry], chunk_size: usize) -> Vec<TranscriptChunk> {
    if entries.is_empty() {
        return Vec::new();
    }
    let target = chunk_size.max(120);
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut start_ms = entries[0].start_ms;
    let mut end_ms = entries[0].start_ms + entries[0].duration_ms;

    for e in entries {
        let next = if buf.is_empty() {
            e.text.clone()
        } else {
            format!("{buf} {}", e.text)
        };
        let seg_end = e.start_ms + e.duration_ms;
        if !buf.is_empty() && next.chars().count() > target {
            out.push(TranscriptChunk {
                start_ms,
                end_ms,
                text: buf.trim().to_string(),
            });
            buf = e.text.clone();
            start_ms = e.start_ms;
        } else {
            buf = next;
        }
        end_ms = seg_end;
    }
    if !buf.trim().is_empty() {
        out.push(TranscriptChunk {
            start_ms,
            end_ms,
            text: buf.trim().to_string(),
        });
    }
    out
}

fn normalize_channel_input_to_url(input: &str) -> String {
    if input.starts_with("http://") || input.starts_with("https://") {
        return input.to_string();
    }
    if input.starts_with('@') {
        return format!("https://www.youtube.com/{input}");
    }
    if input.starts_with("UC") {
        return format!("https://www.youtube.com/channel/{input}");
    }
    format!("https://www.youtube.com/@{input}")
}

fn normalize_playlist_input_to_url(input: &str) -> String {
    if input.starts_with("http://") || input.starts_with("https://") {
        return input.to_string();
    }
    format!("https://www.youtube.com/playlist?list={input}")
}

fn extract_playlist_id(input: &str) -> Option<String> {
    if input.starts_with("http://") || input.starts_with("https://") {
        if let Ok(u) = url::Url::parse(input) {
            return u
                .query_pairs()
                .find_map(|(k, v)| (k == "list").then(|| v.to_string()));
        }
        None
    } else {
        Some(input.to_string())
    }
}

async fn fetch_channel_metadata_ytdlp(url: &str) -> Result<ChannelResult> {
    let output = tokio::time::timeout(
        Duration::from_secs(20),
        tokio::process::Command::new("yt-dlp")
            .args([
                "--dump-single-json",
                "--flat-playlist",
                "--no-warnings",
                "--quiet",
                url,
            ])
            .output(),
    )
    .await
    .context("yt-dlp channel metadata timed out")?
    .map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            anyhow::anyhow!("yt-dlp not found. Install with: pip install yt-dlp")
        } else {
            anyhow::anyhow!("failed to execute yt-dlp for channel metadata: {e}")
        }
    })?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("yt-dlp channel metadata failed: {err}");
    }
    let v: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    Ok(ChannelResult {
        id: v["channel_id"].as_str().map(|s| s.to_string()),
        name: v["channel"]
            .as_str()
            .or_else(|| v["uploader"].as_str())
            .unwrap_or("")
            .to_string(),
        handle: v["uploader_id"].as_str().map(|s| s.to_string()),
        description: v["description"].as_str().map(|s| s.to_string()),
        follower_count: v["channel_follower_count"].as_u64(),
        videos: Vec::new(),
    })
}

async fn fetch_flat_video_ids(url: &str, max_results: usize) -> Result<Vec<String>> {
    let output = tokio::time::timeout(
        Duration::from_secs(25),
        tokio::process::Command::new("yt-dlp")
            .args([
                "--flat-playlist",
                "--dump-single-json",
                "--playlist-end",
                &max_results.to_string(),
                "--no-warnings",
                "--quiet",
                url,
            ])
            .output(),
    )
    .await
    .context("yt-dlp flat playlist timed out")?
    .map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            anyhow::anyhow!("yt-dlp not found. Install with: pip install yt-dlp")
        } else {
            anyhow::anyhow!("failed to execute yt-dlp for playlist/channel videos: {e}")
        }
    })?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("yt-dlp flat playlist failed: {err}");
    }

    let v: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let mut ids = Vec::new();
    if let Some(entries) = v["entries"].as_array() {
        for e in entries.iter().take(max_results) {
            if let Some(id) = e["id"].as_str() {
                ids.push(id.to_string());
            }
        }
    }
    Ok(ids)
}
