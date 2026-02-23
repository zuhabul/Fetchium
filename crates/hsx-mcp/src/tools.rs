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
