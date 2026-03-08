//! Tavily Search API backend — AI-optimized search with pre-ranked results.
//!
//! Returns relevance-scored results with content extraction in one call.
//! API key required: set `TAVILY_API_KEY` env var or `search.tavily_api_key` in config.

use crate::error::{FetchiumError, FetchiumResult};
use crate::http::HttpClient;
use crate::search::{SearchBackend, SearchContext, TimeRange};
use crate::types::{BackendId, ResultItem};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Serialize)]
struct TavilyRequest<'a> {
    api_key: &'a str,
    query: &'a str,
    search_depth: &'a str,
    include_answer: bool,
    include_raw_content: bool,
    max_results: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    days: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct TavilyResponse {
    #[serde(default)]
    results: Vec<TavilyResult>,
    #[serde(default)]
    response_time: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct TavilyResult {
    url: String,
    title: String,
    #[serde(default)]
    content: String,
    #[serde(default)]
    score: Option<f64>,
}

/// Tavily search backend — returns pre-ranked, content-rich results.
pub struct TavilyBackend {
    http: HttpClient,
    api_key: String,
}

impl TavilyBackend {
    pub fn new(http: HttpClient, api_key: String) -> Self {
        Self { http, api_key }
    }
}

impl TavilyBackend {
    async fn search_inner(
        &self,
        query: &str,
        max_results: u32,
        days: Option<u32>,
    ) -> FetchiumResult<Vec<ResultItem>> {
        let request = TavilyRequest {
            api_key: &self.api_key,
            query,
            search_depth: "advanced",
            include_answer: false,
            include_raw_content: false,
            max_results: max_results.min(20),
            days,
        };

        let body = serde_json::to_string(&request)
            .map_err(|e| FetchiumError::Search(format!("Tavily serialization: {e}")))?;

        let response = self
            .http
            .post_json("https://api.tavily.com/search", &body)
            .await?;

        let parsed: TavilyResponse = serde_json::from_str(&response)
            .map_err(|e| FetchiumError::Search(format!("Tavily parse: {e}")))?;

        debug!(
            "Tavily: {} results in {:.2}s [days={:?}]",
            parsed.results.len(),
            parsed.response_time.unwrap_or(0.0),
            days,
        );

        let results = parsed
            .results
            .into_iter()
            .enumerate()
            .map(|(i, r)| ResultItem {
                title: r.title,
                url: r.url,
                snippet: r.content,
                rank: (i + 1) as u32,
                backend: BackendId::Tavily,
                score: r.score,
                published_date: None,
            })
            .collect();

        Ok(results)
    }

    fn time_range_to_days(tr: Option<TimeRange>) -> Option<u32> {
        match tr {
            Some(TimeRange::Day) => Some(1),
            Some(TimeRange::Week) => Some(7),
            Some(TimeRange::Month) => Some(30),
            Some(TimeRange::Year) => Some(365),
            None => None,
        }
    }
}

#[async_trait]
impl SearchBackend for TavilyBackend {
    fn id(&self) -> BackendId {
        BackendId::Tavily
    }

    async fn search(&self, query: &str, max_results: u32) -> FetchiumResult<Vec<ResultItem>> {
        self.search_inner(query, max_results, None).await
    }

    async fn search_with_context(
        &self,
        query: &str,
        max_results: u32,
        ctx: &SearchContext,
    ) -> FetchiumResult<Vec<ResultItem>> {
        let days = Self::time_range_to_days(ctx.time_range);
        self.search_inner(query, max_results, days).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tavily_response() {
        let json = r#"{
            "results": [
                {"url": "https://example.com", "title": "Test", "content": "Content here", "score": 0.95}
            ],
            "response_time": 1.5
        }"#;
        let parsed: TavilyResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.results.len(), 1);
        assert_eq!(parsed.results[0].score, Some(0.95));
    }

    #[test]
    fn parse_tavily_empty_response() {
        let json = r#"{"results": [], "response_time": 0.1}"#;
        let parsed: TavilyResponse = serde_json::from_str(json).unwrap();
        assert!(parsed.results.is_empty());
    }
}
