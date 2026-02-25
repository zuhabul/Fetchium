//! Facebook post/page analysis.

use crate::social::facebook::types::*;
use crate::social::types::score_sentiment;

/// Analyse Facebook posts and produce a FacebookAnalysis.
pub fn analyse_posts(posts: &[FacebookPost], pages: &[FacebookPage]) -> FacebookAnalysis {
    let total_results = posts.len() + pages.len();
    if total_results == 0 {
        return FacebookAnalysis {
            total_results: 0,
            pages: Vec::new(),
            avg_engagement: 0.0,
            top_post_types: Vec::new(),
            viral_posts: Vec::new(),
        };
    }

    // Average engagement (likes + comments + shares)
    let avg_engagement = if posts.is_empty() {
        0.0
    } else {
        posts
            .iter()
            .map(|p| (p.likes + p.comments + p.shares * 2) as f64)
            .sum::<f64>()
            / posts.len() as f64
    };

    // Post type distribution
    let mut type_counts = std::collections::HashMap::<String, usize>::new();
    for post in posts {
        *type_counts.entry(post.post_type.to_string()).or_insert(0) += 1;
    }
    let mut top_post_types: Vec<(String, usize)> = type_counts.into_iter().collect();
    top_post_types.sort_by(|a, b| b.1.cmp(&a.1));

    // Viral posts: sorted by engagement (best effort — may be 0 without API)
    let mut sorted = posts.to_vec();
    sorted.sort_by(|a, b| {
        let ea = a.likes + a.comments + a.shares * 2;
        let eb = b.likes + b.comments + b.shares * 2;
        eb.cmp(&ea)
    });
    let viral_count = (posts.len() / 5).max(3).min(posts.len());
    let viral_posts = sorted.into_iter().take(viral_count).collect();

    FacebookAnalysis {
        total_results,
        pages: pages.to_vec(),
        avg_engagement,
        top_post_types,
        viral_posts,
    }
}

/// Compute viral potential of a Facebook post (0.0–1.0).
pub fn viral_score_post(post: &FacebookPost) -> f64 {
    let engagement = (post.likes + post.comments * 3 + post.shares * 5) as f64;
    let eng_norm = (engagement.ln().max(0.0) / 20.0).min(1.0);
    let sentiment = score_sentiment(&post.message).abs();
    let has_media = post.media_url.is_some()
        || matches!(
            post.post_type,
            FacebookPostType::Video | FacebookPostType::Photo | FacebookPostType::Reel
        );
    let media_boost = if has_media { 0.2 } else { 0.0 };

    (eng_norm * 0.6 + sentiment * 0.2 + media_boost).min(1.0)
}

/// Format Facebook analysis as markdown.
pub fn format_markdown(result: &crate::social::facebook::types::FacebookPipelineResult) -> String {
    let mut out = format!("# Facebook Intelligence: {}\n\n", result.query);
    out.push_str(&format!(
        "> Source: `{:?}` | Duration: {}ms\n\n",
        result.data_source, result.duration_ms
    ));

    if !result.pages.is_empty() {
        out.push_str("## Pages Found\n\n");
        for page in result.pages.iter().take(5) {
            out.push_str(&format!(
                "- **{}** — {} followers\n  {}\n",
                page.name,
                page.followers
                    .map(|f| format!("{f}"))
                    .unwrap_or_else(|| "N/A".into()),
                page.about.chars().take(120).collect::<String>()
            ));
        }
        out.push('\n');
    }

    if !result.posts.is_empty() {
        out.push_str("## Posts Found\n\n");
        for post in result.posts.iter().take(5) {
            out.push_str(&format!(
                "### [{} — {}]({})\n{}\n\n",
                post.page_name,
                post.post_type,
                post.url,
                post.message.chars().take(200).collect::<String>()
            ));
        }
    }

    if result.posts.is_empty() && result.pages.is_empty() {
        out.push_str(
            "> No Facebook results found. Facebook limits public data access.\n> \
            > **Tip:** Configure `graph_api_token` in settings for richer data.\n",
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_post(likes: u64, comments: u64, shares: u64) -> FacebookPost {
        FacebookPost {
            id: "1".into(),
            url: "https://www.facebook.com/page/posts/1".into(),
            page_name: "TestPage".into(),
            page_url: "https://www.facebook.com/page".into(),
            message: "Amazing announcement from the team!".into(),
            likes,
            comments,
            shares,
            post_type: FacebookPostType::Text,
            published: String::new(),
            media_url: None,
        }
    }

    #[test]
    fn analyse_empty() {
        let a = analyse_posts(&[], &[]);
        assert_eq!(a.total_results, 0);
        assert_eq!(a.avg_engagement, 0.0);
    }

    #[test]
    fn analyse_basic_posts() {
        let posts = vec![make_post(1000, 50, 20), make_post(5000, 200, 100)];
        let a = analyse_posts(&posts, &[]);
        assert_eq!(a.total_results, 2);
        assert!(a.avg_engagement > 0.0);
    }

    #[test]
    fn viral_score_in_range() {
        let post = make_post(50_000, 5_000, 2_000);
        let score = viral_score_post(&post);
        assert!(score >= 0.0 && score <= 1.0, "score={score}");
    }

    #[test]
    fn viral_score_media_boost() {
        let mut base = make_post(1000, 100, 50);
        let without = viral_score_post(&base);
        base.post_type = FacebookPostType::Video;
        let with_video = viral_score_post(&base);
        assert!(
            with_video >= without,
            "video/media should boost viral score"
        );
    }

    #[test]
    fn viral_score_zero_engagement() {
        let post = make_post(0, 0, 0);
        let score = viral_score_post(&post);
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn analyse_total_includes_pages() {
        let posts = vec![make_post(100, 10, 5)];
        let pages = vec![crate::social::facebook::types::FacebookPage {
            id: "1".into(),
            name: "Test".into(),
            url: "https://fb.com/test".into(),
            followers: Some(1000),
            likes: Some(900),
            category: "Tech".into(),
            about: "A tech page".into(),
            verified: false,
        }];
        let a = analyse_posts(&posts, &pages);
        assert_eq!(a.total_results, 2, "total_results should include pages");
    }
}
