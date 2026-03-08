//! HackerNews backend — Algolia search API (no auth required).
//!
//! Searches HN stories using the Algolia API at `hn.algolia.com`. Great for
//! tech content, programming discussions, and startup/product announcements.

use crate::error::FetchiumResult;
use crate::http::HttpClient;
use crate::search::SearchBackend;
use crate::types::{BackendId, ResultItem};
use async_trait::async_trait;
use serde::Deserialize;
use tracing::debug;

/// Algolia HN search API endpoint.
const HN_API: &str = "https://hn.algolia.com/api/v1/search";

#[derive(Debug, Deserialize)]
struct HnResponse {
    hits: Vec<HnHit>,
}

#[derive(Debug, Deserialize)]
struct HnHit {
    #[serde(rename = "objectID")]
    object_id: String,
    title: Option<String>,
    url: Option<String>,
    #[serde(rename = "story_text")]
    story_text: Option<String>,
    author: Option<String>,
    #[serde(rename = "created_at")]
    created_at: Option<String>,
    points: Option<u32>,
}

/// HackerNews search backend using the Algolia API.
///
/// Filters to `story` tag by default. Falls back to the HN item URL when a
/// story has no external link (Ask HN, Show HN, etc.).
pub struct HackerNewsBackend {
    http: HttpClient,
}

impl HackerNewsBackend {
    /// Create a new HackerNews backend with the given HTTP client.
    pub fn new(http: HttpClient) -> Self {
        Self { http }
    }
}

#[async_trait]
impl SearchBackend for HackerNewsBackend {
    fn id(&self) -> BackendId {
        BackendId::HackerNews
    }

    async fn search(&self, query: &str, max_results: u32) -> FetchiumResult<Vec<ResultItem>> {
        let hits_per_page = max_results.min(30);
        let url = format!(
            "{HN_API}?query={}&hitsPerPage={hits_per_page}&tags=story",
            urlencoding_encode(query)
        );

        let body = match self.http.fetch_text(&url).await {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!("HackerNews request failed: {e}");
                return Ok(vec![]);
            }
        };

        let resp: HnResponse = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("HackerNews JSON parse failed: {e}");
                return Ok(vec![]);
            }
        };

        let results = resp
            .hits
            .into_iter()
            .enumerate()
            .filter_map(|(i, hit)| {
                // Skip hits with no title
                let title = hit.title?;

                // Use provided URL or fall back to the HN story URL
                let url = hit.url.unwrap_or_else(|| {
                    format!("https://news.ycombinator.com/item?id={}", hit.object_id)
                });

                // Build snippet from story text or metadata
                let snippet = hit
                    .story_text
                    .as_deref()
                    .filter(|t| !t.is_empty())
                    .map(|t| truncate_text(t, 200))
                    .unwrap_or_else(|| {
                        format!(
                            "HN story by {} | {} points",
                            hit.author.as_deref().unwrap_or("unknown"),
                            hit.points.unwrap_or(0)
                        )
                    });

                Some(ResultItem {
                    title,
                    url,
                    snippet,
                    rank: (i + 1) as u32,
                    backend: BackendId::HackerNews,
                    score: None,
                    published_date: hit.created_at,
                })
            })
            .collect::<Vec<_>>();

        debug!("HackerNews: {} results for {:?}", results.len(), query);
        Ok(results)
    }
}

/// Truncate text to `max_chars` characters, appending `...` if truncated.
fn truncate_text(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        return s.to_string();
    }
    let truncated: String = s.chars().take(max_chars).collect();
    format!("{truncated}...")
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
    fn parse_hn_response_with_url() {
        let json = r#"{"hits":[{
            "objectID":"123",
            "title":"Rust 2024",
            "url":"https://blog.rust-lang.org",
            "story_text":null,
            "author":"user1",
            "created_at":"2024-01-01",
            "points":100
        }]}"#;
        let resp: HnResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.hits.len(), 1);
        assert_eq!(resp.hits[0].title.as_deref(), Some("Rust 2024"));
        assert_eq!(
            resp.hits[0].url.as_deref(),
            Some("https://blog.rust-lang.org")
        );
        assert_eq!(resp.hits[0].points, Some(100));
    }

    #[test]
    fn parse_hn_response_no_title_skipped() {
        let json = r#"{"hits":[
            {"objectID":"1","title":null,"url":"https://a.com","story_text":null,"author":"x","created_at":"2024-01-01","points":5},
            {"objectID":"2","title":"Real Story","url":"https://b.com","story_text":null,"author":"y","created_at":"2024-01-01","points":10}
        ]}"#;
        let resp: HnResponse = serde_json::from_str(json).unwrap();
        let results: Vec<ResultItem> = resp
            .hits
            .into_iter()
            .enumerate()
            .filter_map(|(i, hit)| {
                let title = hit.title?;
                let url = hit.url.unwrap_or_else(|| {
                    format!("https://news.ycombinator.com/item?id={}", hit.object_id)
                });
                Some(ResultItem {
                    title,
                    url,
                    snippet: String::new(),
                    rank: (i + 1) as u32,
                    backend: BackendId::HackerNews,
                    score: None,
                    published_date: hit.created_at,
                })
            })
            .collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Real Story");
    }

    #[test]
    fn fallback_to_hn_url_when_no_url() {
        let hit = HnHit {
            object_id: "42".to_string(),
            title: Some("Ask HN: Test".to_string()),
            url: None,
            story_text: None,
            author: None,
            created_at: None,
            points: None,
        };
        let url = hit
            .url
            .unwrap_or_else(|| format!("https://news.ycombinator.com/item?id={}", hit.object_id));
        assert_eq!(url, "https://news.ycombinator.com/item?id=42");
    }

    #[test]
    fn truncate_text_short_string() {
        let s = "short text";
        assert_eq!(truncate_text(s, 200), s);
    }

    #[test]
    fn truncate_text_long_string() {
        let long = "a".repeat(300);
        let truncated = truncate_text(&long, 200);
        assert!(truncated.ends_with("..."), "Expected '...' suffix");
        // 200 chars + 3 for "..."
        assert_eq!(truncated.len(), 203);
    }

    #[test]
    fn truncate_text_exact_length() {
        let s = "a".repeat(200);
        let result = truncate_text(&s, 200);
        assert_eq!(result, s);
        assert!(!result.ends_with("..."));
    }
}
