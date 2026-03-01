//! Firecrawl search+scrape backend — search the web and get full markdown content.
//!
//! Combines search with content scraping in one API call.
//! Returns clean markdown content ready for LLM consumption.
//! API key required: set `FIRECRAWL_API_KEY` env var or `search.firecrawl_api_key` in config.

use crate::error::{HsxError, HsxResult};
use crate::http::HttpClient;
use crate::search::SearchBackend;
use crate::types::{BackendId, ResultItem};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Serialize)]
struct FirecrawlSearchRequest<'a> {
    query: &'a str,
    limit: u32,
}

#[derive(Debug, Deserialize)]
struct FirecrawlSearchResponse {
    #[serde(default)]
    success: bool,
    #[serde(default)]
    data: Vec<FirecrawlResult>,
}

#[derive(Debug, Deserialize)]
struct FirecrawlResult {
    #[serde(default)]
    url: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    markdown: Option<String>,
}

/// Firecrawl search+scrape backend — returns full markdown content.
pub struct FirecrawlBackend {
    http: HttpClient,
    api_key: String,
}

impl FirecrawlBackend {
    pub fn new(http: HttpClient, api_key: String) -> Self {
        Self { http, api_key }
    }
}

#[async_trait]
impl SearchBackend for FirecrawlBackend {
    fn id(&self) -> BackendId {
        BackendId::Firecrawl
    }

    async fn search(&self, query: &str, max_results: u32) -> HsxResult<Vec<ResultItem>> {
        let request = FirecrawlSearchRequest {
            query,
            limit: max_results.min(5),
        };

        let body = serde_json::to_string(&request)
            .map_err(|e| HsxError::Search(format!("Firecrawl serialization: {e}")))?;

        let auth_value = format!("Bearer {}", self.api_key);
        let response = self
            .http
            .post_json_with_header(
                "https://api.firecrawl.dev/v1/search",
                &body,
                "Authorization",
                &auth_value,
            )
            .await?;

        let parsed: FirecrawlSearchResponse = serde_json::from_str(&response)
            .map_err(|e| HsxError::Search(format!("Firecrawl parse: {e}")))?;

        debug!(
            "Firecrawl: {} results (success={})",
            parsed.data.len(),
            parsed.success
        );

        let results = parsed
            .data
            .into_iter()
            .enumerate()
            .filter(|(_, r)| !r.url.is_empty())
            .map(|(i, r)| {
                // Use markdown content as rich snippet (truncated)
                let snippet = if let Some(ref md) = r.markdown {
                    let chars: String = md.chars().take(1500).collect();
                    if md.len() > 1500 {
                        format!("{chars}...")
                    } else {
                        chars
                    }
                } else {
                    r.description.unwrap_or_default()
                };

                ResultItem {
                    title: r.title.unwrap_or_else(|| "Untitled".into()),
                    url: r.url,
                    snippet,
                    rank: (i + 1) as u32,
                    backend: BackendId::Firecrawl,
                    score: None,
                    published_date: None,
                }
            })
            .collect();

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_firecrawl_response() {
        let json = r##"{
            "success": true,
            "data": [
                {
                    "url": "https://example.com",
                    "title": "Test",
                    "description": "A test page",
                    "markdown": "# Test Page Content"
                }
            ]
        }"##;
        let parsed: FirecrawlSearchResponse = serde_json::from_str(json).unwrap();
        assert!(parsed.success);
        assert_eq!(parsed.data.len(), 1);
        assert!(parsed.data[0].markdown.is_some());
    }

    #[test]
    fn parse_firecrawl_empty() {
        let json = r#"{"success": true, "data": []}"#;
        let parsed: FirecrawlSearchResponse = serde_json::from_str(json).unwrap();
        assert!(parsed.data.is_empty());
    }
}
