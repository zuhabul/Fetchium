//! Request/response types for the Fetchium REST API.

use serde::{Deserialize, Serialize};

// ─── Shared ───────────────────────────────────────────────────────

/// Common metadata included in every API response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResponseMeta {
    #[serde(default)]
    pub request_id: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_used: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_id: Option<String>,
}

/// Structured API error.
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
    pub error_type: String,
    pub status: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Queued,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobAcceptedResponse {
    pub meta: ResponseMeta,
    pub job_id: String,
    pub status: JobState,
    pub status_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatusResponse {
    pub meta: ResponseMeta,
    pub job_id: String,
    pub job_type: String,
    pub status: JobState,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ─── Search ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    /// Token budget: 100–10,000 (default 2,000)
    pub token_budget: Option<usize>,
    /// Detail tier: key_facts | summary | detailed | complete
    pub tier: Option<String>,
    /// Max sources to fetch: 1–20 (default 10)
    pub max_sources: Option<usize>,
    pub validate: Option<bool>,
}

impl SearchRequest {
    /// Validate and apply bounds to all fields.
    pub fn validated(self) -> Result<Self, &'static str> {
        if self.query.is_empty() {
            return Err("query cannot be empty");
        }
        if self.query.len() > 500 {
            return Err("query must be at most 500 characters");
        }
        if let Some(b) = self.token_budget {
            if !(100..=10_000).contains(&b) {
                return Err("token_budget must be between 100 and 10,000");
            }
        }
        if let Some(s) = self.max_sources {
            if !(1..=20).contains(&s) {
                return Err("max_sources must be between 1 and 20");
            }
        }
        if let Some(ref t) = self.tier {
            let valid = ["key_facts", "summary", "detailed", "complete"];
            if !valid.contains(&t.as_str()) {
                return Err("tier must be: key_facts | summary | detailed | complete");
            }
        }
        Ok(self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    #[serde(default)]
    pub meta: ResponseMeta,
    pub results: Vec<SearchResultItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub title: String,
    pub url: String,
    pub snippet: Option<String>,
    pub score: Option<f64>,
}

// ─── Fetch ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct FetchRequest {
    pub url: String,
    pub query: Option<String>,
    /// Token budget: 100–10,000 (default 2,000)
    pub token_budget: Option<usize>,
    /// Format: markdown | text | html (default markdown)
    pub format: Option<String>,
}

impl FetchRequest {
    pub fn validated(self) -> Result<Self, &'static str> {
        if self.url.is_empty() {
            return Err("url cannot be empty");
        }
        if self.url.len() > 2048 {
            return Err("url must be at most 2,048 characters");
        }
        if let Some(b) = self.token_budget {
            if !(100..=10_000).contains(&b) {
                return Err("token_budget must be between 100 and 10,000");
            }
        }
        if let Some(ref f) = self.format {
            let valid = ["markdown", "text", "html"];
            if !valid.contains(&f.as_str()) {
                return Err("format must be: markdown | text | html");
            }
        }
        Ok(self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchResponse {
    #[serde(default)]
    pub meta: ResponseMeta,
    pub url: String,
    pub title: Option<String>,
    pub content: String,
    pub tokens: usize,
    pub format: String,
    pub result_id: String,
}

// ─── Research ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct ResearchRequest {
    pub query: String,
    /// Token budget: 1,000–50,000 (default 10,000)
    pub token_budget: Option<usize>,
    /// Max sources: 1–20 (default 10)
    pub max_sources: Option<usize>,
    /// Depth: quick | standard | deep (default standard)
    pub depth: Option<String>,
    pub strict_evidence: Option<bool>,
    /// Citation style: apa | mla | chicago | ieee (default apa)
    pub citation_style: Option<String>,
}

impl ResearchRequest {
    pub fn validated(self) -> Result<Self, &'static str> {
        if self.query.is_empty() {
            return Err("query cannot be empty");
        }
        if self.query.len() > 500 {
            return Err("query must be at most 500 characters");
        }
        if let Some(b) = self.token_budget {
            if !(1_000..=50_000).contains(&b) {
                return Err("token_budget must be between 1,000 and 50,000");
            }
        }
        if let Some(s) = self.max_sources {
            if !(1..=20).contains(&s) {
                return Err("max_sources must be between 1 and 20");
            }
        }
        if let Some(ref d) = self.depth {
            let valid = ["quick", "standard", "deep"];
            if !valid.contains(&d.as_str()) {
                return Err("depth must be: quick | standard | deep");
            }
        }
        Ok(self)
    }
}

#[derive(Debug, Serialize)]
pub struct ResearchResponse {
    pub meta: ResponseMeta,
    pub report: String,
    pub reference_section: String,
    pub sources: Vec<SourceInfo>,
    pub confidence: f64,
}

#[derive(Debug, Serialize)]
pub struct SourceInfo {
    pub index: usize,
    pub title: String,
    pub url: String,
}

// ─── Estimate ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct EstimateRequest {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct EstimateResponse {
    pub meta: ResponseMeta,
    pub url: String,
    pub estimated_tokens: usize,
    pub estimated_relevant_tokens: usize,
    pub extraction_layer: u8,
    pub content_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataResponse<T> {
    pub meta: ResponseMeta,
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageResponse {
    pub meta: ResponseMeta,
    pub usage: serde_json::Value,
}

// ─── YouTube ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct YouTubeSearchRequest {
    pub query: String,
    pub max_results: Option<usize>,
    pub fact_check: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct YouTubeAnalyzeRequest {
    pub url: String,
    pub transcript: Option<bool>,
    pub comments: Option<bool>,
    pub teaching: Option<bool>,
}

// ─── Social ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct SocialResearchRequest {
    pub query: String,
    pub platforms: Option<Vec<String>>,
    pub max_per_platform: Option<usize>,
    pub generate_ideas: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedditSearchRequest {
    pub query: String,
    pub subreddits: Option<Vec<String>>,
    pub max_posts: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HackerNewsSearchRequest {
    pub query: String,
    pub max_results: Option<usize>,
}
