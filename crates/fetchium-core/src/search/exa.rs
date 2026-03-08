//! Exa AI neural search backend — semantic search with content extraction.
//!
//! Uses next-link prediction for semantic relevance beyond keyword matching.
//! Returns results with text content, highlights, author, and published date.
//! API key required: set `EXA_API_KEY` env var or `search.exa_api_key` in config.

use crate::error::{FetchiumError, FetchiumResult};
use crate::http::HttpClient;
use crate::search::{SearchBackend, SearchContext, TimeRange};
use crate::types::{BackendId, ResultItem};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExaRequest<'a> {
    query: &'a str,
    #[serde(rename = "type")]
    search_type: &'a str,
    num_results: u32,
    contents: ExaContents,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_published_date: Option<&'a str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExaContents {
    text: ExaTextConfig,
    highlights: ExaHighlightConfig,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExaTextConfig {
    max_characters: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExaHighlightConfig {
    num_sentences: u32,
    highlights_per_url: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExaResponse {
    #[serde(default)]
    results: Vec<ExaResult>,
    #[serde(default)]
    search_time: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExaResult {
    title: String,
    url: String,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    highlights: Vec<String>,
    #[serde(default)]
    highlight_scores: Vec<f64>,
    #[serde(default)]
    published_date: Option<String>,
    #[serde(default)]
    score: Option<f64>,
}

use std::sync::atomic::{AtomicUsize, Ordering};

/// Exa AI neural search backend — semantic search with content.
pub struct ExaBackend {
    http: HttpClient,
    api_keys: Vec<String>,
    current_key_index: AtomicUsize,
}

impl ExaBackend {
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

impl ExaBackend {
    async fn search_inner(
        &self,
        query: &str,
        max_results: u32,
        start_date: Option<&str>,
    ) -> FetchiumResult<Vec<ResultItem>> {
        let mut last_err = None;
        let num_keys = self.api_keys.len().max(1);

        for _ in 0..num_keys {
            let api_key = self.get_key();
            let request = ExaRequest {
                query,
                search_type: "auto",
                num_results: max_results.min(10),
                contents: ExaContents {
                    text: ExaTextConfig {
                        max_characters: 1500,
                    },
                    highlights: ExaHighlightConfig {
                        num_sentences: 3,
                        highlights_per_url: 3,
                    },
                },
                start_published_date: start_date,
            };

            let body = serde_json::to_string(&request)
                .map_err(|e| FetchiumError::Search(format!("Exa serialization: {e}")))?;

            let result = self
                .http
                .post_json_with_header("https://api.exa.ai/search", &body, "x-api-key", api_key)
                .await;

            match result {
                Ok(response) => {
                    let parsed: ExaResponse = serde_json::from_str(&response)
                        .map_err(|e| FetchiumError::Search(format!("Exa parse: {e}")))?;

                    debug!(
                        "Exa: {} results in {:.0}ms [start_date={:?}]",
                        parsed.results.len(),
                        parsed.search_time.unwrap_or(0.0),
                        start_date,
                    );

                    let results = parsed
                        .results
                        .into_iter()
                        .enumerate()
                        .map(|(i, r)| {
                            let snippet = if !r.highlights.is_empty() {
                                r.highlights.join(" ... ")
                            } else {
                                r.text.unwrap_or_default()
                            };

                            let score = r.score.or_else(|| r.highlight_scores.first().copied());

                            ResultItem {
                                title: r.title,
                                url: r.url,
                                snippet,
                                rank: (i + 1) as u32,
                                backend: BackendId::Exa,
                                score,
                                published_date: r.published_date,
                            }
                        })
                        .collect();

                    return Ok(results);
                }
                Err(e) => {
                    if let FetchiumError::Structured(ref se) = e {
                        if se.message.contains("401") || se.message.contains("403") || se.message.contains("429") || se.message.contains("limit") {
                            tracing::warn!("Exa key exhausted or limited, rotating... Error: {}", se.message);
                            self.rotate_key();
                            last_err = Some(e);
                            continue;
                        }
                    }
                    return Err(e);
                }
            }
        }

        Err(last_err.unwrap_or(FetchiumError::Search("Exa: All keys exhausted".into())))
    }

    fn time_range_to_date(tr: Option<TimeRange>) -> Option<String> {
        let now = chrono::Utc::now();
        match tr {
            Some(TimeRange::Day) => Some(
                (now - chrono::Duration::days(1))
                    .format("%Y-%m-%dT%H:%M:%SZ")
                    .to_string(),
            ),
            Some(TimeRange::Week) => Some(
                (now - chrono::Duration::days(7))
                    .format("%Y-%m-%dT%H:%M:%SZ")
                    .to_string(),
            ),
            Some(TimeRange::Month) => Some(
                (now - chrono::Duration::days(30))
                    .format("%Y-%m-%dT%H:%M:%SZ")
                    .to_string(),
            ),
            Some(TimeRange::Year) => Some(
                (now - chrono::Duration::days(365))
                    .format("%Y-%m-%dT%H:%M:%SZ")
                    .to_string(),
            ),
            None => None,
        }
    }
}

#[async_trait]
impl SearchBackend for ExaBackend {
    fn id(&self) -> BackendId {
        BackendId::Exa
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
        let date_str = Self::time_range_to_date(ctx.time_range);
        self.search_inner(query, max_results, date_str.as_deref())
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_exa_response() {
        let json = r#"{
            "results": [
                {
                    "title": "Test Article",
                    "url": "https://example.com",
                    "text": "Full text here",
                    "highlights": ["Key finding 1", "Key finding 2"],
                    "highlightScores": [0.95, 0.87],
                    "publishedDate": "2025-06-15",
                    "score": 0.92
                }
            ],
            "searchTime": 800.5
        }"#;
        let parsed: ExaResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.results.len(), 1);
        assert_eq!(parsed.results[0].highlights.len(), 2);
    }

    #[test]
    fn parse_exa_empty() {
        let json = r#"{"results": [], "searchTime": 100.0}"#;
        let parsed: ExaResponse = serde_json::from_str(json).unwrap();
        assert!(parsed.results.is_empty());
    }
}
