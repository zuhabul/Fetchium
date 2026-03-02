//! YouTube Intelligence System CLI command handler.

use crate::cli::Format;
use anyhow::{Context, Result};
use colored::Colorize;
use fetchium_core::config::HsxConfig;
use fetchium_core::http::client::HttpClient;
use fetchium_core::summarize::{self, SummarizeConfig};
use fetchium_core::youtube::pipeline;
use fetchium_core::youtube::types::{
    EnhancedTranscript, KeyMoment, TranscriptEntry, YouTubePipelineConfig,
};
use serde::Serialize;
use std::time::Duration;

/// Run the YouTube subcommand.
pub async fn run(args: crate::cli::YouTubeArgs, config: &HsxConfig, format: Format) -> Result<()> {
    let http = HttpClient::new(config)?;

    match args.action {
        crate::cli::YouTubeAction::Search(search_args) => {
            run_search(&search_args, config, &http, format).await
        }
        crate::cli::YouTubeAction::Watch(watch_args) => {
            run_watch(&watch_args, config, &http, format).await
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
        crate::cli::YouTubeAction::PlaylistAnalyze(playlist_args) => {
            run_playlist_analyze(&playlist_args, config, &http, format).await
        }
        crate::cli::YouTubeAction::ChannelAudit(channel_args) => {
            run_channel_audit(&channel_args, config, &http, format).await
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
    } else if args.compact {
        print_compact_search_table(&result);
    } else {
        println!("{}", pipeline::format_result_markdown(&result));
    }

    if let Some(idx) = args.open {
        if let Some(url) = ranked_url(&result, idx) {
            open_in_browser(&url);
            eprintln!("Opened: {url}");
        } else {
            eprintln!("No ranked result at index {}", idx);
        }
    }
    if let Some(idx) = args.copy_url {
        if let Some(url) = ranked_url(&result, idx) {
            if copy_to_clipboard(&url).is_ok() {
                eprintln!("Copied URL: {url}");
            } else {
                eprintln!("Could not copy automatically. URL: {url}");
            }
        } else {
            eprintln!("No ranked result at index {}", idx);
        }
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

async fn run_watch(
    args: &crate::cli::YtWatchArgs,
    config: &HsxConfig,
    http: &HttpClient,
    format: Format,
) -> Result<()> {
    let url = if args.input.starts_with("http://") || args.input.starts_with("https://") {
        args.input.clone()
    } else {
        format!("https://www.youtube.com/watch?v={}", args.input)
    };
    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_message("Building unified watch report...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let result =
        pipeline::analyze_single_video(&url, config, http, args.comments, args.transcript, false)
            .await?;
    let summary = summarize::summarize(&url, &SummarizeConfig::default(), config)
        .await
        .ok();
    spinner.finish_and_clear();

    if format == Format::Json {
        let mut out = serde_json::to_value(&result)?;
        if let Some(s) = summary {
            out["summary"] = serde_json::Value::String(s.summary);
        }
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    let video = match result.videos.first() {
        Some(v) => v,
        None => {
            println!("No video analysis available.");
            return Ok(());
        }
    };
    println!("{}", "YouTube Watch".bold());
    println!("{}", "═".repeat(72));
    println!("Title: {}", video.metadata.title);
    println!(
        "URL: https://www.youtube.com/watch?v={}",
        video.metadata.video_id
    );
    println!("Channel: {}", video.metadata.channel.name);
    println!(
        "Published: {}{}",
        video.metadata.published,
        format_published_absolute(&video.metadata.published)
    );
    println!(
        "Views: {} | Likes: {} | Duration: {}",
        format_number(video.metadata.view_count),
        format_number(video.metadata.like_count),
        format_duration_hms(video.metadata.duration_secs)
    );
    println!(
        "Confidence: views={} likes={} publish={}",
        confidence_badge(metadata_confidence_views(video)),
        confidence_badge(metadata_confidence_likes(video)),
        confidence_badge(metadata_confidence_published(video))
    );
    if !video.metadata.chapters.is_empty() {
        println!("\n{}", "Chapters".bold());
        for ch in &video.metadata.chapters {
            println!(
                "  {} {}",
                format_duration_hms(ch.start_secs).cyan(),
                ch.title
            );
        }
    }
    if let Some(s) = summary {
        println!("\n{}", "Summary".bold());
        println!("{}", s.summary);
    }
    if let Some(t) = &video.transcript {
        println!("\n{}", "Transcript Preview".bold());
        for entry in t.entries.iter().take(5) {
            println!(
                "  {} {}",
                format_jump_timestamp(entry.start_ms).dimmed(),
                truncate_str(&entry.text, 140)
            );
        }
        println!("\n{}", "Top Highlights".bold());
        for m in select_highlights(t, args.highlights) {
            println!(
                "  {} {}",
                format_jump_timestamp(m.timestamp_ms).cyan(),
                m.text
            );
        }
    }
    if let Some(c) = &video.comments {
        println!("\n{}", "Top Comment Signals".bold());
        println!(
            "Comments analyzed: {} | Positive sentiment: {:.0}%",
            c.analyzed_comments,
            c.overall_sentiment.positive * 100.0
        );
        if !c.informative_comments.is_empty() {
            println!("{}", "Top Comments".bold());
            for comment in c.informative_comments.iter().take(5) {
                println!(
                    "  [{}] {}",
                    format!("{:.2}", comment.score).yellow(),
                    truncate_str(&comment.text, 180)
                );
            }
        }
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

async fn run_playlist_analyze(
    args: &crate::cli::YtPlaylistAnalyzeArgs,
    config: &HsxConfig,
    http: &HttpClient,
    format: Format,
) -> Result<()> {
    let url = normalize_playlist_input_to_url(&args.input);
    let ids = fetch_flat_video_ids(&url, args.max_results).await?;
    if ids.is_empty() {
        anyhow::bail!("No videos found in playlist");
    }
    let mut videos = Vec::new();
    for vid in ids.iter().take(args.max_results) {
        let url = format!("https://www.youtube.com/watch?v={vid}");
        if let Ok(one) = pipeline::analyze_single_video(&url, config, http, true, true, false).await
        {
            if let Some(v) = one.videos.into_iter().next() {
                videos.push(v);
            }
        }
    }
    let query = format!(
        "playlist:{}",
        extract_playlist_id(&args.input).unwrap_or_default()
    );
    let rankings = fetchium_core::youtube::ranking::rank_videos(&videos, &query);
    let learning_path = if args.learning_path {
        Some(fetchium_core::youtube::intelligence::generate_learning_path(&videos))
    } else {
        None
    };
    let result = fetchium_core::youtube::types::YouTubePipelineResult {
        query,
        videos,
        rankings,
        fact_checks: None,
        timeline: None,
        learning_path,
        teaching: None,
        duration_ms: 0,
    };
    if format == Format::Json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("{}", "Playlist Analysis".bold());
        println!("{}", pipeline::format_result_markdown(&result));
    }
    Ok(())
}

async fn run_channel_audit(
    args: &crate::cli::YtChannelAuditArgs,
    config: &HsxConfig,
    http: &HttpClient,
    format: Format,
) -> Result<()> {
    let url = normalize_channel_input_to_url(&args.input);
    let videos = fetch_flat_video_ids(&format!("{url}/videos"), args.max_results).await?;
    if videos.is_empty() {
        anyhow::bail!("No videos found for channel audit");
    }
    let mut analyses = Vec::new();
    for vid in videos.iter().take(args.max_results) {
        let video_url = format!("https://www.youtube.com/watch?v={vid}");
        if let Ok(one) =
            pipeline::analyze_single_video(&video_url, config, http, false, true, false).await
        {
            if let Some(v) = one.videos.into_iter().next() {
                analyses.push(v);
            }
        }
    }
    let avg_views = if analyses.is_empty() {
        0.0
    } else {
        analyses
            .iter()
            .map(|v| v.metadata.view_count as f64)
            .sum::<f64>()
            / analyses.len() as f64
    };
    let avg_duration = if analyses.is_empty() {
        0.0
    } else {
        analyses
            .iter()
            .map(|v| v.metadata.duration_secs as f64)
            .sum::<f64>()
            / analyses.len() as f64
    };
    let edu_ratio = if analyses.is_empty() {
        0.0
    } else {
        let ranks = fetchium_core::youtube::ranking::rank_videos(&analyses, "channel audit");
        ranks.iter().map(|r| r.educational_score).sum::<f64>() / ranks.len() as f64
    };

    if format == Format::Json {
        let out = serde_json::json!({
            "channel_input": args.input,
            "audited_videos": analyses.len(),
            "avg_views": avg_views,
            "avg_duration_secs": avg_duration,
            "avg_educational_score": edu_ratio,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("{}", "Channel Audit".bold());
        println!("Input: {}", args.input);
        println!("Audited videos: {}", analyses.len());
        println!("Average views: {}", format_number(avg_views as u64));
        println!(
            "Average duration: {}",
            format_duration_hms(avg_duration as u64)
        );
        println!("Average educational score: {:.2}", edu_ratio);
        println!("Strategy: prioritize high educational-score topics with above-average views.");
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
    let mut transcript = fetch_universal_transcript(&args.url, http, config).await?;
    let mut translated_entries = true;
    if let Some(lang) = args.translate.as_deref() {
        // Best-effort translation. Translating each entry can be very slow on long videos,
        // so we translate full text first and only translate entries for shorter transcripts.
        transcript.full_text = maybe_translate_text(&transcript.full_text, lang, http).await;
        if transcript.entries.len() <= 80 {
            for entry in &mut transcript.entries {
                entry.text = maybe_translate_text(&entry.text, lang, http).await;
            }
        } else {
            translated_entries = false;
        }
        transcript.language = lang.to_string();
    }
    spinner.finish_and_clear();

    if format == Format::Json {
        if args.chunks {
            let chunks = if args.semantic {
                chunk_transcript_semantic(&transcript.entries, &transcript.key_moments)
            } else {
                chunk_transcript(&transcript.entries, args.chunk_size)
            };
            let out = serde_json::json!({
                "language": transcript.language,
                "source": transcript.source,
                "quality_score": transcript.quality_score,
                "word_count": transcript.word_count,
                "text": transcript.full_text,
                "translated_entries": translated_entries,
                "chunks": chunks,
                "highlights": select_highlights(&transcript, args.highlights),
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
            let chunks = if args.semantic {
                chunk_transcript_semantic(&transcript.entries, &transcript.key_moments)
            } else {
                chunk_transcript(&transcript.entries, args.chunk_size)
            };
            for c in chunks {
                println!(
                    "\n[{} - {}]\n{}",
                    format_jump_timestamp(c.start_ms),
                    format_jump_timestamp(c.end_ms),
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
        if args.translate.is_some() && !translated_entries {
            println!(
                "{}",
                "Note: translated full transcript text; per-entry translation skipped for speed."
                    .dimmed()
            );
        }

        let show_chapters =
            args.chapters || fetchium_core::multimodal::video::extract_video_id(&args.url).is_ok();
        if show_chapters {
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
            for moment in select_highlights(&transcript, args.highlights) {
                let ts = format_jump_timestamp(moment.timestamp_ms);
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

fn format_jump_timestamp(ms: u32) -> String {
    let secs = ms / 1000;
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
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

fn chunk_transcript_semantic(
    entries: &[TranscriptEntry],
    moments: &[KeyMoment],
) -> Vec<TranscriptChunk> {
    if entries.is_empty() {
        return Vec::new();
    }
    if moments.is_empty() {
        return chunk_transcript(entries, 600);
    }
    let mut boundaries: Vec<u32> = moments.iter().map(|m| m.timestamp_ms).collect();
    boundaries.sort_unstable();
    boundaries.dedup();

    let mut out = Vec::new();
    let mut current: Vec<&TranscriptEntry> = Vec::new();
    let mut next_boundary_idx = 0usize;
    let mut boundary = boundaries
        .get(next_boundary_idx)
        .copied()
        .unwrap_or(u32::MAX);

    for e in entries {
        while e.start_ms >= boundary && !current.is_empty() {
            let start_ms = current.first().map(|x| x.start_ms).unwrap_or(e.start_ms);
            let end_ms = current
                .last()
                .map(|x| x.start_ms + x.duration_ms)
                .unwrap_or(e.start_ms);
            let text = current
                .iter()
                .map(|x| x.text.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            out.push(TranscriptChunk {
                start_ms,
                end_ms,
                text,
            });
            current.clear();
            next_boundary_idx += 1;
            boundary = boundaries
                .get(next_boundary_idx)
                .copied()
                .unwrap_or(u32::MAX);
        }
        current.push(e);
    }
    if !current.is_empty() {
        let start_ms = current.first().map(|x| x.start_ms).unwrap_or(0);
        let end_ms = current
            .last()
            .map(|x| x.start_ms + x.duration_ms)
            .unwrap_or(start_ms);
        let text = current
            .iter()
            .map(|x| x.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        out.push(TranscriptChunk {
            start_ms,
            end_ms,
            text,
        });
    }
    out
}

fn select_highlights(transcript: &EnhancedTranscript, n: usize) -> Vec<KeyMoment> {
    let mut moments = transcript.key_moments.clone();
    moments.sort_by(|a, b| {
        b.importance
            .partial_cmp(&a.importance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    moments.into_iter().take(n.max(1)).collect()
}

fn ranked_url(
    result: &fetchium_core::youtube::types::YouTubePipelineResult,
    idx: usize,
) -> Option<String> {
    if idx == 0 {
        return None;
    }
    let rank = result.rankings.get(idx - 1)?;
    Some(format!("https://www.youtube.com/watch?v={}", rank.video_id))
}

fn open_in_browser(url: &str) {
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
}

fn copy_to_clipboard(text: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        use std::io::Write;
        let mut child = std::process::Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(text.as_bytes())?;
        }
        let _ = child.wait();
        return Ok(());
    }
    #[cfg(target_os = "linux")]
    {
        use std::io::Write;
        for bin in ["wl-copy", "xclip", "xsel"] {
            let mut cmd = match bin {
                "wl-copy" => {
                    let mut c = std::process::Command::new(bin);
                    c.stdin(std::process::Stdio::piped());
                    c
                }
                "xclip" => {
                    let mut c = std::process::Command::new(bin);
                    c.args(["-selection", "clipboard"])
                        .stdin(std::process::Stdio::piped());
                    c
                }
                _ => {
                    let mut c = std::process::Command::new(bin);
                    c.args(["--clipboard", "--input"])
                        .stdin(std::process::Stdio::piped());
                    c
                }
            };
            if let Ok(mut child) = cmd.spawn() {
                if let Some(stdin) = child.stdin.as_mut() {
                    let _ = stdin.write_all(text.as_bytes());
                }
                let _ = child.wait();
                return Ok(());
            }
        }
        anyhow::bail!("No clipboard utility found");
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    anyhow::bail!("Clipboard not supported on this platform")
}

fn print_compact_search_table(result: &fetchium_core::youtube::types::YouTubePipelineResult) {
    println!("{}", format!("YouTube Search: {}", result.query).bold());
    println!(
        "{:<3} {:<42} {:<10} {:<10} {:<12} {:<8} URL",
        "#", "Title", "Duration", "Views", "Published", "Score"
    );
    println!("{}", "-".repeat(110));
    for (i, rank) in result.rankings.iter().enumerate() {
        if let Some(v) = result
            .videos
            .iter()
            .find(|x| x.metadata.video_id == rank.video_id)
        {
            println!(
                "{:<3} {:<42} {:<10} {:<10} {:<12} {:<8.2} {}",
                i + 1,
                truncate_str(&v.metadata.title, 42),
                format_duration_hms(v.metadata.duration_secs),
                format_number(v.metadata.view_count),
                truncate_str(&v.metadata.published, 12),
                rank.final_score,
                format_args!("https://youtu.be/{}", v.metadata.video_id)
            );
        }
    }
}

fn format_duration_hms(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

fn format_published_absolute(published: &str) -> String {
    if published.is_empty() {
        return String::new();
    }
    if published.len() >= 10 && published.chars().nth(4) == Some('-') {
        return format!(" ({published})");
    }
    let d = if let Some(days) = parse_relative_days(published) {
        let dt = chrono::Utc::now() - chrono::Duration::days(days as i64);
        dt.format("%Y-%m-%d").to_string()
    } else {
        String::new()
    };
    if d.is_empty() {
        String::new()
    } else {
        format!(" ({d})")
    }
}

fn parse_relative_days(s: &str) -> Option<u64> {
    let lower = s.to_lowercase();
    let n: u64 = lower
        .split_whitespace()
        .find_map(|w| w.parse::<u64>().ok())?;
    if lower.contains("minute") || lower.contains("hour") {
        Some(0)
    } else if lower.contains("day") {
        Some(n)
    } else if lower.contains("week") {
        Some(n * 7)
    } else if lower.contains("month") {
        Some(n * 30)
    } else if lower.contains("year") {
        Some(n * 365)
    } else {
        None
    }
}

async fn maybe_translate_text(text: &str, target_lang: &str, http: &HttpClient) -> String {
    if text.trim().is_empty() || target_lang.trim().is_empty() {
        return text.to_string();
    }
    let input = truncate_str(text, 450);
    let encoded: String = url::form_urlencoded::byte_serialize(input.as_bytes()).collect();
    let url = format!(
        "https://api.mymemory.translated.net/get?q={encoded}&langpair=en|{}",
        target_lang
    );
    match http.fetch_text_once(&url).await {
        Ok(body) => {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) {
                if v["responseStatus"].as_i64().unwrap_or(200) != 200 {
                    return text.to_string();
                }
                if let Some(t) = v["responseData"]["translatedText"].as_str() {
                    let t = t.trim();
                    let upper = t.to_ascii_uppercase();
                    if !t.is_empty()
                        && !upper.contains("QUERY LENGTH LIMIT EXCEEDED")
                        && !upper.contains("MAX ALLOWED QUERY")
                        && !upper.contains("MYMEMORY WARNING")
                    {
                        return t.to_string();
                    }
                }
            }
            text.to_string()
        }
        Err(_) => text.to_string(),
    }
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn metadata_confidence_views(video: &fetchium_core::youtube::types::VideoAnalysis) -> &'static str {
    if video.metadata.view_count > 0 {
        "high"
    } else if video.metadata.duration_secs > 0 && !video.metadata.title.trim().is_empty() {
        "medium"
    } else {
        "low"
    }
}

fn metadata_confidence_likes(video: &fetchium_core::youtube::types::VideoAnalysis) -> &'static str {
    if video.metadata.like_count > 0 {
        "high"
    } else if video.metadata.view_count > 0 {
        "medium"
    } else {
        "low"
    }
}

fn metadata_confidence_published(
    video: &fetchium_core::youtube::types::VideoAnalysis,
) -> &'static str {
    let p = video.metadata.published.trim();
    if p.is_empty() {
        "low"
    } else if p.len() >= 10 && p.chars().nth(4) == Some('-') && p.chars().nth(7) == Some('-') {
        "high"
    } else {
        "medium"
    }
}

fn confidence_badge(level: &str) -> colored::ColoredString {
    match level {
        "high" => "[HIGH]".green().bold(),
        "medium" => "[MED]".yellow().bold(),
        _ => "[LOW]".red().bold(),
    }
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
