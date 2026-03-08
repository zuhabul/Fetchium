//! TikTok intelligence pipeline — full orchestration.

use crate::config::FetchiumConfig;
use crate::error::FetchiumResult;
use crate::http::client::HttpClient;
use crate::social::tiktok::{analysis, search, types::*};
use std::time::Instant;

/// Run the full TikTok intelligence pipeline.
pub async fn run_tiktok_pipeline(
    config_tt: &TikTokPipelineConfig,
    _config: &FetchiumConfig,
    http: &HttpClient,
) -> FetchiumResult<TikTokPipelineResult> {
    let started = Instant::now();

    // Parallel: search videos + fetch trends
    let (videos_res, trends_res) = tokio::join!(
        search::search_videos(&config_tt.query, config_tt, http),
        async {
            if config_tt.fetch_trends {
                search::fetch_trends(config_tt, http)
                    .await
                    .unwrap_or_default()
            } else {
                Vec::new()
            }
        }
    );

    let mut videos = videos_res.unwrap_or_default();

    // Sort by viral potential
    videos.sort_by(|a, b| {
        let va = analysis::viral_score_video(a).overall;
        let vb = analysis::viral_score_video(b).overall;
        vb.partial_cmp(&va).unwrap_or(std::cmp::Ordering::Equal)
    });

    let video_analysis = analysis::analyse_videos(&videos);

    Ok(TikTokPipelineResult {
        query: config_tt.query.clone(),
        videos,
        trends: trends_res,
        analysis: video_analysis,
        duration_ms: started.elapsed().as_millis() as u64,
    })
}

/// Format a TikTokPipelineResult as a markdown report.
pub fn format_markdown(result: &TikTokPipelineResult) -> String {
    let mut out = String::new();

    out.push_str(&format!("# TikTok Intelligence: {}\n\n", result.query));
    out.push_str(&format!(
        "**{} videos analysed** | **{} trends** | `{}ms`\n\n",
        result.videos.len(),
        result.trends.len(),
        result.duration_ms
    ));

    // Stats
    let a = &result.analysis;
    out.push_str("## Stats\n\n");
    out.push_str(&format!(
        "- Avg plays: **{:.0}**\n- Avg engagement rate: **{:.2}%**\n\n",
        a.avg_plays,
        a.avg_engagement_rate * 100.0
    ));

    // Top hashtags
    if !a.top_hashtags.is_empty() {
        out.push_str("## Top Hashtags\n\n");
        for (tag, count) in a.top_hashtags.iter().take(10) {
            out.push_str(&format!("- `{tag}` × {count}\n"));
        }
        out.push('\n');
    }

    // Trending sounds
    if !a.trending_music.is_empty() {
        out.push_str("## Trending Sounds\n\n");
        for (music, count) in a.trending_music.iter().take(5) {
            out.push_str(&format!("- 🎵 {music} × {count}\n"));
        }
        out.push('\n');
    }

    // Trending hashtag topics
    if !result.trends.is_empty() {
        out.push_str("## Trending Hashtags\n\n");
        for trend in result.trends.iter().take(15) {
            out.push_str(&format!(
                "- `{}` — {} views",
                trend.hashtag, trend.view_count
            ));
            if trend.is_challenge {
                out.push_str(" 🏆 Challenge");
            }
            out.push('\n');
        }
        out.push('\n');
    }

    // Top videos
    if !result.videos.is_empty() {
        out.push_str("## Top Videos\n\n");
        for video in result.videos.iter().take(8) {
            out.push_str(&format!(
                "**@{}** | 👁 {} | ❤ {} | 💬 {}\n> {}\n[Watch]({})\n\n",
                video.author.username,
                video.plays,
                video.likes,
                video.comments,
                video.description.chars().take(150).collect::<String>(),
                video.url
            ));
        }
    }

    out
}
