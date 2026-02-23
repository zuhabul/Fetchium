//! SearXNG meta-search backend — JSON API with multi-instance fallback.
//!
//! Public instances often disable JSON; we try each in order and return empty
//! on total failure (the FallbackChain handles degraded quality gracefully).

use crate::error::HsxResult;
use crate::http::HttpClient;
use crate::search::SearchBackend;
use crate::types::{BackendId, ResultItem};
use async_trait::async_trait;
use serde::Deserialize;
use tracing::{debug, warn};

/// Public SearXNG instances that tend to support the JSON API.
/// Rotated through in order when the first fails.
const SEARXNG_INSTANCES: &[&str] = &[
    "https://searx.be",
    "https://search.ononoki.org",
    "https://search.sapti.me",
];

#[derive(Debug, Deserialize)]
struct SearxResponse {
    results: Vec<SearxResult>,
}

#[derive(Debug, Deserialize)]
struct SearxResult {
    url: String,
    title: String,
    content: Option<String>,
    #[allow(dead_code)]
    engine: Option<String>,
    score: Option<f64>,
    #[serde(rename = "publishedDate")]
    published_date: Option<String>,
}

/// SearXNG meta-search backend.
///
/// Uses the JSON search API (`/search?format=json`). Rotates through multiple
/// public instances for reliability. Returns empty results when all instances
/// fail — the FallbackChain will try the next backend.
pub struct SearxngBackend {
    http: HttpClient,
}

impl SearxngBackend {
    /// Create a new SearXNG backend using the default public instance list.
    pub fn new(http: HttpClient) -> Self {
        Self { http }
    }
}

#[async_trait]
impl SearchBackend for SearxngBackend {
    fn id(&self) -> BackendId {
        BackendId::Searxng
    }

    async fn search(&self, query: &str, max_results: u32) -> HsxResult<Vec<ResultItem>> {
        // Try each instance in order; return on first success
        for instance in SEARXNG_INSTANCES {
            let url = format!(
                "{}/search?q={}&format=json&pageno=1&language=en",
                instance,
                urlencoding_encode(query)
            );

            match self.http.fetch_text(&url).await {
                Ok(body) => {
                    match serde_json::from_str::<SearxResponse>(&body) {
                        Ok(resp) => {
                            let results = resp
                                .results
                                .into_iter()
                                .take(max_results as usize)
                                .enumerate()
                                .filter_map(|(i, r)| {
                                    if r.url.starts_with("http") {
                                        Some(ResultItem {
                                            title: r.title,
                                            url: r.url,
                                            snippet: r.content.unwrap_or_default(),
                                            rank: (i + 1) as u32,
                                            backend: BackendId::Searxng,
                                            score: r.score,
                                            published_date: r.published_date,
                                        })
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>();
                            debug!("SearXNG {}: {} results", instance, results.len());
                            return Ok(results);
                        }
                        Err(e) => {
                            warn!("SearXNG {instance} JSON parse error: {e}");
                            // Instance may have returned HTML (JSON disabled) — try next
                            continue;
                        }
                    }
                }
                Err(e) => {
                    warn!("SearXNG {instance} request failed: {e}");
                    continue;
                }
            }
        }

        // All instances failed — soft fail with empty results
        warn!("SearXNG: all instances exhausted for query {:?}", query);
        Ok(vec![])
    }
}

/// Percent-encode a query string for use in URLs using `application/x-www-form-urlencoded`.
fn urlencoding_encode(s: &str) -> String {
    url::form_urlencoded::Serializer::new(String::new())
        .append_key_only(s)
        .finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_searx_response() {
        let json = r#"{"results":[{"url":"https://example.com","title":"Example","content":"A test page"}]}"#;
        let resp: SearxResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.results.len(), 1);
        assert_eq!(resp.results[0].title, "Example");
    }

    #[test]
    fn parse_searx_response_with_score() {
        let json = r#"{"results":[{"url":"https://example.com","title":"Example","content":"A test page","score":0.95,"publishedDate":"2024-01-01"}]}"#;
        let resp: SearxResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.results[0].score, Some(0.95));
        assert_eq!(
            resp.results[0].published_date.as_deref(),
            Some("2024-01-01")
        );
    }

    #[test]
    fn urlencoding_spaces() {
        let encoded = urlencoding_encode("hello world");
        // form_urlencoded encodes spaces as '+'
        assert!(encoded.contains('+') || encoded.contains("%20"),
            "Expected '+' or '%20' in {:?}", encoded);
    }

    #[test]
    fn filters_non_http_urls() {
        let json = r#"{"results":[
            {"url":"https://good.com","title":"Good","content":"ok"},
            {"url":"javascript:void(0)","title":"Bad","content":"bad"}
        ]}"#;
        let resp: SearxResponse = serde_json::from_str(json).unwrap();
        let items: Vec<ResultItem> = resp
            .results
            .into_iter()
            .filter_map(|r| {
                if r.url.starts_with("http") {
                    Some(ResultItem {
                        title: r.title,
                        url: r.url,
                        snippet: r.content.unwrap_or_default(),
                        rank: 1,
                        backend: BackendId::Searxng,
                        score: None,
                        published_date: None,
                    })
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].url, "https://good.com");
    }

    #[test]
    fn max_results_respected() {
        let json = r#"{"results":[
            {"url":"https://a.com","title":"A","content":"a"},
            {"url":"https://b.com","title":"B","content":"b"},
            {"url":"https://c.com","title":"C","content":"c"}
        ]}"#;
        let resp: SearxResponse = serde_json::from_str(json).unwrap();
        let results: Vec<ResultItem> = resp
            .results
            .into_iter()
            .take(2)
            .enumerate()
            .filter_map(|(i, r)| {
                if r.url.starts_with("http") {
                    Some(ResultItem {
                        title: r.title,
                        url: r.url,
                        snippet: r.content.unwrap_or_default(),
                        rank: (i + 1) as u32,
                        backend: BackendId::Searxng,
                        score: None,
                        published_date: None,
                    })
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(results.len(), 2);
    }
}
