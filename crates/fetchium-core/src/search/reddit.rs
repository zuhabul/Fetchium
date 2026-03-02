//! Reddit search backend — JSON API (`search.json`).
//!
//! Reddit's public JSON API appends `.json` to the search URL. No auth is
//! required, but a descriptive `User-Agent` must be provided to avoid 429s.

use crate::error::HsxResult;
use crate::http::HttpClient;
use crate::search::SearchBackend;
use crate::types::{BackendId, ResultItem};
use async_trait::async_trait;
use serde::Deserialize;
use tracing::debug;

/// Reddit search JSON endpoint.
const REDDIT_SEARCH: &str = "https://www.reddit.com/search.json";

#[derive(Debug, Deserialize)]
struct RedditResponse {
    data: RedditData,
}

#[derive(Debug, Deserialize)]
struct RedditData {
    children: Vec<RedditChild>,
}

#[derive(Debug, Deserialize)]
struct RedditChild {
    data: RedditPost,
}

#[derive(Debug, Deserialize)]
struct RedditPost {
    title: String,
    url: String,
    selftext: Option<String>,
    subreddit: String,
    score: Option<i64>,
    created_utc: Option<f64>,
    permalink: Option<String>,
}

/// Reddit search backend using the public JSON API.
///
/// Reddit requires a descriptive `User-Agent` to avoid rate limiting.
/// The client is built once at construction and reused across requests
/// to take advantage of connection pooling and TLS session resumption.
pub struct RedditBackend {
    /// Pre-built client with Reddit-specific User-Agent and connection pooling.
    client: reqwest::Client,
}

impl RedditBackend {
    /// Create a new Reddit backend with the given HTTP client.
    pub fn new(_http: HttpClient) -> Self {
        let client = reqwest::Client::builder()
            // Keep Reddit fail-fast to avoid dragging overall search tail latency.
            .timeout(std::time::Duration::from_secs(3))
            .connect_timeout(std::time::Duration::from_secs(2))
            .user_agent("Fetchium:search-bot:v0.1 (research tool)")
            .pool_max_idle_per_host(4)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build()
            .unwrap_or_default();
        Self { client }
    }
}

#[async_trait]
impl SearchBackend for RedditBackend {
    fn id(&self) -> BackendId {
        BackendId::Reddit
    }

    async fn search(&self, query: &str, max_results: u32) -> HsxResult<Vec<ResultItem>> {
        let limit = max_results.min(25);
        let url = format!(
            "{REDDIT_SEARCH}?q={}&limit={limit}&sort=relevance&type=link",
            urlencoding_encode(query)
        );

        let resp = match self.client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Reddit request failed: {e}");
                return Ok(vec![]);
            }
        };

        if !resp.status().is_success() {
            tracing::warn!("Reddit HTTP {}", resp.status());
            return Ok(vec![]);
        }

        let body = match resp.text().await {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!("Reddit body read failed: {e}");
                return Ok(vec![]);
            }
        };

        let parsed: RedditResponse = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Reddit JSON parse failed: {e}");
                return Ok(vec![]);
            }
        };

        let results = parsed
            .data
            .children
            .into_iter()
            .enumerate()
            .map(|(i, child)| {
                let post = child.data;
                let snippet = build_snippet(&post);

                // For self-posts (Reddit-internal links), use the permalink
                let final_url = if post.url.starts_with("https://www.reddit.com")
                    || post.url.starts_with("http://www.reddit.com")
                    || post.url.starts_with('/')
                {
                    post.permalink
                        .as_deref()
                        .map(|p| format!("https://www.reddit.com{p}"))
                        .unwrap_or(post.url.clone())
                } else {
                    post.url.clone()
                };

                // Convert UTC epoch to ISO-8601 string
                let published = post.created_utc.and_then(|ts| {
                    chrono::DateTime::from_timestamp(ts as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                });

                ResultItem {
                    title: post.title,
                    url: final_url,
                    snippet,
                    rank: (i + 1) as u32,
                    backend: BackendId::Reddit,
                    score: None,
                    published_date: published,
                }
            })
            .collect::<Vec<_>>();

        debug!("Reddit: {} results for {:?}", results.len(), query);
        Ok(results)
    }
}

/// Build a human-readable snippet from post metadata.
fn build_snippet(post: &RedditPost) -> String {
    let mut parts = Vec::new();

    // Include selftext if present and not deleted
    if let Some(ref text) = post.selftext {
        let text = text.trim();
        if !text.is_empty() && text != "[deleted]" && text != "[removed]" {
            let char_count = text.chars().count();
            let truncated: String = text.chars().take(200).collect();
            parts.push(if char_count > 200 {
                format!("{truncated}...")
            } else {
                truncated
            });
        }
    }

    let meta = format!("r/{} | {} points", post.subreddit, post.score.unwrap_or(0));
    parts.push(meta);

    parts.join(" — ")
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
    fn parse_reddit_response() {
        let json = r#"{"data":{"children":[{"data":{
            "title":"Rust 2024",
            "url":"https://reddit.com/r/rust/comments/123",
            "selftext":"Great year for Rust!",
            "subreddit":"rust",
            "score":500,
            "created_utc":1700000000.0,
            "permalink":"/r/rust/comments/123"
        }}]}}"#;
        let resp: RedditResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.data.children.len(), 1);
        let post = &resp.data.children[0].data;
        assert_eq!(post.title, "Rust 2024");
        assert_eq!(post.subreddit, "rust");
        assert_eq!(post.score, Some(500));
    }

    #[test]
    fn snippet_includes_subreddit_and_score() {
        let post = RedditPost {
            title: "test".to_string(),
            url: "https://example.com".to_string(),
            selftext: None,
            subreddit: "rust".to_string(),
            score: Some(42),
            created_utc: None,
            permalink: None,
        };
        let s = build_snippet(&post);
        assert!(s.contains("r/rust"), "Expected 'r/rust' in: {s}");
        assert!(s.contains("42"), "Expected '42' in: {s}");
    }

    #[test]
    fn snippet_includes_selftext_when_present() {
        let post = RedditPost {
            title: "test".to_string(),
            url: "https://example.com".to_string(),
            selftext: Some("This is the body of the post.".to_string()),
            subreddit: "programming".to_string(),
            score: Some(10),
            created_utc: None,
            permalink: None,
        };
        let s = build_snippet(&post);
        assert!(s.contains("This is the body"), "Expected selftext in: {s}");
    }

    #[test]
    fn snippet_skips_deleted_selftext() {
        for deleted in &["[deleted]", "[removed]"] {
            let post = RedditPost {
                title: "test".to_string(),
                url: "https://example.com".to_string(),
                selftext: Some(deleted.to_string()),
                subreddit: "test".to_string(),
                score: Some(1),
                created_utc: None,
                permalink: None,
            };
            let s = build_snippet(&post);
            assert!(
                !s.contains("[deleted]") && !s.contains("[removed]"),
                "Should skip deleted text, got: {s}"
            );
        }
    }

    #[test]
    fn self_post_uses_permalink() {
        let post = RedditPost {
            title: "Ask Reddit: something".to_string(),
            url: "https://www.reddit.com/r/AskReddit/comments/abc".to_string(),
            selftext: Some("My question".to_string()),
            subreddit: "AskReddit".to_string(),
            score: Some(100),
            created_utc: None,
            permalink: Some("/r/AskReddit/comments/abc".to_string()),
        };
        let final_url = if post.url.starts_with("https://www.reddit.com") {
            post.permalink
                .as_deref()
                .map(|p| format!("https://www.reddit.com{p}"))
                .unwrap_or(post.url.clone())
        } else {
            post.url.clone()
        };
        assert_eq!(final_url, "https://www.reddit.com/r/AskReddit/comments/abc");
    }

    #[test]
    fn external_link_post_uses_url() {
        let post = RedditPost {
            title: "Interesting article".to_string(),
            url: "https://example.com/article".to_string(),
            selftext: None,
            subreddit: "programming".to_string(),
            score: Some(200),
            created_utc: None,
            permalink: Some("/r/programming/comments/xyz".to_string()),
        };
        let final_url = if post.url.starts_with("https://www.reddit.com") {
            post.permalink
                .as_deref()
                .map(|p| format!("https://www.reddit.com{p}"))
                .unwrap_or(post.url.clone())
        } else {
            post.url.clone()
        };
        assert_eq!(final_url, "https://example.com/article");
    }

    #[test]
    fn created_utc_converts_to_iso8601() {
        // 1700000000 UTC = 2023-11-14T22:13:20Z
        let ts = 1_700_000_000.0f64;
        let published = chrono::DateTime::from_timestamp(ts as i64, 0)
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string());
        assert!(published.is_some());
        let s = published.unwrap();
        assert!(s.starts_with("2023-11-"), "Got: {s}");
    }
}
