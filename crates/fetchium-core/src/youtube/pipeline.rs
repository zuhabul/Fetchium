//! YouTube Intelligence pipeline — full 11-step orchestration.

use crate::config::FetchiumConfig;
use crate::error::{FetchiumError, FetchiumResult};
use crate::http::client::HttpClient;
use crate::youtube::types::*;
use crate::youtube::{comments, intelligence, metadata, ranking, search, transcript};
use std::time::{Duration, Instant};

/// Hard ceiling on the entire pipeline in seconds.
///
/// Computed as: search cap (12s) + per-handle cap (8s) + 2s buffer = 22s.
/// This ensures the pipeline always terminates even if tokio task orphans or
/// network stalls accumulate beyond what individual timeouts account for.
const PIPELINE_TIMEOUT_SECS: u64 = 22;

/// Execute the full YouTube Intelligence pipeline.
///
/// Steps:
/// 1. Search YouTube for videos matching query
/// 2. Fetch metadata for each video (parallel)
/// 3. Fetch transcripts (parallel)
/// 4. Fetch comments (parallel)
/// 5. Score channel credibility
/// 6. Build VideoAnalysis for each video
/// 7. Rank with VideoFusion
/// 8. Cross-video fact checking (optional)
/// 9. Build topic timeline (optional)
/// 10. Generate learning path (optional)
/// 11. Generate teaching content (optional)
pub async fn run_youtube_pipeline(
    pipeline_config: &YouTubePipelineConfig,
    config: &FetchiumConfig,
    http: &HttpClient,
) -> FetchiumResult<YouTubePipelineResult> {
    tokio::time::timeout(
        Duration::from_secs(PIPELINE_TIMEOUT_SECS),
        run_youtube_pipeline_inner(pipeline_config, config, http),
    )
    .await
    .map_err(|_| {
        FetchiumError::YouTube(format!(
            "YouTube pipeline timed out after {PIPELINE_TIMEOUT_SECS}s for '{}'",
            pipeline_config.query
        ))
    })?
}

/// Inner implementation — called by `run_youtube_pipeline` under a hard timeout.
async fn run_youtube_pipeline_inner(
    pipeline_config: &YouTubePipelineConfig,
    config: &FetchiumConfig,
    http: &HttpClient,
) -> FetchiumResult<YouTubePipelineResult> {
    let start = Instant::now();

    // Step 1: Search
    let (search_results, _source) = search::search_youtube(
        &pipeline_config.query,
        pipeline_config.max_videos,
        http,
        config,
    )
    .await?;

    if search_results.is_empty() {
        return Err(FetchiumError::YouTube(format!(
            "No YouTube videos found for '{}'",
            pipeline_config.query
        )));
    }

    // Steps 2-5: Parallel fetch metadata + transcript + comments per video
    let analyses = fetch_all_video_data(&search_results, pipeline_config, config, http).await;

    if analyses.is_empty() {
        return Err(FetchiumError::YouTube(
            "Failed to analyze any videos".into(),
        ));
    }

    // Step 7: Rank with VideoFusion
    let rankings = ranking::rank_videos(&analyses, &pipeline_config.query);

    // Steps 8-11: Optional intelligence features (parallel where possible)
    let (fact_checks, timeline, learning_path, teaching) =
        run_intelligence(&analyses, pipeline_config);

    Ok(YouTubePipelineResult {
        query: pipeline_config.query.clone(),
        videos: analyses,
        rankings,
        fact_checks,
        timeline,
        learning_path,
        teaching,
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

/// Analyze a single video by URL.
pub async fn analyze_single_video(
    url: &str,
    config: &FetchiumConfig,
    http: &HttpClient,
    fetch_comments_flag: bool,
    fetch_transcript_flag: bool,
    generate_teaching_flag: bool,
) -> FetchiumResult<YouTubePipelineResult> {
    let start = Instant::now();
    let video_id = crate::multimodal::video::extract_video_id(url)?;

    // Parallel: metadata + transcript + comments
    let (meta_result, transcript_result, comments_result) = tokio::join!(
        metadata::fetch_metadata(&video_id, http, config),
        async {
            if fetch_transcript_flag {
                transcript::fetch_transcript(&video_id, http, config)
                    .await
                    .ok()
            } else {
                None
            }
        },
        async {
            if fetch_comments_flag {
                comments::fetch_comments(&video_id, http, config, 100)
                    .await
                    .ok()
            } else {
                None
            }
        }
    );

    let meta = meta_result?;
    let credibility = metadata::score_channel_credibility(&meta.channel);
    let comment_analysis = comments_result.map(|c| comments::analyze_comments(&c));

    let analysis = VideoAnalysis {
        metadata: meta,
        transcript: transcript_result,
        comments: comment_analysis,
        credibility,
    };

    let rankings = ranking::rank_videos(std::slice::from_ref(&analysis), &analysis.metadata.title);
    let teaching = if generate_teaching_flag {
        Some(intelligence::generate_teaching(std::slice::from_ref(
            &analysis,
        )))
    } else {
        None
    };

    Ok(YouTubePipelineResult {
        query: analysis.metadata.title.clone(),
        videos: vec![analysis],
        rankings,
        fact_checks: None,
        timeline: None,
        learning_path: None,
        teaching,
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

/// Maximum concurrent video metadata fetches — prevents overwhelming HTTP pool.
///
/// Each fetch races oEmbed (~150ms) against Invidious/Piped. With 10 concurrent
/// and oEmbed winning, a batch of 10 completes in ~200ms.
const MAX_CONCURRENT_FETCHES: usize = 10;

/// Timeout cap for the full parallel fetch phase.
const FETCH_PHASE_TIMEOUT_SECS: u64 = 10;

/// Fetch data for all search results with bounded concurrency.
///
/// Uses a semaphore to ensure at most `MAX_CONCURRENT_FETCHES` requests
/// run simultaneously. This is resource-aware: a machine with fewer cores
/// gets the same throughput since oEmbed latency is the bottleneck, not CPU.
async fn fetch_all_video_data(
    search_results: &[YouTubeSearchResult],
    pipeline_config: &YouTubePipelineConfig,
    config: &FetchiumConfig,
    http: &HttpClient,
) -> Vec<VideoAnalysis> {
    use std::sync::Arc;
    use tokio::sync::Semaphore;
    use tokio::task::JoinSet;

    let sem = Arc::new(Semaphore::new(MAX_CONCURRENT_FETCHES));
    let mut tasks: JoinSet<FetchiumResult<VideoAnalysis>> = JoinSet::new();

    for result in search_results.iter().take(pipeline_config.max_videos) {
        let seed = result.clone();
        let config = config.clone();
        let http = http.clone();
        let fetch_transcript_flag = pipeline_config.fetch_transcript;
        let fetch_comments_flag = pipeline_config.fetch_comments;
        let sem = sem.clone();

        tasks.spawn(async move {
            // Acquire slot before fetching; auto-released when _permit drops
            let _permit = sem.acquire_owned().await.ok();
            fetch_single_video_data(
                &seed,
                &config,
                &http,
                fetch_transcript_flag,
                fetch_comments_flag,
            )
            .await
        });
    }

    // Collect results as tasks complete. Use a hard deadline for the whole phase
    // so one straggler doesn't block all already-finished analyses.
    let fetch_deadline =
        tokio::time::Instant::now() + std::time::Duration::from_secs(FETCH_PHASE_TIMEOUT_SECS);
    let mut analyses = Vec::new();
    while !tasks.is_empty() {
        let wait_next = tokio::time::timeout_at(fetch_deadline, tasks.join_next()).await;
        match wait_next {
            Ok(Some(Ok(Ok(analysis)))) => analyses.push(analysis),
            Ok(Some(Ok(Err(e)))) => tracing::debug!("Video analysis failed: {e}"),
            Ok(Some(Err(e))) => tracing::debug!("Video task panicked: {e}"),
            Ok(None) => break,
            Err(_) => {
                tracing::debug!(
                    "Video fetch phase hit {}s deadline; aborting remaining {} task(s)",
                    FETCH_PHASE_TIMEOUT_SECS,
                    tasks.len()
                );
                tasks.abort_all();
                break;
            }
        }
    }

    analyses
}

/// Fetch metadata + transcript + comments for a single video.
async fn fetch_single_video_data(
    seed: &YouTubeSearchResult,
    config: &FetchiumConfig,
    http: &HttpClient,
    fetch_transcript_flag: bool,
    fetch_comments_flag: bool,
) -> FetchiumResult<VideoAnalysis> {
    let video_id = &seed.video_id;
    // Parallel fetch: metadata + transcript + comments
    let (meta_result, transcript_result, comments_result) = tokio::join!(
        metadata::fetch_metadata(video_id, http, config),
        async {
            if fetch_transcript_flag {
                transcript::fetch_transcript(video_id, http, config)
                    .await
                    .ok()
            } else {
                None
            }
        },
        async {
            if fetch_comments_flag {
                comments::fetch_comments(video_id, http, config, 100)
                    .await
                    .ok()
            } else {
                None
            }
        }
    );

    let mut meta = meta_result?;
    hydrate_metadata_from_search_seed(&mut meta, seed);
    let credibility = metadata::score_channel_credibility(&meta.channel);
    let comment_analysis = comments_result.map(|c| comments::analyze_comments(&c));

    Ok(VideoAnalysis {
        metadata: meta,
        transcript: transcript_result,
        comments: comment_analysis,
        credibility,
    })
}

fn hydrate_metadata_from_search_seed(meta: &mut VideoMetadata, seed: &YouTubeSearchResult) {
    if meta.title.trim().is_empty() {
        meta.title = seed.title.clone();
    }
    if meta.description.trim().is_empty() {
        meta.description = seed.description.clone();
    }
    if meta.channel.name.trim().is_empty() {
        meta.channel.name = seed.channel.clone();
    }
    if meta.duration_secs == 0 {
        meta.duration_secs = seed.duration_secs;
    }
    if meta.view_count == 0 {
        meta.view_count = seed.view_count;
    }
    if meta.published.trim().is_empty() {
        meta.published = seed.published.clone();
    }
    if meta.thumbnail_url.is_none() {
        meta.thumbnail_url = seed.thumbnail_url.clone();
    }
}

/// Intelligence pipeline output: fact checks, timeline, learning path, teaching.
type IntelligenceOutput = (
    Option<Vec<FactCheckResult>>,
    Option<Vec<TimelineEntry>>,
    Option<Vec<LearningStep>>,
    Option<TeachingContent>,
);

/// Run optional intelligence features.
fn run_intelligence(
    analyses: &[VideoAnalysis],
    config: &YouTubePipelineConfig,
) -> IntelligenceOutput {
    let fact_checks = if config.fact_check {
        Some(intelligence::cross_check_facts(analyses, 0.4))
    } else {
        None
    };

    let timeline = if config.build_timeline {
        Some(intelligence::build_topic_timeline(analyses))
    } else {
        None
    };

    let learning_path = if config.build_learning_path {
        Some(intelligence::generate_learning_path(analyses))
    } else {
        None
    };

    let teaching = if config.generate_teaching {
        Some(intelligence::generate_teaching(analyses))
    } else {
        None
    };

    (fact_checks, timeline, learning_path, teaching)
}

/// Format pipeline result as markdown.
pub fn format_result_markdown(result: &YouTubePipelineResult) -> String {
    let mut out = String::new();
    out.push_str(&format!("# YouTube Intelligence: {}\n\n", result.query));
    out.push_str(&format!(
        "**Analyzed {} videos** in {}ms\n\n",
        result.videos.len(),
        result.duration_ms
    ));

    // Rankings
    out.push_str("## Video Rankings (VideoFusion)\n\n");
    out.push_str("| # | Title | Score | Educational | Clickbait |\n");
    out.push_str("|---|-------|-------|-------------|----------|\n");
    for (i, r) in result.rankings.iter().enumerate() {
        out.push_str(&format!(
            "| {} | {} | {:.2} | {:.2} | {:.2} |\n",
            i + 1,
            truncate(&r.title, 50),
            r.final_score,
            r.educational_score,
            r.clickbait_score,
        ));
    }

    // Video details
    out.push_str("\n## Video Details\n\n");
    for video in &result.videos {
        let meta = &video.metadata;
        out.push_str(&format!("### {}\n", meta.title));
        out.push_str(&format!(
            "- **URL:** https://www.youtube.com/watch?v={}\n",
            meta.video_id
        ));
        out.push_str(&format!("- **Channel:** {}\n", meta.channel.name));
        out.push_str(&format!(
            "- **Views:** {} | **Likes:** {}\n",
            format_number(meta.view_count),
            format_number(meta.like_count)
        ));
        out.push_str(&format!(
            "- **Duration:** {}\n",
            format_duration(meta.duration_secs)
        ));
        out.push_str(&format!("- **Published:** {}\n", meta.published));
        out.push_str(&format!(
            "- **Credibility:** {:.0}% ({:?})\n",
            video.credibility.score * 100.0,
            video.credibility.tier
        ));

        if !meta.chapters.is_empty() {
            out.push_str("- **Chapters:** ");
            let ch_names: Vec<&str> = meta.chapters.iter().map(|c| c.title.as_str()).collect();
            out.push_str(&ch_names.join(", "));
            out.push('\n');
        }

        if let Some(ref t) = video.transcript {
            out.push_str(&format!(
                "- **Transcript:** {} words ({:?})\n",
                t.word_count, t.source
            ));
            if !t.key_moments.is_empty() {
                out.push_str(&format!("- **Key moments:** {}\n", t.key_moments.len()));
            }
        }

        if let Some(ref c) = video.comments {
            out.push_str(&format!(
                "- **Comments:** {} analyzed | Sentiment: {:.0}% positive\n",
                c.analyzed_comments,
                c.overall_sentiment.positive * 100.0
            ));
        }

        out.push('\n');
    }

    // Fact checks
    if let Some(ref facts) = result.fact_checks {
        if !facts.is_empty() {
            out.push_str("## Fact Check Results\n\n");
            for fact in facts {
                let icon = match fact.consensus {
                    FactConsensus::Confirmed => "✓",
                    FactConsensus::Disputed => "⚠",
                    FactConsensus::Unverified => "?",
                    FactConsensus::Contradicted => "✗",
                };
                out.push_str(&format!(
                    "- {} **{:?}**: {}\n",
                    icon,
                    fact.consensus,
                    truncate(&fact.claim, 80)
                ));
            }
            out.push('\n');
        }
    }

    // Timeline
    if let Some(ref timeline) = result.timeline {
        if !timeline.is_empty() {
            out.push_str("## Topic Timeline\n\n");
            for entry in timeline {
                out.push_str(&format!(
                    "- **{}** — {} ({})\n",
                    entry.date, entry.event, entry.video_title
                ));
            }
            out.push('\n');
        }
    }

    // Learning path
    if let Some(ref path) = result.learning_path {
        if !path.is_empty() {
            out.push_str("## Learning Path\n\n");
            for step in path {
                out.push_str(&format!(
                    "{}. **{}** ({:?}, ~{} min)\n   Concepts: {}\n",
                    step.order,
                    step.title,
                    step.difficulty,
                    step.estimated_minutes,
                    step.key_concepts.join(", "),
                ));
            }
            out.push('\n');
        }
    }

    // Teaching
    if let Some(ref teaching) = result.teaching {
        out.push_str("## Teaching Content\n\n");
        if !teaching.flashcards.is_empty() {
            out.push_str("### Flashcards\n\n");
            for card in &teaching.flashcards {
                out.push_str(&format!("**Q:** {}\n**A:** {}\n\n", card.front, card.back));
            }
        }
        if !teaching.quiz_questions.is_empty() {
            out.push_str("### Quiz\n\n");
            for (i, q) in teaching.quiz_questions.iter().enumerate() {
                out.push_str(&format!("**{}. {}**\n", i + 1, q.question));
                for (j, opt) in q.options.iter().enumerate() {
                    let marker = if j == q.correct_index { "→" } else { " " };
                    out.push_str(&format!(
                        "  {} {}. {}\n",
                        marker,
                        (b'A' + j as u8) as char,
                        opt
                    ));
                }
                out.push('\n');
            }
        }
    }

    out
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        // Find safe char boundary to avoid panics on multi-byte chars
        let mut idx = max;
        while idx > 0 && !s.is_char_boundary(idx) {
            idx -= 1;
        }
        format!("{}...", &s[..idx])
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

fn format_duration(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_number_variants() {
        assert_eq!(format_number(500), "500");
        assert_eq!(format_number(1500), "1.5K");
        assert_eq!(format_number(2_500_000), "2.5M");
    }

    #[test]
    fn format_duration_variants() {
        assert_eq!(format_duration(65), "1:05");
        assert_eq!(format_duration(3661), "1:01:01");
        assert_eq!(format_duration(0), "0:00");
    }

    #[test]
    fn truncate_short() {
        assert_eq!(truncate("Hello", 10), "Hello");
    }

    #[test]
    fn truncate_long() {
        let result = truncate("This is a very long title", 10);
        assert!(result.len() <= 13); // 10 + "..."
    }

    #[test]
    fn pipeline_config_defaults() {
        let cfg = YouTubePipelineConfig::default();
        assert_eq!(cfg.max_videos, 5);
        assert!(cfg.fetch_transcript);
        assert!(!cfg.fact_check);
    }

    #[test]
    fn format_empty_result() {
        let result = YouTubePipelineResult {
            query: "test query".into(),
            videos: vec![],
            rankings: vec![],
            fact_checks: None,
            timeline: None,
            learning_path: None,
            teaching: None,
            duration_ms: 100,
        };
        let md = format_result_markdown(&result);
        assert!(md.contains("test query"));
        assert!(md.contains("0 videos"));
    }
}
