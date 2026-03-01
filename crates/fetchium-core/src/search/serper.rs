//! Serper.dev Google Search API backend — fast Google results + Scholar + News.
//!
//! Returns 10 organic results with snippets in ~1.5s.
//! Also queries Scholar and News endpoints for diverse source types.
//! API key required: set `SERPER_API_KEY` env var or `search.serper_api_key` in config.

use crate::error::HsxResult;
use crate::http::HttpClient;
use crate::search::SearchBackend;
use crate::types::{BackendId, ResultItem};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Serialize)]
struct SerperRequest<'a> {
    q: &'a str,
    num: u32,
}

#[derive(Debug, Deserialize)]
struct SerperSearchResponse {
    #[serde(default)]
    organic: Vec<SerperOrganic>,
}

#[derive(Debug, Deserialize)]
struct SerperOrganic {
    title: String,
    link: String,
    #[serde(default)]
    snippet: String,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    position: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct SerperScholarResponse {
    #[serde(default)]
    organic: Vec<SerperScholarResult>,
}

#[derive(Debug, Deserialize)]
struct SerperScholarResult {
    title: String,
    link: String,
    #[serde(default)]
    snippet: String,
    #[serde(default)]
    year: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SerperNewsResponse {
    #[serde(default)]
    news: Vec<SerperNews>,
}

#[derive(Debug, Deserialize)]
struct SerperNews {
    title: String,
    link: String,
    #[serde(default)]
    snippet: String,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    source: Option<String>,
}

/// Serper search backend — queries Google Search, Scholar, and News in parallel.
pub struct SerperBackend {
    http: HttpClient,
    api_key: String,
}

impl SerperBackend {
    pub fn new(http: HttpClient, api_key: String) -> Self {
        Self { http, api_key }
    }

    async fn search_organic(&self, query: &str, max_results: u32) -> Vec<ResultItem> {
        let request = SerperRequest {
            q: query,
            num: max_results.min(10),
        };
        let body = match serde_json::to_string(&request) {
            Ok(b) => b,
            Err(_) => return vec![],
        };

        let response = self
            .http
            .post_json_with_header(
                "https://google.serper.dev/search",
                &body,
                "X-API-KEY",
                &self.api_key,
            )
            .await;

        match response {
            Ok(text) => {
                let parsed: SerperSearchResponse = match serde_json::from_str(&text) {
                    Ok(p) => p,
                    Err(_) => return vec![],
                };

                parsed
                    .organic
                    .into_iter()
                    .enumerate()
                    .map(|(i, r)| ResultItem {
                        title: r.title,
                        url: r.link,
                        snippet: r.snippet,
                        rank: r.position.unwrap_or((i + 1) as u32),
                        backend: BackendId::Serper,
                        score: None,
                        published_date: r.date,
                    })
                    .collect()
            }
            Err(_) => vec![],
        }
    }

    async fn search_scholar(&self, query: &str) -> Vec<ResultItem> {
        let request = SerperRequest { q: query, num: 5 };
        let body = match serde_json::to_string(&request) {
            Ok(b) => b,
            Err(_) => return vec![],
        };

        let response = self
            .http
            .post_json_with_header(
                "https://google.serper.dev/scholar",
                &body,
                "X-API-KEY",
                &self.api_key,
            )
            .await;

        match response {
            Ok(text) => {
                let parsed: SerperScholarResponse = match serde_json::from_str(&text) {
                    Ok(p) => p,
                    Err(_) => return vec![],
                };

                parsed
                    .organic
                    .into_iter()
                    .enumerate()
                    .map(|(i, r)| ResultItem {
                        title: r.title,
                        url: r.link,
                        snippet: r.snippet,
                        rank: (i + 1) as u32,
                        backend: BackendId::Serper,
                        score: None,
                        published_date: r.year.map(|y| format!("{y}-01-01")),
                    })
                    .collect()
            }
            Err(_) => vec![],
        }
    }

    async fn search_news(&self, query: &str) -> Vec<ResultItem> {
        let request = SerperRequest { q: query, num: 5 };
        let body = match serde_json::to_string(&request) {
            Ok(b) => b,
            Err(_) => return vec![],
        };

        let response = self
            .http
            .post_json_with_header(
                "https://google.serper.dev/news",
                &body,
                "X-API-KEY",
                &self.api_key,
            )
            .await;

        match response {
            Ok(text) => {
                let parsed: SerperNewsResponse = match serde_json::from_str(&text) {
                    Ok(p) => p,
                    Err(_) => return vec![],
                };

                parsed
                    .news
                    .into_iter()
                    .enumerate()
                    .map(|(i, r)| ResultItem {
                        title: r.title,
                        url: r.link,
                        snippet: if let Some(ref src) = r.source {
                            format!("[{}] {}", src, r.snippet)
                        } else {
                            r.snippet
                        },
                        rank: (i + 1) as u32,
                        backend: BackendId::Serper,
                        score: None,
                        published_date: r.date,
                    })
                    .collect()
            }
            Err(_) => vec![],
        }
    }
}

#[async_trait]
impl SearchBackend for SerperBackend {
    fn id(&self) -> BackendId {
        BackendId::Serper
    }

    async fn search(&self, query: &str, max_results: u32) -> HsxResult<Vec<ResultItem>> {
        let (organic, scholar, news) = tokio::join!(
            self.search_organic(query, max_results),
            self.search_scholar(query),
            self.search_news(query),
        );

        let mut results = organic;
        results.extend(scholar);
        results.extend(news);

        debug!(
            "Serper: {} total results (organic + scholar + news)",
            results.len()
        );

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_serper_organic() {
        let json = r#"{"organic": [{"title": "Test", "link": "https://example.com", "snippet": "A test", "position": 1}]}"#;
        let parsed: SerperSearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.organic.len(), 1);
    }

    #[test]
    fn parse_serper_scholar() {
        let json = r#"{"organic": [{"title": "Paper", "link": "https://arxiv.org/abs/123", "snippet": "Abstract", "year": "2025"}]}"#;
        let parsed: SerperScholarResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.organic.len(), 1);
    }

    #[test]
    fn parse_serper_news() {
        let json = r#"{"news": [{"title": "Breaking", "link": "https://news.com/1", "snippet": "News", "source": "CNN"}]}"#;
        let parsed: SerperNewsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.news.len(), 1);
    }
}
