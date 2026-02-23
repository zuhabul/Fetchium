//! Request/response types for the HyperSearchX REST API.

use serde::{Deserialize, Serialize};

// ─── Shared ───────────────────────────────────────────────────────

/// Common metadata included in every API response.
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMeta {
    pub query: String,
    pub tier: String,
    pub tokens_used: usize,
    pub sources_count: usize,
    pub duration_ms: u64,
    pub result_id: String,
}

/// Structured API error.
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
    pub error_type: String,
    pub status: u16,
}

// ─── Search ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub token_budget: Option<usize>,
    pub tier: Option<String>,
    pub max_sources: Option<usize>,
    pub validate: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
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

#[derive(Debug, Deserialize)]
pub struct FetchRequest {
    pub url: String,
    pub query: Option<String>,
    pub token_budget: Option<usize>,
    pub format: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchResponse {
    pub url: String,
    pub title: Option<String>,
    pub content: String,
    pub tokens: usize,
    pub format: String,
    pub result_id: String,
}

// ─── Research ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ResearchRequest {
    pub query: String,
    pub token_budget: Option<usize>,
    pub max_sources: Option<usize>,
    pub depth: Option<String>,
    pub strict_evidence: Option<bool>,
    pub citation_style: Option<String>,
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

#[derive(Debug, Deserialize)]
pub struct EstimateRequest {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct EstimateResponse {
    pub url: String,
    pub estimated_tokens: usize,
    pub estimated_relevant_tokens: usize,
    pub extraction_layer: u8,
    pub content_type: String,
}
