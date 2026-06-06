//! StackOverflow search backend — StackExchange API v2.3.
//!
//! Uses the `/questions/search` endpoint. No API key is required for basic usage
//! (300 requests/day). Set `STACKEXCHANGE_KEY` for a higher quota (10,000 req/day).
//!
//! StackExchange returns gzip-compressed responses by default; reqwest handles
//! decompression automatically when built with the `gzip` feature.

use crate::error::FetchiumError;
use crate::error::FetchiumResult;
use crate::http::HttpClient;
use crate::search::SearchBackend;
use crate::types::{BackendId, ResultItem};
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;

/// StackExchange API v2.3 search endpoint.
const SO_API: &str = "https://api.stackexchange.com/2.3/search";
/// Backoff window when StackExchange starts rate-limiting aggressively.
const SO_COOLDOWN_SECS: u64 = 300;
static SO_COOLDOWN_UNTIL_MS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Deserialize)]
struct SoResponse {
    items: Vec<SoQuestion>,
}

#[derive(Debug, Deserialize)]
struct SoQuestion {
    title: String,
    link: String,
    score: i32,
    answer_count: u32,
    is_answered: bool,
    #[serde(default)]
    tags: Vec<String>,
    creation_date: Option<u64>,
    body_markdown: Option<String>,
}

/// StackOverflow search backend using the StackExchange API.
///
/// Searches for questions whose titles match the query. The `withbody` filter
/// includes `body_markdown` for richer snippets (counts toward quota).
/// The client is built once and reused to enable connection pooling and
/// avoid repeated TLS handshakes (StackExchange always returns gzip).
pub struct StackOverflowBackend {
    /// Pre-built client with gzip support and connection pooling.
    client: reqwest::Client,
    api_key: Option<String>,
}

impl StackOverflowBackend {
    /// Create a new StackOverflow backend. Reads `STACKEXCHANGE_KEY` from the environment.
    pub fn new(_http: HttpClient) -> Self {
        let api_key = std::env::var("STACKEXCHANGE_KEY").ok();
        let client = reqwest::Client::builder()
            // Keep StackOverflow fail-fast to avoid long-tail latency in aggregate search.
            .timeout(std::time::Duration::from_secs(4))
            .connect_timeout(std::time::Duration::from_secs(2))
            .user_agent("Fetchium/0.1")
            .gzip(true)
            .pool_max_idle_per_host(4)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build()
            .unwrap_or_default();
        Self { client, api_key }
    }

    /// Create a StackOverflow backend with an explicit API key.
    pub fn with_key(_http: HttpClient, api_key: impl Into<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(4))
            .connect_timeout(std::time::Duration::from_secs(2))
            .user_agent("Fetchium/0.1")
            .gzip(true)
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
impl SearchBackend for StackOverflowBackend {
    fn id(&self) -> BackendId {
        BackendId::StackOverflow
    }

    async fn search(&self, query: &str, max_results: u32) -> FetchiumResult<Vec<ResultItem>> {
        let now = now_ms();
        let cooldown_until = SO_COOLDOWN_UNTIL_MS.load(Ordering::Relaxed);
        if now < cooldown_until {
            debug!("StackOverflow in cooldown window, skipping request");
            return Ok(vec![]);
        }

        let page_size = max_results.min(30);
        let mut url = format!(
            "{SO_API}?intitle={}&site=stackoverflow&order=desc&sort=relevance\
             &pagesize={page_size}&filter=withbody",
            urlencoding_encode(query)
        );
        if let Some(ref key) = self.api_key {
            url.push_str(&format!("&key={key}"));
        }

        let resp = match self
            .client
            .get(&url)
            .header("Accept-Encoding", "gzip")
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("StackOverflow request failed: {e}");
                return Ok(vec![]);
            }
        };

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            if status.as_u16() == 429 || body.contains("throttle_violation") {
                let until = now_ms() + SO_COOLDOWN_SECS * 1000;
                SO_COOLDOWN_UNTIL_MS.store(until, Ordering::Relaxed);
                return Err(FetchiumError::Search(format!(
                    "StackOverflow rate-limited (HTTP {status}) — cooling down for {}s",
                    SO_COOLDOWN_SECS
                )));
            }
            if status.is_server_error() {
                return Err(FetchiumError::Search(format!(
                    "StackOverflow upstream server error: HTTP {status}"
                )));
            }
            debug!("StackOverflow non-success HTTP {status}, skipping");
            return Ok(vec![]);
        }

        let body = match resp.text().await {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!("StackOverflow body read failed: {e}");
                return Ok(vec![]);
            }
        };

        let parsed: SoResponse = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("StackOverflow JSON parse failed: {e}");
                return Ok(vec![]);
            }
        };

        let results = parsed
            .items
            .into_iter()
            .enumerate()
            .map(|(i, q)| {
                let snippet = build_snippet(&q);
                let published = q.creation_date.and_then(|ts| {
                    chrono::DateTime::from_timestamp(ts as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                });
                ResultItem {
                    title: decode_html_entities(&q.title),
                    url: q.link,
                    snippet,
                    rank: (i + 1) as u32,
                    backend: BackendId::StackOverflow,
                    score: None,
                    published_date: published,
                }
            })
            .collect::<Vec<_>>();

        debug!("StackOverflow: {} results for {:?}", results.len(), query);
        Ok(results)
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Build a human-readable snippet from question metadata.
fn build_snippet(q: &SoQuestion) -> String {
    let mut parts = Vec::new();

    // Include body if available
    if let Some(ref body) = q.body_markdown {
        let stripped = strip_html(body);
        let stripped = stripped.trim();
        if !stripped.is_empty() {
            let char_count = stripped.chars().count();
            let truncated: String = stripped.chars().take(200).collect();
            parts.push(if char_count > 200 {
                format!("{truncated}...")
            } else {
                truncated
            });
        }
    }

    let status = if q.is_answered {
        "Answered"
    } else {
        "Unanswered"
    };
    parts.push(format!(
        "{status} | {} answers | score {}",
        q.answer_count, q.score
    ));

    if !q.tags.is_empty() {
        parts.push(format!("Tags: {}", q.tags.join(", ")));
    }

    parts.join(" — ")
}

/// Strip HTML tags from a string (for StackExchange body_markdown fields).
fn strip_html(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    result.trim().to_string()
}

/// Decode the five predefined HTML entities in question titles.
fn decode_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
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
    fn parse_so_response() {
        let json = r#"{"items":[{
            "title":"How to use Rust?",
            "link":"https://stackoverflow.com/q/123",
            "score":10,
            "answer_count":3,
            "is_answered":true,
            "tags":["rust","programming"],
            "creation_date":1700000000
        }]}"#;
        let resp: SoResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.items.len(), 1);
        let q = &resp.items[0];
        assert_eq!(q.title, "How to use Rust?");
        assert!(q.is_answered);
        assert_eq!(q.answer_count, 3);
        assert_eq!(q.score, 10);
        assert_eq!(q.tags, ["rust", "programming"]);
    }

    #[test]
    fn parse_so_response_empty_tags() {
        let json = r#"{"items":[{
            "title":"Test question",
            "link":"https://stackoverflow.com/q/999",
            "score":0,
            "answer_count":0,
            "is_answered":false,
            "tags":[],
            "creation_date":null
        }]}"#;
        let resp: SoResponse = serde_json::from_str(json).unwrap();
        assert!(resp.items[0].tags.is_empty());
        assert!(resp.items[0].creation_date.is_none());
    }

    #[test]
    fn decode_html_entities_works() {
        assert_eq!(decode_html_entities("A &amp; B"), "A & B");
        assert_eq!(decode_html_entities("&lt;code&gt;"), "<code>");
        assert_eq!(
            decode_html_entities("it&#39;s &quot;quoted&quot;"),
            "it's \"quoted\""
        );
    }

    #[test]
    fn strip_html_removes_tags() {
        assert_eq!(strip_html("<p>Hello <b>world</b></p>"), "Hello world");
        assert_eq!(strip_html("plain text"), "plain text");
    }

    #[test]
    fn snippet_includes_answer_status_answered() {
        let q = SoQuestion {
            title: "test".to_string(),
            link: "https://so.com/q/1".to_string(),
            score: 5,
            answer_count: 2,
            is_answered: true,
            tags: vec!["rust".to_string()],
            creation_date: None,
            body_markdown: None,
        };
        let s = build_snippet(&q);
        assert!(s.contains("Answered"), "Expected 'Answered' in: {s}");
        assert!(s.contains("2 answers"), "Expected '2 answers' in: {s}");
        assert!(s.contains("score 5"), "Expected 'score 5' in: {s}");
        assert!(s.contains("Tags: rust"), "Expected 'Tags: rust' in: {s}");
    }

    #[test]
    fn snippet_includes_answer_status_unanswered() {
        let q = SoQuestion {
            title: "test".to_string(),
            link: "https://so.com/q/2".to_string(),
            score: -1,
            answer_count: 0,
            is_answered: false,
            tags: vec![],
            creation_date: None,
            body_markdown: None,
        };
        let s = build_snippet(&q);
        assert!(s.contains("Unanswered"), "Expected 'Unanswered' in: {s}");
    }

    #[test]
    fn snippet_includes_body_when_present() {
        let q = SoQuestion {
            title: "test".to_string(),
            link: "https://so.com/q/3".to_string(),
            score: 3,
            answer_count: 1,
            is_answered: true,
            tags: vec![],
            creation_date: None,
            body_markdown: Some("<p>How do I do this specific thing in Rust?</p>".to_string()),
        };
        let s = build_snippet(&q);
        assert!(
            s.contains("How do I do this specific thing"),
            "Expected body text in: {s}"
        );
    }

    #[test]
    fn creation_date_converts_to_iso8601() {
        let ts: u64 = 1_700_000_000;
        let published = chrono::DateTime::from_timestamp(ts as i64, 0)
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string());
        assert!(published.is_some());
        let s = published.unwrap();
        assert!(s.starts_with("2023-11-"), "Got: {s}");
    }
}
