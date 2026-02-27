//! Twitter/X intelligence pipeline — full orchestration.

use crate::config::HsxConfig;
use crate::error::HsxResult;
use crate::http::client::HttpClient;
use crate::social::twitter::{analysis, search, types::*};
use std::time::Instant;

/// Run the full Twitter/X intelligence pipeline.
pub async fn run_twitter_pipeline(
    config_tw: &TwitterPipelineConfig,
    _config: &HsxConfig,
    http: &HttpClient,
) -> HsxResult<TwitterPipelineResult> {
    let started = Instant::now();

    // Parallel: search tweets + fetch trends
    let (tweets_res, trends_res) = tokio::join!(
        search::search_tweets(&config_tw.query, config_tw.max_tweets, config_tw, http),
        async {
            if config_tw.fetch_trends {
                search::fetch_trends(config_tw, http)
                    .await
                    .unwrap_or_default()
            } else {
                Vec::new()
            }
        }
    );

    let mut tweets = tweets_res.unwrap_or_default();

    // Thread reconstruction (optional)
    let threads = if config_tw.reconstruct_threads {
        analysis::reconstruct_threads(&tweets)
    } else {
        Vec::new()
    };

    // Analysis
    let mut tweet_analysis = analysis::analyse_tweets(&tweets);
    tweet_analysis.threads = threads;

    // Score and sort by viral potential
    tweets.sort_by(|a, b| {
        let va = analysis::viral_score_tweet(a).overall;
        let vb = analysis::viral_score_tweet(b).overall;
        vb.partial_cmp(&va).unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(TwitterPipelineResult {
        query: config_tw.query.clone(),
        tweets,
        trends: trends_res,
        analysis: tweet_analysis,
        duration_ms: started.elapsed().as_millis() as u64,
    })
}

/// Format a TwitterPipelineResult as a markdown report.
pub fn format_markdown(result: &TwitterPipelineResult) -> String {
    let mut out = String::new();

    out.push_str(&format!("# Twitter/X Intelligence: {}\n\n", result.query));
    out.push_str(&format!(
        "**{} tweets analysed** | **{} trends** | `{}ms`\n\n",
        result.tweets.len(),
        result.trends.len(),
        result.duration_ms
    ));

    // Sentiment
    let s = &result.analysis.sentiment;
    out.push_str("## Sentiment\n\n");
    out.push_str(&format!(
        "| Positive | Negative | Neutral | Compound |\n|----------|----------|---------|----------|\n| {:.0}% | {:.0}% | {:.0}% | {:.2} |\n\n",
        s.positive_pct * 100.0,
        s.negative_pct * 100.0,
        s.neutral_pct * 100.0,
        s.compound
    ));

    // Top hashtags
    if !result.analysis.top_hashtags.is_empty() {
        out.push_str("## Top Hashtags\n\n");
        for (tag, count) in result.analysis.top_hashtags.iter().take(10) {
            out.push_str(&format!("- `{tag}` × {count}\n"));
        }
        out.push('\n');
    }

    // Trending topics
    if !result.trends.is_empty() {
        out.push_str("## Trending Topics\n\n");
        for trend in result.trends.iter().take(10) {
            out.push_str(&format!("- **{}**", trend.topic));
            if let Some(vol) = trend.tweet_volume {
                out.push_str(&format!(" ({vol} tweets)"));
            }
            out.push('\n');
        }
        out.push('\n');
    }

    // Top viral tweets
    if !result.analysis.viral_tweets.is_empty() {
        out.push_str("## Most Viral Tweets\n\n");
        for tweet in result.analysis.viral_tweets.iter().take(5) {
            out.push_str(&format!(
                "**@{}** · {} · ❤ {} · 🔁 {}\n> {}\n\n",
                tweet.author.username,
                tweet.published,
                tweet.likes,
                tweet.retweets,
                tweet.text.chars().take(200).collect::<String>()
            ));
        }
    }

    out
}
