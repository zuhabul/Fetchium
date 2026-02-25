//! Facebook social intelligence types.

use serde::{Deserialize, Serialize};

/// A Facebook post (normalised from search result or graph API).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacebookPost {
    pub id: String,
    pub url: String,
    pub page_name: String,
    pub page_url: String,
    pub message: String,
    pub likes: u64,
    pub comments: u64,
    pub shares: u64,
    pub post_type: FacebookPostType,
    pub published: String,
    pub media_url: Option<String>,
}

/// Type of Facebook post.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FacebookPostType {
    Text,
    Photo,
    Video,
    Link,
    Story,
    Reel,
}

impl std::fmt::Display for FacebookPostType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text => write!(f, "text"),
            Self::Photo => write!(f, "photo"),
            Self::Video => write!(f, "video"),
            Self::Link => write!(f, "link"),
            Self::Story => write!(f, "story"),
            Self::Reel => write!(f, "reel"),
        }
    }
}

/// Facebook Page info (from Open Graph or Graph API).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacebookPage {
    pub id: String,
    pub name: String,
    pub url: String,
    pub followers: Option<u64>,
    pub likes: Option<u64>,
    pub category: String,
    pub about: String,
    pub verified: bool,
}

/// A trending topic on Facebook (from DDG search analysis).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacebookTrend {
    pub topic: String,
    pub result_count: u64,
    pub sample_urls: Vec<String>,
}

/// Analysis result for a Facebook search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacebookAnalysis {
    pub total_results: usize,
    pub pages: Vec<FacebookPage>,
    pub avg_engagement: f64,
    pub top_post_types: Vec<(String, usize)>,
    pub viral_posts: Vec<FacebookPost>,
}

/// Pipeline config for Facebook intelligence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacebookPipelineConfig {
    pub query: String,
    pub max_results: usize,
    pub graph_api_token: Option<String>, // optional — enables richer data
    pub timeout_secs: u64,
}

impl Default for FacebookPipelineConfig {
    fn default() -> Self {
        Self {
            query: String::new(),
            max_results: 20,
            graph_api_token: None,
            timeout_secs: 15,
        }
    }
}

/// Full result from a Facebook intelligence pipeline run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacebookPipelineResult {
    pub query: String,
    pub posts: Vec<FacebookPost>,
    pub pages: Vec<FacebookPage>,
    pub analysis: FacebookAnalysis,
    pub duration_ms: u64,
    pub data_source: FacebookDataSource,
}

/// Which data source was used (Facebook is restrictive).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FacebookDataSource {
    /// DDG `site:facebook.com` search results (most free, least rich)
    DdgSearch,
    /// OpenGraph metadata from public Facebook URLs
    OpenGraph,
    /// Meta Graph API (optional token required)
    GraphApi,
}
