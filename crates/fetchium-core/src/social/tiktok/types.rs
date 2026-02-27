//! TikTok social intelligence types.

use serde::{Deserialize, Serialize};

/// A TikTok video (normalised).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TikTokVideo {
    pub id: String,
    pub url: String,
    pub author: TikTokUser,
    pub description: String,
    pub published: String,
    pub duration_secs: u32,
    pub likes: u64,
    pub comments: u64,
    pub shares: u64,
    pub plays: u64,
    pub hashtags: Vec<String>,
    pub music: Option<TikTokMusic>,
    pub is_duet: bool,
    pub is_stitch: bool,
}

/// TikTok user profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TikTokUser {
    pub username: String,
    pub display_name: String,
    pub followers: Option<u64>,
    pub following: Option<u64>,
    pub verified: bool,
    pub bio: String,
}

/// Music used in a TikTok video.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TikTokMusic {
    pub title: String,
    pub artist: String,
    pub is_original: bool,
}

/// A trending TikTok hashtag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TikTokTrend {
    pub hashtag: String,
    pub view_count: u64,
    pub video_count: Option<u64>,
    pub is_challenge: bool,
}

/// Analysis of a set of TikTok videos.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TikTokAnalysis {
    pub total_videos: usize,
    pub avg_plays: f64,
    pub avg_engagement_rate: f64,
    pub top_hashtags: Vec<(String, usize)>,
    pub top_creators: Vec<TikTokUser>,
    pub viral_videos: Vec<TikTokVideo>,
    pub trending_music: Vec<(String, usize)>,
    pub sentiment: crate::social::types::SentimentBreakdown,
}

/// Pipeline config for TikTok intelligence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TikTokPipelineConfig {
    pub query: String,
    pub max_videos: usize,
    pub fetch_trends: bool,
    pub timeout_secs: u64,
}

impl Default for TikTokPipelineConfig {
    fn default() -> Self {
        Self {
            query: String::new(),
            max_videos: 20,
            fetch_trends: true,
            timeout_secs: 15,
        }
    }
}

/// Full result from a TikTok intelligence pipeline run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TikTokPipelineResult {
    pub query: String,
    pub videos: Vec<TikTokVideo>,
    pub trends: Vec<TikTokTrend>,
    pub analysis: TikTokAnalysis,
    pub duration_ms: u64,
}
