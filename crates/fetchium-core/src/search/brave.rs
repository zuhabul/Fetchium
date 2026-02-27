//! Brave Search API backend — free tier (2000 req/month), requires API key.
//!
//! Set the `BRAVE_API_KEY` environment variable to enable. Without a key,
//! this backend returns empty results immediately (soft fail).

use crate::error::HsxResult;
use crate::http::HttpClient;
use crate::search::SearchBackend;
use crate::types::{BackendId, ResultItem};
use async_trait::async_trait;
use serde::Deserialize;
use tracing::debug;

/// Brave Search REST API endpoint.
const BRAVE_API: &str = "https://api.search.brave.com/res/v1/web/search";

#[derive(Debug, Deserialize)]
struct BraveResponse {
    web: Option<BraveWebResults>,
}

#[derive(Debug, Deserialize)]
struct BraveWebResults {
    results: Vec<BraveResult>,
}

#[derive(Debug, Deserialize)]
struct BraveResult {
    title: String,
    url: String,
    description: Option<String>,
    page_age: Option<String>,
}

/// Brave Search API backend.
///
/// Requires the `BRAVE_API_KEY` environment variable. Free tier allows
/// 2000 requests per month. Returns empty results without a key.
/// The client is built once and reused for connection pooling efficiency.
pub struct BraveBackend {
    /// Pre-built client with connection pooling.
    client: reqwest::Client,
    api_key: Option<String>,
}

impl BraveBackend {
    /// Create a new Brave backend. Reads `BRAVE_API_KEY` from the environment.
    pub fn new(_http: HttpClient) -> Self {
        let api_key = std::env::var("BRAVE_API_KEY").ok();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .user_agent("Fetchium/0.1")
            .pool_max_idle_per_host(4)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build()
            .unwrap_or_default();
        Self { client, api_key }
    }

    /// Create a Brave backend with an explicit API key (for testing or config injection).
    pub fn with_key(_http: HttpClient, api_key: impl Into<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .user_agent("Fetchium/0.1")
            .pool_max_idle_per_host(4)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build()
            .unwrap_or_default();
        Self {
            client,
            api_key: Some(api_key.into()),
        }
    }
}

#[async_trait]
impl SearchBackend for BraveBackend {
    fn id(&self) -> BackendId {
        BackendId::Brave
    }

    async fn search(&self, query: &str, max_results: u32) -> HsxResult<Vec<ResultItem>> {
        let Some(ref key) = self.api_key else {
            debug!("Brave: no API key set (BRAVE_API_KEY), skipping");
            return Ok(vec![]);
        };

        let count = max_results.min(20);
        let url = format!(
            "{BRAVE_API}?q={}&count={count}&search_lang=en",
            urlencoding_encode(query)
        );

        let resp = match self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .header("Accept-Encoding", "gzip")
            .header("X-Subscription-Token", key.as_str())
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Brave request failed: {e}");
                return Ok(vec![]);
            }
        };

        if !resp.status().is_success() {
            tracing::warn!("Brave HTTP {}", resp.status());
            return Ok(vec![]);
        }

        let body = match resp.text().await {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!("Brave body read failed: {e}");
                return Ok(vec![]);
            }
        };

        let parsed: BraveResponse = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Brave JSON parse failed: {e}");
                return Ok(vec![]);
            }
        };

        let results = parsed
            .web
            .map(|w| w.results)
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(i, r)| ResultItem {
                title: r.title,
                url: r.url,
                snippet: r.description.unwrap_or_default(),
                rank: (i + 1) as u32,
                backend: BackendId::Brave,
                score: None,
                published_date: r.page_age,
            })
            .collect::<Vec<_>>();

        debug!("Brave: {} results for {:?}", results.len(), query);
        Ok(results)
    }
}

/// Percent-encode a query string for use in URLs.
fn urlencoding_encode(s: &str) -> String {
    url::form_urlencoded::Serializer::new(String::new())
        .append_key_only(s)
        .finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_brave_response_full() {
        let json = r#"{"web":{"results":[{"title":"Rust","url":"https://rust-lang.org","description":"A systems language","page_age":"2024-01-01"}]}}"#;
        let resp: BraveResponse = serde_json::from_str(json).unwrap();
        let web = resp.web.unwrap();
        assert_eq!(web.results.len(), 1);
        assert_eq!(web.results[0].title, "Rust");
        assert_eq!(web.results[0].url, "https://rust-lang.org");
        assert_eq!(
            web.results[0].description.as_deref(),
            Some("A systems language")
        );
        assert_eq!(web.results[0].page_age.as_deref(), Some("2024-01-01"));
    }

    #[test]
    fn parse_brave_response_no_web() {
        let json = r#"{"web":null}"#;
        let resp: BraveResponse = serde_json::from_str(json).unwrap();
        assert!(resp.web.is_none());
    }

    #[test]
    fn parse_brave_response_empty_results() {
        let json = r#"{"web":{"results":[]}}"#;
        let resp: BraveResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.web.unwrap().results.len(), 0);
    }

    #[test]
    fn result_item_mapping() {
        let brave_result = BraveResult {
            title: "Test Title".to_string(),
            url: "https://test.com".to_string(),
            description: Some("Test description".to_string()),
            page_age: Some("2024-06-01".to_string()),
        };
        let item = ResultItem {
            title: brave_result.title.clone(),
            url: brave_result.url.clone(),
            snippet: brave_result.description.clone().unwrap_or_default(),
            rank: 1,
            backend: BackendId::Brave,
            score: None,
            published_date: brave_result.page_age.clone(),
        };
        assert_eq!(item.title, "Test Title");
        assert_eq!(item.snippet, "Test description");
        assert_eq!(item.published_date.as_deref(), Some("2024-06-01"));
    }
}
