//! Unified Social Research Engine — main orchestrator.
//!
//! Runs all platform pipelines in parallel and fuses results.

use crate::config::HsxConfig;
use crate::error::HsxResult;
use crate::http::client::HttpClient;
use crate::social::{
    facebook::{pipeline as fb_pipeline, types::FacebookPipelineConfig},
    hackernews,
    reddit::{pipeline as reddit_pipeline, types::RedditPipelineConfig},
    tiktok::{pipeline as tiktok_pipeline, types::TikTokPipelineConfig},
    twitter::{pipeline as twitter_pipeline, types::TwitterPipelineConfig},
    types::*,
    unified::{ideas, trend},
};
use crate::youtube::{pipeline as yt_pipeline, types::YouTubePipelineConfig};
use std::collections::HashMap;
use std::time::Instant;

/// Unified social research pipeline runner.
pub struct SocialPipelineRunner<'a> {
    pub config: &'a HsxConfig,
    pub http: &'a HttpClient,
}

impl<'a> SocialPipelineRunner<'a> {
    pub fn new(config: &'a HsxConfig, http: &'a HttpClient) -> Self {
        Self { config, http }
    }

    /// Run the full unified pipeline.
    pub async fn run(
        &self,
        pipeline_config: &SocialPipelineConfig,
    ) -> HsxResult<SocialResearchResult> {
        run_social_pipeline(pipeline_config, self.config, self.http).await
    }
}

/// Execute the complete unified social research pipeline.
///
/// All platform fetches run in parallel via `tokio::join!`.
pub async fn run_social_pipeline(
    cfg: &SocialPipelineConfig,
    hsx_cfg: &HsxConfig,
    http: &HttpClient,
) -> HsxResult<SocialResearchResult> {
    let started = Instant::now();
    let query = &cfg.query;
    let max = cfg.max_posts_per_platform;

    // ─── Parallel platform fetches ────────────────────────────────
    let run_twitter = cfg.platforms.contains(&SocialPlatform::Twitter);
    let run_reddit = cfg.platforms.contains(&SocialPlatform::Reddit);
    let run_tiktok = cfg.platforms.contains(&SocialPlatform::TikTok);
    let run_hn = cfg.platforms.contains(&SocialPlatform::HackerNews);
    let run_yt = cfg.platforms.contains(&SocialPlatform::YouTube);
    let run_fb = cfg.platforms.contains(&SocialPlatform::Facebook);

    let (twitter_res, reddit_res, tiktok_res, hn_res, yt_res, fb_res) = tokio::join!(
        async {
            if run_twitter {
                let cfg_tw = TwitterPipelineConfig {
                    query: query.clone(),
                    max_tweets: max,
                    fetch_trends: cfg.include_trends,
                    searxng_url: hsx_cfg.search.searxng_url.clone(),
                    ..Default::default()
                };
                Some(twitter_pipeline::run_twitter_pipeline(&cfg_tw, hsx_cfg, http).await)
            } else {
                None
            }
        },
        async {
            if run_reddit {
                let cfg_rd = RedditPipelineConfig {
                    query: query.clone(),
                    max_posts: max,
                    ..Default::default()
                };
                Some(reddit_pipeline::run_reddit_pipeline(&cfg_rd, hsx_cfg, http).await)
            } else {
                None
            }
        },
        async {
            if run_tiktok {
                let cfg_tt = TikTokPipelineConfig {
                    query: query.clone(),
                    max_videos: max,
                    fetch_trends: cfg.include_trends,
                    ..Default::default()
                };
                Some(tiktok_pipeline::run_tiktok_pipeline(&cfg_tt, hsx_cfg, http).await)
            } else {
                None
            }
        },
        async {
            if run_hn {
                Some(hackernews::search_stories(query, max, http, cfg.timeout_secs).await)
            } else {
                None
            }
        },
        async {
            if run_yt {
                let cfg_yt = YouTubePipelineConfig {
                    query: query.clone(),
                    max_videos: (max / 3).max(3),
                    fetch_transcript: false,
                    fetch_comments: false,
                    fact_check: false,
                    ..Default::default()
                };
                Some(yt_pipeline::run_youtube_pipeline(&cfg_yt, hsx_cfg, http).await)
            } else {
                None
            }
        },
        async {
            if run_fb {
                let cfg_fb = FacebookPipelineConfig {
                    query: query.clone(),
                    max_results: max,
                    graph_api_token: None,
                    timeout_secs: cfg.timeout_secs,
                    searxng_url: hsx_cfg.search.searxng_url.clone(),
                };
                Some(fb_pipeline::run_facebook_pipeline(&cfg_fb, http).await)
            } else {
                None
            }
        }
    );

    // ─── Normalise to SocialPost ──────────────────────────────────
    let mut platform_posts: HashMap<SocialPlatform, Vec<SocialPost>> = HashMap::new();
    let mut platform_trends_map: HashMap<SocialPlatform, Vec<TrendItem>> = HashMap::new();
    let mut platform_results: HashMap<String, PlatformResult> = HashMap::new();

    // Twitter
    if let Some(Ok(tw)) = twitter_res {
        let posts: Vec<SocialPost> = tw.tweets.iter().map(tweet_to_social_post).collect();
        let trends: Vec<TrendItem> = tw
            .trends
            .iter()
            .map(|tr| twitter_trend_to_trend_item(tr, &posts))
            .collect();

        let stats = compute_platform_stats(SocialPlatform::Twitter, &posts);
        platform_trends_map.insert(SocialPlatform::Twitter, trends.clone());
        platform_results.insert(
            "twitter".into(),
            PlatformResult {
                platform: SocialPlatform::Twitter,
                posts: posts.clone(),
                trends,
                stats,
            },
        );
        platform_posts.insert(SocialPlatform::Twitter, posts);
    }

    // Reddit
    if let Some(Ok(rd)) = reddit_res {
        let posts: Vec<SocialPost> = rd.posts.iter().map(reddit_post_to_social_post).collect();
        let trends: Vec<TrendItem> = rd
            .analysis
            .top_subreddits
            .iter()
            .take(5)
            .map(|(sub, count)| TrendItem {
                platform: SocialPlatform::Reddit,
                topic: format!("r/{sub}"),
                post_count: *count as u64,
                velocity: 10.0,
                sentiment: 0.0,
                region: None,
                related_topics: Vec::new(),
                sample_posts: posts.iter().take(3).cloned().collect(),
            })
            .collect();

        let stats = compute_platform_stats(SocialPlatform::Reddit, &posts);
        platform_trends_map.insert(SocialPlatform::Reddit, trends.clone());
        platform_results.insert(
            "reddit".into(),
            PlatformResult {
                platform: SocialPlatform::Reddit,
                posts: posts.clone(),
                trends,
                stats,
            },
        );
        platform_posts.insert(SocialPlatform::Reddit, posts);
    }

    // TikTok
    if let Some(Ok(tt)) = tiktok_res {
        let posts: Vec<SocialPost> = tt.videos.iter().map(tiktok_video_to_social_post).collect();
        let trends: Vec<TrendItem> = tt
            .trends
            .iter()
            .map(|tr| tiktok_trend_to_trend_item(tr, &posts))
            .collect();

        let stats = compute_platform_stats(SocialPlatform::TikTok, &posts);
        platform_trends_map.insert(SocialPlatform::TikTok, trends.clone());
        platform_results.insert(
            "tiktok".into(),
            PlatformResult {
                platform: SocialPlatform::TikTok,
                posts: posts.clone(),
                trends,
                stats,
            },
        );
        platform_posts.insert(SocialPlatform::TikTok, posts);
    }

    // HackerNews
    if let Some(Ok(hn_stories)) = hn_res {
        let posts: Vec<SocialPost> = hn_stories.iter().map(hackernews::to_social_post).collect();
        let stats = compute_platform_stats(SocialPlatform::HackerNews, &posts);
        platform_results.insert(
            "hackernews".into(),
            PlatformResult {
                platform: SocialPlatform::HackerNews,
                posts: posts.clone(),
                trends: Vec::new(),
                stats,
            },
        );
        platform_posts.insert(SocialPlatform::HackerNews, posts);
    }

    // YouTube
    if let Some(Ok(yt)) = yt_res {
        let posts: Vec<SocialPost> = yt.rankings.iter().map(yt_ranking_to_social_post).collect();
        let stats = compute_platform_stats(SocialPlatform::YouTube, &posts);
        platform_results.insert(
            "youtube".into(),
            PlatformResult {
                platform: SocialPlatform::YouTube,
                posts: posts.clone(),
                trends: Vec::new(),
                stats,
            },
        );
        platform_posts.insert(SocialPlatform::YouTube, posts);
    }

    // Facebook
    if let Some(Ok(fb)) = fb_res {
        let posts: Vec<SocialPost> = fb
            .posts
            .iter()
            .map(fb_post_to_social_post)
            .chain(fb.pages.iter().map(fb_page_to_social_post))
            .collect();
        let stats = compute_platform_stats(SocialPlatform::Facebook, &posts);
        platform_results.insert(
            "facebook".into(),
            PlatformResult {
                platform: SocialPlatform::Facebook,
                posts: posts.clone(),
                trends: Vec::new(),
                stats,
            },
        );
        platform_posts.insert(SocialPlatform::Facebook, posts);
    }

    // ─── Cross-platform analysis ──────────────────────────────────
    let mut cross_platform_trends = trend::detect_cross_platform_trends(&platform_trends_map, 0.25);

    // Merge and deduplicate all posts
    let mut top_posts = trend::merge_and_dedup_posts(&platform_posts, 0.70);
    top_posts.truncate(50);

    // Cross-platform validation: boost authenticity of posts confirmed on 2+ platforms.
    crate::social::validate::cross_validate_boost(&mut top_posts);

    // Generate content ideas
    let content_ideas = if cfg.generate_ideas {
        let ideas = ideas::generate_ideas(&cross_platform_trends, &top_posts);
        // Inject back into trends
        for trend in &mut cross_platform_trends {
            let matching: Vec<crate::social::types::ContentIdea> = ideas
                .iter()
                .filter(|i| {
                    let topic_lower = trend.topic.to_lowercase();
                    i.title.to_lowercase().contains(&topic_lower)
                })
                .take(2)
                .cloned()
                .collect();
            trend.content_ideas = matching;
        }
        ideas
    } else {
        Vec::new()
    };

    // ─── Build summary ────────────────────────────────────────────
    let total_posts: usize = platform_posts.values().map(|v| v.len()).sum();
    let viral_topics: Vec<String> = cross_platform_trends
        .iter()
        .filter(|t| t.is_viral)
        .take(3)
        .map(|t| t.topic.clone())
        .collect();

    let summary = build_summary(query, total_posts, &cross_platform_trends, &viral_topics);

    Ok(SocialResearchResult {
        query: query.clone(),
        platform_results,
        cross_platform_trends,
        top_posts,
        content_ideas,
        summary,
        duration_ms: started.elapsed().as_millis() as u64,
    })
}

// ─── Normalisation helpers ────────────────────────────────────────

fn tweet_to_social_post(tweet: &crate::social::twitter::types::Tweet) -> SocialPost {
    let mut eng = EngagementMetrics {
        likes: tweet.likes,
        shares: tweet.retweets,
        comments: tweet.replies,
        views: tweet.views,
        score: 0.0,
    };
    eng.compute_score();
    SocialPost {
        platform: SocialPlatform::Twitter,
        id: tweet.id.clone(),
        url: tweet.url.clone(),
        author: tweet.author.username.clone(),
        content: tweet.text.clone(),
        published: tweet.published.clone(),
        engagement: eng,
        media: Vec::new(),
        hashtags: tweet.hashtags.clone(),
        mentions: tweet.mentions.clone(),
        sentiment: score_sentiment(&tweet.text),
        authenticity: 0.7,
    }
}

fn reddit_post_to_social_post(post: &crate::social::reddit::types::RedditPost) -> SocialPost {
    let mut eng = EngagementMetrics {
        likes: post.score.max(0) as u64,
        shares: post.crossposts as u64,
        comments: post.num_comments,
        views: None,
        score: 0.0,
    };
    eng.compute_score();
    SocialPost {
        platform: SocialPlatform::Reddit,
        id: post.id.clone(),
        url: post.url.clone(),
        author: post.author.clone(),
        content: format!("{} {}", post.title, post.selftext),
        published: (post.created_utc as u64).to_string(),
        engagement: eng,
        media: Vec::new(),
        hashtags: Vec::new(),
        mentions: Vec::new(),
        sentiment: score_sentiment(&post.title),
        authenticity: 0.85,
    }
}

fn tiktok_video_to_social_post(video: &crate::social::tiktok::types::TikTokVideo) -> SocialPost {
    let mut eng = EngagementMetrics {
        likes: video.likes,
        shares: video.shares,
        comments: video.comments,
        views: Some(video.plays),
        score: 0.0,
    };
    eng.compute_score();
    SocialPost {
        platform: SocialPlatform::TikTok,
        id: video.id.clone(),
        url: video.url.clone(),
        author: video.author.username.clone(),
        content: video.description.clone(),
        published: video.published.clone(),
        engagement: eng,
        media: Vec::new(),
        hashtags: video.hashtags.clone(),
        mentions: Vec::new(),
        sentiment: score_sentiment(&video.description),
        authenticity: 0.6,
    }
}

fn twitter_trend_to_trend_item(
    tr: &crate::social::twitter::types::TwitterTrend,
    posts: &[SocialPost],
) -> TrendItem {
    TrendItem {
        platform: SocialPlatform::Twitter,
        topic: tr.topic.clone(),
        post_count: tr.tweet_volume.unwrap_or(0),
        velocity: tr.tweet_volume.unwrap_or(0) as f64 / 24.0,
        sentiment: 0.0,
        region: Some(tr.region.clone()),
        related_topics: Vec::new(),
        sample_posts: posts.iter().take(3).cloned().collect(),
    }
}

fn tiktok_trend_to_trend_item(
    tr: &crate::social::tiktok::types::TikTokTrend,
    posts: &[SocialPost],
) -> TrendItem {
    TrendItem {
        platform: SocialPlatform::TikTok,
        topic: tr.hashtag.clone(),
        post_count: tr.view_count,
        velocity: tr.view_count as f64 / 168.0, // weekly → hourly
        sentiment: 0.0,
        region: None,
        related_topics: Vec::new(),
        sample_posts: posts.iter().take(3).cloned().collect(),
    }
}

fn fb_post_to_social_post(post: &crate::social::facebook::types::FacebookPost) -> SocialPost {
    let mut eng = EngagementMetrics {
        likes: post.likes,
        shares: post.shares,
        comments: post.comments,
        views: None,
        score: 0.0,
    };
    eng.compute_score();
    SocialPost {
        platform: SocialPlatform::Facebook,
        id: post.id.clone(),
        url: post.url.clone(),
        author: post.page_name.clone(),
        content: post.message.clone(),
        published: post.published.clone(),
        engagement: eng,
        media: Vec::new(),
        hashtags: Vec::new(),
        mentions: Vec::new(),
        sentiment: score_sentiment(&post.message),
        authenticity: 0.75,
    }
}

fn fb_page_to_social_post(page: &crate::social::facebook::types::FacebookPage) -> SocialPost {
    let mut eng = EngagementMetrics {
        likes: page.likes.unwrap_or(0),
        shares: 0,
        comments: 0,
        views: page.followers,
        score: 0.0,
    };
    eng.compute_score();
    SocialPost {
        platform: SocialPlatform::Facebook,
        id: page.id.clone(),
        url: page.url.clone(),
        author: page.name.clone(),
        content: format!("{}: {}", page.name, page.about),
        published: String::new(),
        engagement: eng,
        media: Vec::new(),
        hashtags: Vec::new(),
        mentions: Vec::new(),
        sentiment: score_sentiment(&page.about),
        authenticity: 0.80,
    }
}

fn yt_ranking_to_social_post(r: &crate::youtube::types::VideoRanking) -> SocialPost {
    let mut eng = EngagementMetrics {
        likes: 0,
        shares: 0,
        comments: 0,
        views: None,
        score: r.final_score,
    };
    eng.score = r.final_score;
    SocialPost {
        platform: SocialPlatform::YouTube,
        id: r.video_id.clone(),
        url: format!("https://www.youtube.com/watch?v={}", r.video_id),
        author: String::new(),
        content: r.title.clone(),
        published: String::new(),
        engagement: eng,
        media: Vec::new(),
        hashtags: Vec::new(),
        mentions: Vec::new(),
        sentiment: 0.0,
        authenticity: 0.8,
    }
}

fn compute_platform_stats(platform: SocialPlatform, posts: &[SocialPost]) -> PlatformStats {
    let avg_engagement = if posts.is_empty() {
        0.0
    } else {
        posts.iter().map(|p| p.engagement.score).sum::<f64>() / posts.len() as f64
    };

    let mut tag_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for post in posts {
        for tag in &post.hashtags {
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }
    let mut top_hashtags: Vec<(String, usize)> = tag_counts.into_iter().collect();
    top_hashtags.sort_by(|a, b| b.1.cmp(&a.1));
    top_hashtags.truncate(5);

    let scores: Vec<f64> = posts.iter().map(|p| p.sentiment).collect();
    let n = scores.len().max(1) as f64;
    let pos = scores.iter().filter(|&&s| s > 0.1).count() as f64 / n;
    let neg = scores.iter().filter(|&&s| s < -0.1).count() as f64 / n;
    let compound = scores.iter().sum::<f64>() / n;

    PlatformStats {
        platform,
        posts_analyzed: posts.len(),
        top_authors: Vec::new(),
        top_hashtags,
        sentiment: SentimentBreakdown {
            positive_pct: pos,
            negative_pct: neg,
            neutral_pct: 1.0 - pos - neg,
            compound,
        },
        avg_engagement,
    }
}

fn build_summary(
    query: &str,
    total_posts: usize,
    trends: &[CrossPlatformTrend],
    viral_topics: &[String],
) -> String {
    let viral_str = if viral_topics.is_empty() {
        "No viral bursts detected.".to_string()
    } else {
        format!("Viral topics: {}", viral_topics.join(", "))
    };

    format!(
        "Analysed {} posts about \"{query}\" across {} platforms. \
         Found {} cross-platform trends. {}",
        total_posts,
        5,
        trends.len(),
        viral_str
    )
}

/// Format a SocialResearchResult as a markdown report.
pub fn format_markdown(result: &SocialResearchResult) -> String {
    let mut out = String::new();

    out.push_str(&format!("# Unified Social Research: {}\n\n", result.query));
    out.push_str(&format!("{}\n\n", result.summary));
    out.push_str(&format!("`{}ms` total\n\n", result.duration_ms));

    // Cross-platform trends
    if !result.cross_platform_trends.is_empty() {
        out.push_str("## Cross-Platform Trends\n\n");
        for trend in result.cross_platform_trends.iter().take(10) {
            let platform_str: Vec<String> = trend.platforms.iter().map(|p| p.to_string()).collect();
            out.push_str(&format!(
                "### {} {}\n**Platforms:** {} | **Engagement:** {} | **Velocity:** {:.0}/hr\n\n",
                if trend.is_viral { "🔥" } else { "📈" },
                trend.topic,
                platform_str.join(", "),
                trend.total_engagement,
                trend.velocity
            ));
        }
    }

    // Per-platform summary
    out.push_str("## Platform Breakdown\n\n");
    for (name, result) in &result.platform_results {
        out.push_str(&format!(
            "- **{}**: {} posts, avg engagement {:.2}\n",
            name, result.stats.posts_analyzed, result.stats.avg_engagement
        ));
    }
    out.push('\n');

    // Content ideas
    if !result.content_ideas.is_empty() {
        out.push_str(&ideas::format_ideas_markdown(&result.content_ideas));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_summary_no_viral() {
        let s = build_summary("rust", 100, &[], &[]);
        assert!(s.contains("rust"));
        assert!(s.contains("100 posts"));
    }

    #[test]
    fn compute_platform_stats_empty() {
        let stats = compute_platform_stats(SocialPlatform::Reddit, &[]);
        assert_eq!(stats.posts_analyzed, 0);
        assert_eq!(stats.avg_engagement, 0.0);
    }

    #[test]
    fn build_summary_with_viral_trends() {
        use crate::social::types::{CrossPlatformTrend, SocialPlatform as SP};
        let trend = CrossPlatformTrend {
            topic: "AI".into(),
            platforms: vec![SP::Twitter, SP::Reddit],
            total_engagement: 50_000,
            velocity: 200.0,
            sentiment: 0.4,
            is_viral: true,
            content_ideas: Vec::new(),
            sample_posts: Vec::new(),
        };
        let s = build_summary("AI tools", 50, &[trend], &[]);
        assert!(s.contains("AI tools"));
        assert!(s.contains("50 posts"));
        assert!(s.contains("AI"), "should mention viral trend topic");
    }

    #[test]
    fn compute_platform_stats_with_posts() {
        use crate::social::types::{EngagementMetrics, SocialPost};
        let make_post = |platform: SocialPlatform| {
            let mut e = EngagementMetrics {
                likes: 500,
                shares: 100,
                comments: 50,
                views: None,
                score: 0.0,
            };
            e.compute_score();
            SocialPost {
                platform,
                id: "1".into(),
                url: "https://example.com".into(),
                author: "user".into(),
                content: "great content about programming".into(),
                published: String::new(),
                engagement: e,
                media: Vec::new(),
                hashtags: vec!["#rust".into()],
                mentions: Vec::new(),
                sentiment: 0.5,
                authenticity: 0.9,
            }
        };
        let posts = vec![
            make_post(SocialPlatform::Reddit),
            make_post(SocialPlatform::Reddit),
        ];
        let stats = compute_platform_stats(SocialPlatform::Reddit, &posts);
        assert_eq!(stats.posts_analyzed, 2);
        assert!(stats.avg_engagement > 0.0);
        assert!(!stats.top_hashtags.is_empty());
        assert_eq!(stats.top_hashtags[0].0, "#rust");
    }
}
