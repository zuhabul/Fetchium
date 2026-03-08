//! Firecrawl search+scrape backend — search the web and get full markdown content.
//!
//! Combines search with content scraping in one API call.
//! Returns clean markdown content ready for LLM consumption.
//! API key required: set `FIRECRAWL_API_KEY` env var or `search.firecrawl_api_key` in config.

use crate::error::{FetchiumError, FetchiumResult};
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

use std::sync::atomic::{AtomicUsize, Ordering};

/// Firecrawl search+scrape backend — returns full markdown content.
pub struct FirecrawlBackend {
    http: HttpClient,
    api_keys: Vec<String>,
    current_key_index: AtomicUsize,
}

impl FirecrawlBackend {
    pub fn new(http: HttpClient, api_keys: Vec<String>) -> Self {
        Self {
            http,
            api_keys,
            current_key_index: AtomicUsize::new(0),
        }
    }

    fn get_key(&self) -> &str {
        if self.api_keys.is_empty() {
            return "";
        }
        let idx = self.current_key_index.load(Ordering::Relaxed) % self.api_keys.len();
        &self.api_keys[idx]
    }

    fn rotate_key(&self) {
        if !self.api_keys.is_empty() {
            self.current_key_index.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[async_trait]
impl SearchBackend for FirecrawlBackend {
    fn id(&self) -> BackendId {
        BackendId::Firecrawl
    }

    async fn search(&self, query: &str, max_results: u32) -> FetchiumResult<Vec<ResultItem>> {
        let mut last_err = None;
        let num_keys = self.api_keys.len().max(1);

        for _ in 0..num_keys {
            let api_key = self.get_key();
            let request = FirecrawlSearchRequest {
                query,
                limit: max_results.min(5),
            };

            let body = serde_json::to_string(&request)
                .map_err(|e| FetchiumError::Search(format!("Firecrawl serialization: {e}")))?;

            let auth_value = format!("Bearer {}", api_key);
            let result = self
                .http
                .post_json_with_header(
                    "https://api.firecrawl.dev/v1/search",
                    &body,
                    "Authorization",
                    &auth_value,
                )
                .await;

            match result {
                Ok(response) => {
                    let parsed: FirecrawlSearchResponse = serde_json::from_str(&response)
                        .map_err(|e| FetchiumError::Search(format!("Firecrawl parse: {e}")))?;

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

                    return Ok(results);
                }
                Err(e) => {
                    if let FetchiumError::Structured(ref se) = e {
                        if se.message.contains("401") || se.message.contains("402") || se.message.contains("429") || se.message.contains("limit") || se.message.contains("credits") {
                            tracing::warn!("Firecrawl key exhausted or limited, rotating... Error: {}", se.message);
                            self.rotate_key();
                            last_err = Some(e);
                            continue;
                        }
                    }
                    return Err(e);
                }
            }
        }

        Err(last_err.unwrap_or(FetchiumError::Search("Firecrawl: All keys exhausted".into())))
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
