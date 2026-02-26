//! Twitter/X intelligence types.

use serde::{Deserialize, Serialize};

/// A Twitter/X tweet (normalised).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tweet {
    pub id: String,
    pub url: String,
    pub author: TwitterUser,
    pub text: String,
    pub published: String,
    pub likes: u64,
    pub retweets: u64,
    pub replies: u64,
    pub views: Option<u64>,
    pub hashtags: Vec<String>,
    pub mentions: Vec<String>,
    pub media_urls: Vec<String>,
    pub is_reply: bool,
    pub is_retweet: bool,
    pub quoted_tweet: Option<Box<Tweet>>,
}

/// Twitter/X user profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitterUser {
    pub username: String,
    pub display_name: String,
    pub followers: Option<u64>,
    pub following: Option<u64>,
    pub verified: bool,
    pub bio: String,
}

/// A trending topic on Twitter/X.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitterTrend {
    pub topic: String,
    pub tweet_volume: Option<u64>,
    pub region: String,
    pub url: String,
}

/// Thread reconstruction: ordered list of tweets in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitterThread {
    pub root_tweet_id: String,
    pub tweets: Vec<Tweet>,
    pub total_engagement: u64,
    pub author: TwitterUser,
}

/// Twitter search result envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitterSearchResult {
    pub query: String,
    pub tweets: Vec<Tweet>,
    pub total_fetched: usize,
    pub source: TwitterSource,
}

/// Source used to fetch Twitter data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TwitterSource {
    Nitter,   // nitter.net mirror scraping
    SyndFeed, // RSS/Atom syndication
    Fallback, // internal fallback
}

/// Analysis of a set of tweets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweetAnalysis {
    pub total_tweets: usize,
    pub avg_engagement: f64,
    pub sentiment: crate::social::types::SentimentBreakdown,
    pub top_hashtags: Vec<(String, usize)>,
    pub top_authors: Vec<TwitterUser>,
    pub viral_tweets: Vec<Tweet>,
    pub threads: Vec<TwitterThread>,
}

/// Pipeline config for Twitter intelligence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitterPipelineConfig {
    pub query: String,
    pub max_tweets: usize,
    pub fetch_trends: bool,
    pub reconstruct_threads: bool,
    pub analyze_sentiment: bool,
    pub nitter_instances: Vec<String>,
    pub timeout_secs: u64,
    pub searxng_url: Option<String>, // local SearXNG for reliable site:x.com search
}

impl Default for TwitterPipelineConfig {
    fn default() -> Self {
        Self {
            query: String::new(),
            max_tweets: 50,
            fetch_trends: true,
            reconstruct_threads: false,
            analyze_sentiment: true,
            nitter_instances: vec![
                // xcancel.com runs unixfox/nitter-fork — most reliable (Feb 2026)
                // DDG-primary approach means these are only fallback Tier 2/3
                "https://xcancel.com".into(),
                "https://nitter.privacyredirect.com".into(),
                "https://lightbrd.com".into(),
                // nitter.poast.org and nitter.privacydev.net removed (503/timeout Feb 2026)
            ],
            timeout_secs: 10, // shorter timeout — DDG is Tier 1, nitter is fallback
            searxng_url: None,
        }
    }
}

/// Full result from a Twitter intelligence pipeline run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitterPipelineResult {
    pub query: String,
    pub tweets: Vec<Tweet>,
    pub trends: Vec<TwitterTrend>,
    pub analysis: TweetAnalysis,
    pub duration_ms: u64,
}
