//! Reddit intelligence pipeline — full orchestration.

use crate::config::FetchiumConfig;
use crate::error::FetchiumResult;
use crate::http::client::HttpClient;
use crate::social::reddit::{analysis, search, types::*};
use std::time::Instant;

/// Run the full Reddit intelligence pipeline.
pub async fn run_reddit_pipeline(
    config_rd: &RedditPipelineConfig,
    _config: &FetchiumConfig,
    http: &HttpClient,
) -> FetchiumResult<RedditPipelineResult> {
    let started = Instant::now();

    // Search posts
    let mut posts = search::search_posts(&config_rd.query, config_rd, http)
        .await
        .unwrap_or_default();

    // Fetch subreddit hot posts in parallel if subreddits are specified
    if !config_rd.subreddits.is_empty() {
        let extra = search::fetch_multi_subreddit_hot(
            &config_rd.subreddits,
            config_rd.max_posts / config_rd.subreddits.len().max(1),
            http,
            config_rd.timeout_secs,
        )
        .await;
        posts.extend(extra);
        // Dedup by ID
        posts.sort_by(|a, b| a.id.cmp(&b.id));
        posts.dedup_by(|a, b| a.id == b.id);
    }

    // Fetch subreddit stats (parallel)
    let subreddit_stats = {
        let mut handles = Vec::new();
        let subreddits: Vec<String> = posts
            .iter()
            .map(|p| p.subreddit.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .take(5)
            .collect();

        for sub in subreddits {
            let http2 = http.clone();
            let timeout = config_rd.timeout_secs;
            let handle =
                tokio::spawn(
                    async move { search::fetch_subreddit_info(&sub, &http2, timeout).await },
                );
            handles.push(handle);
        }

        let mut stats = Vec::new();
        for h in handles {
            if let Ok(Ok(s)) = h.await {
                stats.push(s);
            }
        }
        stats
    };

    // Score and sort by viral potential
    posts.sort_by(|a, b| {
        let va = analysis::viral_score_post(a);
        let vb = analysis::viral_score_post(b);
        vb.partial_cmp(&va).unwrap_or(std::cmp::Ordering::Equal)
    });

    let post_analysis = analysis::analyse_posts(&posts);

    Ok(RedditPipelineResult {
        query: config_rd.query.clone(),
        posts,
        analysis: post_analysis,
        subreddit_stats,
        duration_ms: started.elapsed().as_millis() as u64,
    })
}

/// Format a RedditPipelineResult as a markdown report.
pub fn format_markdown(result: &RedditPipelineResult) -> String {
    let mut out = String::new();

    out.push_str(&format!("# Reddit Intelligence: {}\n\n", result.query));
    out.push_str(&format!(
        "**{} posts analysed** | `{}ms`\n\n",
        result.posts.len(),
        result.duration_ms
    ));

    // Sentiment
    let s = &result.analysis.sentiment;
    out.push_str("## Sentiment\n\n");
    out.push_str(&format!(
        "| Positive | Negative | Neutral |\n|----------|----------|---------|\n| {:.0}% | {:.0}% | {:.0}% |\n\n",
        s.positive_pct * 100.0,
        s.negative_pct * 100.0,
        s.neutral_pct * 100.0
    ));

    // Top subreddits
    if !result.analysis.top_subreddits.is_empty() {
        out.push_str("## Top Subreddits\n\n");
        for (sub, count) in result.analysis.top_subreddits.iter().take(10) {
            out.push_str(&format!("- r/{sub} ({count} posts)\n"));
        }
        out.push('\n');
    }

    // Top posts
    if !result.posts.is_empty() {
        out.push_str("## Top Posts\n\n");
        for post in result.posts.iter().take(10) {
            out.push_str(&format!(
                "**[{}]({})** — r/{} | ↑{} | 💬{}\n",
                post.title, post.url, post.subreddit, post.score, post.num_comments
            ));
            if !post.selftext.is_empty() {
                out.push_str(&format!(
                    "> {}\n",
                    post.selftext.chars().take(150).collect::<String>()
                ));
            }
            out.push('\n');
        }
    }

    out
}
