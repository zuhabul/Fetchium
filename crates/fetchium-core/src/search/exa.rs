//! Exa AI neural search backend — semantic search with content extraction.
//!
//! Uses next-link prediction for semantic relevance beyond keyword matching.
//! Returns results with text content, highlights, author, and published date.
//! API key required: set `EXA_API_KEY` env var or `search.exa_api_key` in config.

use crate::error::{HsxError, HsxResult};
use crate::http::HttpClient;
use crate::search::SearchBackend;
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

/// Exa AI neural search backend — semantic search with content.
pub struct ExaBackend {
    http: HttpClient,
    api_key: String,
}

impl ExaBackend {
    pub fn new(http: HttpClient, api_key: String) -> Self {
        Self { http, api_key }
    }
}

#[async_trait]
impl SearchBackend for ExaBackend {
    fn id(&self) -> BackendId {
        BackendId::Exa
    }

    async fn search(&self, query: &str, max_results: u32) -> HsxResult<Vec<ResultItem>> {
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
        };

        let body = serde_json::to_string(&request)
            .map_err(|e| HsxError::Search(format!("Exa serialization: {e}")))?;

        let response = self
            .http
            .post_json_with_header(
                "https://api.exa.ai/search",
                &body,
                "x-api-key",
                &self.api_key,
            )
            .await?;

        let parsed: ExaResponse = serde_json::from_str(&response)
            .map_err(|e| HsxError::Search(format!("Exa parse: {e}")))?;

        debug!(
            "Exa: {} results in {:.0}ms",
            parsed.results.len(),
            parsed.search_time.unwrap_or(0.0)
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

        Ok(results)
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
