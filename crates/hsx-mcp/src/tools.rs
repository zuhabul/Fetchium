//! MCP tool schema definitions (PRD §30).
//!
//! Defines the 5 composite HyperSearchX tools and their JSON Schema input types.

use serde::Deserialize;
use serde_json::{json, Value};

/// Return the JSON Schema definitions for all 5 composite MCP tools.
pub fn tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "hypersearch_search",
            "description": "Search the web and return token-efficient results. Handles the full pipeline: multi-backend search, ranking, deduplication, validation, and token budgeting in a single call.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "The search query" },
                    "token_budget": { "type": "integer", "description": "Maximum tokens in response (default: 2000)" },
                    "tier": { "type": "string", "enum": ["key_facts", "summary", "detailed", "complete"], "description": "Detail level (default: summary)" },
                    "max_sources": { "type": "integer", "description": "Maximum number of sources to include (default: 10)" }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": "hypersearch_fetch",
            "description": "Fetch a URL with query-aware extraction. Extracts only content relevant to the query, within the token budget. Far more efficient than raw scraping.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "url": { "type": "string", "description": "The URL to fetch" },
                    "query": { "type": "string", "description": "Extract only content relevant to this query (optional but recommended)" },
                    "token_budget": { "type": "integer", "description": "Maximum tokens in response (default: 3000)" },
                    "format": { "type": "string", "enum": ["markdown", "segments", "json"], "description": "Output format (default: markdown)" }
                },
                "required": ["url"]
            }
        }),
        json!({
            "name": "hypersearch_research",
            "description": "Conduct multi-source research with citations and evidence tracking. Searches, extracts, ranks, validates, and synthesizes findings with full citation chains.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "The research query" },
                    "token_budget": { "type": "integer", "description": "Maximum tokens in response (default: 4000)" },
                    "depth": { "type": "string", "enum": ["shallow", "standard", "deep"], "description": "Research depth (default: standard)" },
                    "max_sources": { "type": "integer", "description": "Maximum number of sources to analyze (default: 10)" },
                    "strict_evidence": { "type": "boolean", "description": "Require citation for every claim (default: false)" },
                    "citation_style": { "type": "string", "description": "Citation style: inline, apa, ieee (default: inline)" }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": "hypersearch_estimate",
            "description": "Estimate the token cost of fetching a URL without actually fetching it. Use this before committing tokens.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "url": { "type": "string", "description": "The URL to estimate" }
                },
                "required": ["url"]
            }
        }),
        json!({
            "name": "hypersearch_expand",
            "description": "Get more detail on a previous result using its result_id and Progressive Detail Streaming (PDS). Expands from key_facts → summary → detailed → complete without re-fetching.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "result_id": { "type": "string", "description": "The result_id from a previous search or fetch call" },
                    "tier": { "type": "string", "enum": ["key_facts", "summary", "detailed", "complete"], "description": "The detail tier to expand to" }
                },
                "required": ["result_id", "tier"]
            }
        }),
        json!({
            "name": "youtube_search",
            "description": "Search YouTube videos with VideoFusion ranking. Returns ranked videos with relevance, freshness, authority, engagement, and educational scores.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "The search query" },
                    "max_results": { "type": "integer", "description": "Maximum videos to return (default: 5)" },
                    "fact_check": { "type": "boolean", "description": "Enable cross-video fact checking (default: false)" }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": "youtube_analyze",
            "description": "Analyze a single YouTube video: metadata, transcript, comments, credibility, clickbait detection, and educational scoring.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "url": { "type": "string", "description": "The YouTube video URL" },
                    "transcript": { "type": "boolean", "description": "Fetch transcript (default: true)" },
                    "comments": { "type": "boolean", "description": "Fetch comments (default: true)" },
                    "teaching": { "type": "boolean", "description": "Generate teaching content (default: false)" }
                },
                "required": ["url"]
            }
        }),
        json!({
            "name": "social_research",
            "description": "Unified cross-platform social media research: Twitter/X, Reddit, TikTok, HackerNews, YouTube simultaneously. Returns trends, viral content, and content ideas.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "The research query or topic" },
                    "platforms": { "type": "array", "items": { "type": "string", "enum": ["twitter", "reddit", "tiktok", "hackernews", "youtube"] }, "description": "Platforms to include (default: all)" },
                    "max_per_platform": { "type": "integer", "description": "Max posts per platform (default: 20)" },
                    "generate_ideas": { "type": "boolean", "description": "Generate content ideas (default: true)" }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": "reddit_search",
            "description": "Search Reddit posts with sentiment analysis, subreddit clustering, and viral detection. Uses the free public JSON API.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query" },
                    "subreddits": { "type": "array", "items": { "type": "string" }, "description": "Specific subreddits to search (optional)" },
                    "max_posts": { "type": "integer", "description": "Maximum posts (default: 25)" }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": "hackernews_search",
            "description": "Search Hacker News stories via Algolia + Firebase APIs. Free, no rate limits, returns ranked stories with engagement metrics.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query" },
                    "max_results": { "type": "integer", "description": "Maximum stories (default: 20)" }
                },
                "required": ["query"]
            }
        }),
    ]
}

// ─── Input structs ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SearchInput {
    pub query: String,
    pub token_budget: Option<usize>,
    pub tier: Option<String>,
    pub max_sources: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct FetchInput {
    pub url: String,
    pub query: Option<String>,
    pub token_budget: Option<usize>,
    pub format: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResearchInput {
    pub query: String,
    pub token_budget: Option<usize>,
    pub max_sources: Option<usize>,
    pub depth: Option<String>,
    pub strict_evidence: Option<bool>,
    pub citation_style: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EstimateInput {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct ExpandInput {
    pub result_id: String,
    pub tier: String,
}

#[derive(Debug, Deserialize)]
pub struct YouTubeSearchInput {
    pub query: String,
    pub max_results: Option<usize>,
    pub fact_check: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct YouTubeAnalyzeInput {
    pub url: String,
    pub transcript: Option<bool>,
    pub comments: Option<bool>,
    pub teaching: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SocialResearchInput {
    pub query: String,
    pub platforms: Option<Vec<String>>,
    pub max_per_platform: Option<usize>,
    pub generate_ideas: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RedditSearchInput {
    pub query: String,
    pub subreddits: Option<Vec<String>>,
    pub max_posts: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct HackerNewsSearchInput {
    pub query: String,
    pub max_results: Option<usize>,
}
