//! Reddit post analysis: scoring, sentiment, subreddit clustering, viral detection.

use crate::social::reddit::types::*;
use crate::social::types::{score_sentiment, SentimentBreakdown};
use std::collections::HashMap;

/// Analyse a set of Reddit posts and return RedditAnalysis.
pub fn analyse_posts(posts: &[RedditPost]) -> RedditAnalysis {
    let total_posts = posts.len();
    if total_posts == 0 {
        return RedditAnalysis {
            total_posts: 0,
            top_subreddits: Vec::new(),
            avg_score: 0.0,
            avg_comments: 0.0,
            sentiment: SentimentBreakdown::default(),
            top_flairs: Vec::new(),
            viral_posts: Vec::new(),
        };
    }

    // Subreddit distribution
    let mut sub_counts: HashMap<String, usize> = HashMap::new();
    for post in posts {
        *sub_counts.entry(post.subreddit.clone()).or_insert(0) += 1;
    }
    let mut top_subreddits: Vec<(String, usize)> = sub_counts.into_iter().collect();
    top_subreddits.sort_by(|a, b| b.1.cmp(&a.1));
    top_subreddits.truncate(10);

    // Averages
    let avg_score = posts.iter().map(|p| p.score as f64).sum::<f64>() / total_posts as f64;
    let avg_comments =
        posts.iter().map(|p| p.num_comments as f64).sum::<f64>() / total_posts as f64;

    // Sentiment (title + selftext combined)
    let scores: Vec<f64> = posts
        .iter()
        .map(|p| score_sentiment(&format!("{} {}", p.title, p.selftext)))
        .collect();
    let sentiment = compute_sentiment(&scores);

    // Top flairs
    let mut flair_counts: HashMap<String, usize> = HashMap::new();
    for post in posts {
        if let Some(ref f) = post.flair {
            *flair_counts.entry(f.clone()).or_insert(0) += 1;
        }
    }
    let mut top_flairs: Vec<(String, usize)> = flair_counts.into_iter().collect();
    top_flairs.sort_by(|a, b| b.1.cmp(&a.1));
    top_flairs.truncate(5);

    // Viral posts: top 10% by score
    let mut sorted = posts.to_vec();
    sorted.sort_by(|a, b| b.score.cmp(&a.score));
    let viral_count = (total_posts / 10).max(3).min(total_posts);
    let viral_posts = sorted.into_iter().take(viral_count).collect();

    RedditAnalysis {
        total_posts,
        top_subreddits,
        avg_score,
        avg_comments,
        sentiment,
        top_flairs,
        viral_posts,
    }
}

/// Score viral potential of a Reddit post.
pub fn viral_score_post(post: &RedditPost) -> f64 {
    let score_norm = (post.score as f64).ln().max(0.0) / 15.0;
    let comment_norm = (post.num_comments as f64).ln().max(0.0) / 12.0;
    let ratio_score = post.upvote_ratio;
    let award_bonus = (post.awards as f64 * 0.05).min(0.2);
    let crosspost_bonus = (post.crossposts as f64 * 0.02).min(0.1);

    (score_norm * 0.4 + comment_norm * 0.3 + ratio_score * 0.2 + award_bonus + crosspost_bonus)
        .min(1.0)
}

/// Cluster posts by subreddit and return per-subreddit summaries.
pub fn cluster_by_subreddit(posts: &[RedditPost]) -> HashMap<String, Vec<&RedditPost>> {
    let mut clusters: HashMap<String, Vec<&RedditPost>> = HashMap::new();
    for post in posts {
        clusters
            .entry(post.subreddit.clone())
            .or_default()
            .push(post);
    }
    clusters
}

/// Find the most discussed posts (by comment count).
pub fn top_by_comments(posts: &[RedditPost], n: usize) -> Vec<&RedditPost> {
    let mut sorted: Vec<&RedditPost> = posts.iter().collect();
    sorted.sort_by(|a, b| b.num_comments.cmp(&a.num_comments));
    sorted.truncate(n);
    sorted
}

fn compute_sentiment(scores: &[f64]) -> SentimentBreakdown {
    if scores.is_empty() {
        return SentimentBreakdown::default();
    }
    let n = scores.len() as f64;
    let pos = scores.iter().filter(|&&s| s > 0.1).count() as f64 / n;
    let neg = scores.iter().filter(|&&s| s < -0.1).count() as f64 / n;
    let compound = scores.iter().sum::<f64>() / n;
    SentimentBreakdown {
        positive_pct: pos,
        negative_pct: neg,
        neutral_pct: 1.0 - pos - neg,
        compound,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_post(score: i64, comments: u64, sub: &str) -> RedditPost {
        RedditPost {
            id: "x1".into(),
            url: "https://reddit.com/r/rust/x1".into(),
            permalink: "/r/rust/comments/x1/".into(),
            title: "Test post amazing wonderful".into(),
            selftext: String::new(),
            author: "user".into(),
            subreddit: sub.into(),
            score,
            upvote_ratio: 0.95,
            num_comments: comments,
            created_utc: 1700000000.0,
            flair: Some("Question".into()),
            is_self: true,
            link_url: None,
            awards: 2,
            crossposts: 1,
        }
    }

    #[test]
    fn analyse_empty() {
        let a = analyse_posts(&[]);
        assert_eq!(a.total_posts, 0);
    }

    #[test]
    fn analyse_basic() {
        let posts = vec![
            make_post(1000, 50, "rust"),
            make_post(2500, 120, "programming"),
            make_post(500, 30, "rust"),
        ];
        let a = analyse_posts(&posts);
        assert_eq!(a.total_posts, 3);
        assert!(a.avg_score > 0.0);
        assert_eq!(a.top_subreddits[0].0, "rust");
        assert_eq!(a.top_subreddits[0].1, 2);
    }

    #[test]
    fn viral_score_range() {
        let p = make_post(10000, 500, "rust");
        let vs = viral_score_post(&p);
        assert!(vs >= 0.0 && vs <= 1.0);
    }

    #[test]
    fn viral_score_zero_post() {
        let p = make_post(0, 0, "rust");
        let vs = viral_score_post(&p);
        assert!(
            vs >= 0.0 && vs <= 1.0,
            "zero-score post viral in range: {vs}"
        );
    }

    #[test]
    fn viral_score_awards_boost() {
        let mut low = make_post(1000, 50, "test");
        low.awards = 0;
        low.crossposts = 0;
        let mut high = make_post(1000, 50, "test");
        high.awards = 20;
        high.crossposts = 10;
        assert!(
            viral_score_post(&high) >= viral_score_post(&low),
            "awards/crossposts should increase viral score"
        );
    }

    #[test]
    fn cluster_by_subreddit_groups_correctly() {
        let posts = vec![
            make_post(100, 10, "rust"),
            make_post(200, 20, "programming"),
            make_post(150, 15, "rust"),
        ];
        let clusters = cluster_by_subreddit(&posts);
        assert_eq!(clusters["rust"].len(), 2);
        assert_eq!(clusters["programming"].len(), 1);
    }

    #[test]
    fn top_by_comments_ordered() {
        let posts = vec![
            make_post(100, 10, "r1"),
            make_post(200, 500, "r2"),
            make_post(150, 50, "r3"),
            make_post(300, 200, "r4"),
        ];
        let top = top_by_comments(&posts, 2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].num_comments, 500);
        assert_eq!(top[1].num_comments, 200);
    }

    #[test]
    fn top_by_comments_fewer_than_n() {
        let posts = vec![make_post(100, 10, "r1")];
        let top = top_by_comments(&posts, 5);
        assert_eq!(top.len(), 1);
    }

    #[test]
    fn analyse_flair_counting() {
        let mut p1 = make_post(100, 10, "rust");
        p1.flair = Some("Tutorial".into());
        let mut p2 = make_post(200, 20, "rust");
        p2.flair = Some("Tutorial".into());
        let mut p3 = make_post(150, 15, "rust");
        p3.flair = Some("Question".into());
        let a = analyse_posts(&[p1, p2, p3]);
        assert!(!a.top_flairs.is_empty());
        assert_eq!(a.top_flairs[0].0, "Tutorial");
        assert_eq!(a.top_flairs[0].1, 2);
    }

    #[test]
    fn analyse_viral_posts_clamped_to_total() {
        // 2 posts: max(2/10, 3).min(2) = 2
        let posts = vec![make_post(1000, 50, "rust"), make_post(2500, 120, "rust")];
        let a = analyse_posts(&posts);
        assert_eq!(a.viral_posts.len(), 2);
        assert!(a.viral_posts[0].score >= a.viral_posts[1].score);
    }

    #[test]
    fn analyse_sentiment_in_range() {
        let posts = vec![make_post(100, 5, "rust"), make_post(200, 10, "programming")];
        let a = analyse_posts(&posts);
        assert!(
            a.sentiment.compound >= -1.0 && a.sentiment.compound <= 1.0,
            "compound={}",
            a.sentiment.compound
        );
    }
}
