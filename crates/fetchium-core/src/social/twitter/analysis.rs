//! Twitter/X tweet analysis: sentiment, engagement, viral scoring, thread reconstruction.

use crate::social::twitter::types::*;
use crate::social::types::{score_sentiment, SentimentBreakdown, ViralScore};
use std::collections::HashMap;

/// Analyse a set of tweets and produce a full TweetAnalysis.
pub fn analyse_tweets(tweets: &[Tweet]) -> TweetAnalysis {
    let total_tweets = tweets.len();
    if total_tweets == 0 {
        return TweetAnalysis {
            total_tweets: 0,
            avg_engagement: 0.0,
            sentiment: SentimentBreakdown::default(),
            top_hashtags: Vec::new(),
            top_authors: Vec::new(),
            viral_tweets: Vec::new(),
            threads: Vec::new(),
        };
    }

    // Engagement
    let total_eng: u64 = tweets
        .iter()
        .map(|t| t.likes + t.retweets * 2 + t.replies)
        .sum();
    let avg_engagement = total_eng as f64 / total_tweets as f64;

    // Sentiment
    let scores: Vec<f64> = tweets.iter().map(|t| score_sentiment(&t.text)).collect();
    let sentiment = compute_sentiment_breakdown(&scores);

    // Top hashtags
    let mut tag_counts: HashMap<String, usize> = HashMap::new();
    for tweet in tweets {
        for tag in &tweet.hashtags {
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }
    let mut top_hashtags: Vec<(String, usize)> = tag_counts.into_iter().collect();
    top_hashtags.sort_by(|a, b| b.1.cmp(&a.1));
    top_hashtags.truncate(10);

    // Top authors by engagement
    let mut author_eng: HashMap<String, u64> = HashMap::new();
    let mut author_map: HashMap<String, &TwitterUser> = HashMap::new();
    for tweet in tweets {
        let eng = tweet.likes + tweet.retweets * 2 + tweet.replies;
        *author_eng.entry(tweet.author.username.clone()).or_insert(0) += eng;
        author_map.insert(tweet.author.username.clone(), &tweet.author);
    }
    let mut author_sorted: Vec<(String, u64)> = author_eng.into_iter().collect();
    author_sorted.sort_by(|a, b| b.1.cmp(&a.1));
    let top_authors: Vec<TwitterUser> = author_sorted
        .iter()
        .take(5)
        .filter_map(|(u, _)| author_map.get(u).map(|a| (*a).clone()))
        .collect();

    // Viral tweets (top 10% by engagement)
    let mut sorted = tweets.to_vec();
    sorted.sort_by(|a, b| {
        let ea = a.likes + a.retweets * 2 + a.replies;
        let eb = b.likes + b.retweets * 2 + b.replies;
        eb.cmp(&ea)
    });
    let viral_count = (total_tweets / 10).max(3);
    let viral_tweets = sorted.into_iter().take(viral_count).collect();

    TweetAnalysis {
        total_tweets,
        avg_engagement,
        sentiment,
        top_hashtags,
        top_authors,
        viral_tweets,
        threads: Vec::new(), // Thread reconstruction is optional
    }
}

/// Score viral potential of a tweet.
pub fn viral_score_tweet(tweet: &Tweet) -> ViralScore {
    let engagement = (tweet.likes + tweet.retweets * 2 + tweet.replies) as f64;
    let engagement_velocity = (engagement.ln() / 15.0).clamp(0.0, 1.0);

    let sentiment = score_sentiment(&tweet.text).abs();
    let emotional_resonance = sentiment;

    // More hashtags + media → more shareable
    let has_media = !tweet.media_urls.is_empty();
    let shareability = ((tweet.hashtags.len() as f64 * 0.1)
        + if has_media { 0.3 } else { 0.0 }
        + (tweet.retweets as f64 / (tweet.retweets + 1) as f64) * 0.5)
        .min(1.0);

    // Novelty: short posts often go viral
    let novelty = if tweet.text.len() < 100 { 0.8 } else { 0.4 };

    let trend_alignment = if tweet.hashtags.is_empty() { 0.2 } else { 0.7 };

    ViralScore::compute(
        engagement_velocity,
        emotional_resonance,
        shareability,
        novelty,
        trend_alignment,
    )
}

/// Reconstruct tweet threads by grouping replies.
pub fn reconstruct_threads(tweets: &[Tweet]) -> Vec<TwitterThread> {
    // Group by root: tweets that are not replies are roots
    let roots: Vec<&Tweet> = tweets.iter().filter(|t| !t.is_reply).collect();
    let replies: Vec<&Tweet> = tweets.iter().filter(|t| t.is_reply).collect();

    roots
        .iter()
        .filter_map(|root| {
            // Collect replies from the same author as the root (simple heuristic)
            let mut thread_tweets = vec![(*root).clone()];
            for reply in &replies {
                if reply.author.username == root.author.username {
                    thread_tweets.push((*reply).clone());
                }
            }
            if thread_tweets.len() < 2 {
                return None;
            }
            let total_eng: u64 = thread_tweets
                .iter()
                .map(|t| t.likes + t.retweets + t.replies)
                .sum();
            Some(TwitterThread {
                root_tweet_id: root.id.clone(),
                author: root.author.clone(),
                total_engagement: total_eng,
                tweets: thread_tweets,
            })
        })
        .collect()
}

// ─── Helpers ─────────────────────────────────────────────────────

fn compute_sentiment_breakdown(scores: &[f64]) -> SentimentBreakdown {
    if scores.is_empty() {
        return SentimentBreakdown::default();
    }
    let n = scores.len() as f64;
    let positive_pct = scores.iter().filter(|&&s| s > 0.1).count() as f64 / n;
    let negative_pct = scores.iter().filter(|&&s| s < -0.1).count() as f64 / n;
    let neutral_pct = 1.0 - positive_pct - negative_pct;
    let compound = scores.iter().sum::<f64>() / n;
    SentimentBreakdown {
        positive_pct,
        negative_pct,
        neutral_pct,
        compound,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tweet(text: &str, likes: u64, retweets: u64) -> Tweet {
        Tweet {
            id: "1".into(),
            url: "https://x.com/test/status/1".into(),
            author: TwitterUser {
                username: "testuser".into(),
                display_name: "Test".into(),
                followers: None,
                following: None,
                verified: false,
                bio: String::new(),
            },
            text: text.into(),
            published: "2025-01-01".into(),
            likes,
            retweets,
            replies: 0,
            views: None,
            hashtags: vec!["#test".into()],
            mentions: Vec::new(),
            media_urls: Vec::new(),
            is_reply: false,
            is_retweet: false,
            quoted_tweet: None,
        }
    }

    #[test]
    fn analyse_empty() {
        let a = analyse_tweets(&[]);
        assert_eq!(a.total_tweets, 0);
        assert_eq!(a.avg_engagement, 0.0);
    }

    #[test]
    fn analyse_single_tweet() {
        let tweet = make_tweet("Amazing product!", 1000, 200);
        let a = analyse_tweets(&[tweet]);
        assert_eq!(a.total_tweets, 1);
        assert!(a.avg_engagement > 0.0);
    }

    #[test]
    fn viral_score_range() {
        let t = make_tweet("This is viral!", 50000, 10000);
        let vs = viral_score_tweet(&t);
        assert!(vs.overall >= 0.0 && vs.overall <= 1.0);
    }

    #[test]
    fn viral_score_zero_engagement() {
        let t = make_tweet("nothing", 0, 0);
        let vs = viral_score_tweet(&t);
        assert!(vs.overall >= 0.0 && vs.overall <= 1.0);
    }

    #[test]
    fn viral_score_short_tweet_higher_novelty() {
        let short = make_tweet("wow", 1000, 100);
        // Must be > 100 chars to get the lower novelty score
        let long = make_tweet(
            "This is a very long tweet that definitely exceeds one hundred characters in total length to test novelty scoring behavior",
            1000,
            100,
        );
        let short_vs = viral_score_tweet(&short);
        let long_vs = viral_score_tweet(&long);
        assert!(
            short_vs.novelty > long_vs.novelty,
            "short tweets have higher novelty"
        );
    }

    #[test]
    fn analyse_top_hashtags() {
        let mut t1 = make_tweet("hey", 100, 10);
        t1.hashtags = vec!["#rust".into(), "#code".into()];
        let mut t2 = make_tweet("there", 200, 20);
        t2.hashtags = vec!["#rust".into()];
        let a = analyse_tweets(&[t1, t2]);
        assert!(!a.top_hashtags.is_empty());
        assert_eq!(a.top_hashtags[0].0, "#rust");
        assert_eq!(a.top_hashtags[0].1, 2);
    }

    #[test]
    fn analyse_top_authors_by_engagement() {
        let mut t1 = make_tweet("low engagement", 10, 1);
        t1.author.username = "low_user".into();
        let mut t2 = make_tweet("high engagement", 100_000, 20_000);
        t2.author.username = "high_user".into();
        let a = analyse_tweets(&[t1, t2]);
        assert!(!a.top_authors.is_empty());
        assert_eq!(a.top_authors[0].username, "high_user");
    }

    #[test]
    fn reconstruct_threads_basic() {
        let root = make_tweet("Root tweet in a thread", 500, 50);
        let mut reply = make_tweet("Reply to root", 100, 10);
        reply.is_reply = true;
        reply.author.username = "testuser".into(); // same author as root
        let threads = reconstruct_threads(&[root, reply]);
        assert_eq!(threads.len(), 1);
        assert_eq!(threads[0].tweets.len(), 2);
    }

    #[test]
    fn reconstruct_threads_no_replies() {
        let t1 = make_tweet("Standalone tweet 1", 100, 10);
        let t2 = make_tweet("Standalone tweet 2", 200, 20);
        let threads = reconstruct_threads(&[t1, t2]);
        // No replies → no multi-tweet threads
        assert!(threads.is_empty());
    }

    #[test]
    fn analyse_viral_tweets_minimum_three() {
        let tweets: Vec<_> = (0..5)
            .map(|i| make_tweet(&format!("tweet {i}"), i * 100, i * 10))
            .collect();
        let a = analyse_tweets(&tweets);
        // viral_count = max(5/10, 3) = 3
        assert_eq!(a.viral_tweets.len(), 3);
    }
}
