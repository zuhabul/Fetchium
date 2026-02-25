//! Reddit social intelligence types.

use serde::{Deserialize, Serialize};

/// A Reddit post (normalised).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditPost {
    pub id: String,
    pub url: String,
    pub permalink: String,
    pub title: String,
    pub selftext: String,
    pub author: String,
    pub subreddit: String,
    pub score: i64,
    pub upvote_ratio: f64,
    pub num_comments: u64,
    pub created_utc: f64,
    pub flair: Option<String>,
    pub is_self: bool,
    pub link_url: Option<String>,
    pub awards: u32,
    pub crossposts: u32,
}

/// A Reddit comment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditComment {
    pub id: String,
    pub author: String,
    pub body: String,
    pub score: i64,
    pub depth: u32,
    pub replies: Vec<RedditComment>,
}

/// Reddit subreddit statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubredditStats {
    pub name: String,
    pub subscribers: u64,
    pub active_users: Option<u64>,
    pub title: String,
    pub description: String,
    pub over18: bool,
}

/// Reddit post category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RedditCategory {
    Hot,
    New,
    Rising,
    Top,
    Controversial,
}

impl std::fmt::Display for RedditCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hot => write!(f, "hot"),
            Self::New => write!(f, "new"),
            Self::Rising => write!(f, "rising"),
            Self::Top => write!(f, "top"),
            Self::Controversial => write!(f, "controversial"),
        }
    }
}

/// Analysis of a Reddit search result set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditAnalysis {
    pub total_posts: usize,
    pub top_subreddits: Vec<(String, usize)>,
    pub avg_score: f64,
    pub avg_comments: f64,
    pub sentiment: crate::social::types::SentimentBreakdown,
    pub top_flairs: Vec<(String, usize)>,
    pub viral_posts: Vec<RedditPost>,
}

/// Pipeline config for Reddit intelligence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditPipelineConfig {
    pub query: String,
    pub subreddits: Vec<String>, // empty = all of Reddit
    pub max_posts: usize,
    pub category: RedditCategory,
    pub fetch_comments: bool,
    pub max_comments: usize,
    pub timeout_secs: u64,
}

impl Default for RedditPipelineConfig {
    fn default() -> Self {
        Self {
            query: String::new(),
            subreddits: Vec::new(),
            max_posts: 25,
            category: RedditCategory::Hot,
            fetch_comments: false,
            max_comments: 20,
            timeout_secs: 15,
        }
    }
}

/// Full result from a Reddit intelligence pipeline run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditPipelineResult {
    pub query: String,
    pub posts: Vec<RedditPost>,
    pub analysis: RedditAnalysis,
    pub subreddit_stats: Vec<SubredditStats>,
    pub duration_ms: u64,
}
