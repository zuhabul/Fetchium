//! Serper.dev Google Search API backend — fast Google results + Scholar + News.
//!
//! Returns 10 organic results with snippets in ~1.5s.
//! Also queries Scholar and News endpoints for diverse source types.
//! API key required: set `SERPER_API_KEY` env var or `search.serper_api_key` in config.

use crate::error::FetchiumResult;
use crate::http::HttpClient;
use crate::search::{SearchBackend, SearchContext, TimeRange};
use crate::types::{BackendId, ResultItem};
use async_trait::async_trait;
use chrono::Datelike;
use serde::Deserialize;
use tracing::debug;

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

use std::sync::atomic::{AtomicUsize, Ordering};

/// Serper search backend — queries Google Search, Scholar, and News in parallel.
pub struct SerperBackend {
    http: HttpClient,
    api_keys: Vec<String>,
    current_key_index: AtomicUsize,
}

impl SerperBackend {
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

    async fn search_with_rotation(
        &self,
        endpoint: &str,
        query: &str,
        num: u32,
        tbs: Option<&str>,
    ) -> Vec<ResultItem> {
        let num_keys = self.api_keys.len().max(1);

        for _ in 0..num_keys {
            let api_key = self.get_key();
            let is_scholar = endpoint.contains("/scholar");
            let _is_news = endpoint.contains("/news");

            let mut body_json = serde_json::json!({
                "q": query,
                "num": num,
            });

            if let Some(tbs_val) = tbs {
                if !is_scholar {
                    body_json["tbs"] = serde_json::Value::String(tbs_val.to_string());
                } else if tbs_val.starts_with("qdr:y") {
                    // Scholar uses as_ylo for year filtering
                    let current_year = chrono::Utc::now().year();
                    body_json["as_ylo"] = serde_json::Value::String((current_year - 1).to_string());
                }
            }

            let body = body_json.to_string();

            let response = self
                .http
                .post_json_with_header(endpoint, &body, "X-API-KEY", api_key)
                .await;

            match response {
                Ok(text) => {
                    if endpoint.contains("/scholar") {
                        let parsed: SerperScholarResponse = match serde_json::from_str(&text) {
                            Ok(p) => p,
                            Err(_) => return vec![],
                        };
                        return parsed
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
                            .collect();
                    } else if endpoint.contains("/news") {
                        let parsed: SerperNewsResponse = match serde_json::from_str(&text) {
                            Ok(p) => p,
                            Err(_) => return vec![],
                        };
                        return parsed
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
                            .collect();
                    } else {
                        let parsed: SerperSearchResponse = match serde_json::from_str(&text) {
                            Ok(p) => p,
                            Err(_) => return vec![],
                        };
                        return parsed
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
                            .collect();
                    }
                }
                Err(e) => {
                    if let crate::error::FetchiumError::Structured(ref se) = e {
                        if se.message.contains("400")
                            || se.message.contains("credits")
                            || se.message.contains("429")
                        {
                            tracing::warn!(
                                "Serper key exhausted or limited, rotating... Error: {}",
                                se.message
                            );
                            self.rotate_key();
                            continue;
                        }
                    }
                    return vec![];
                }
            }
        }
        vec![]
    }
}

impl SerperBackend {
    fn time_range_to_tbs(tr: Option<TimeRange>) -> Option<&'static str> {
        match tr {
            Some(TimeRange::Day) => Some("qdr:d"),
            Some(TimeRange::Week) => Some("qdr:w"),
            Some(TimeRange::Month) => Some("qdr:m"),
            Some(TimeRange::Year) => Some("qdr:y"),
            None => None,
        }
    }
}

#[async_trait]
impl SearchBackend for SerperBackend {
    fn id(&self) -> BackendId {
        BackendId::Serper
    }

    async fn search(&self, query: &str, max_results: u32) -> FetchiumResult<Vec<ResultItem>> {
        let (organic, scholar, news) = tokio::join!(
            self.search_with_rotation("https://google.serper.dev/search", query, max_results, None),
            self.search_with_rotation("https://google.serper.dev/scholar", query, 5, None),
            self.search_with_rotation("https://google.serper.dev/news", query, 5, None),
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

    async fn search_with_context(
        &self,
        query: &str,
        max_results: u32,
        ctx: &SearchContext,
    ) -> FetchiumResult<Vec<ResultItem>> {
        let tbs = Self::time_range_to_tbs(ctx.time_range);
        let (organic, scholar, news) = tokio::join!(
            self.search_with_rotation("https://google.serper.dev/search", query, max_results, tbs),
            self.search_with_rotation("https://google.serper.dev/scholar", query, 5, None),
            self.search_with_rotation("https://google.serper.dev/news", query, 5, None),
        );

        let mut results = organic;
        results.extend(scholar);
        results.extend(news);

        debug!(
            "Serper: {} total results (organic + scholar + news) [tbs={:?}]",
            results.len(),
            tbs
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
