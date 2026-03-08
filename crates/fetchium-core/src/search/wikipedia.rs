//! Wikipedia search backend — MediaWiki API (no auth required).
//!
//! Uses the `action=query&list=search` endpoint which returns JSON.
//! High authority signal for factual queries (PRD §15 — authority scoring).

use crate::error::FetchiumResult;
use crate::http::HttpClient;
use crate::search::SearchBackend;
use crate::types::{BackendId, ResultItem};
use async_trait::async_trait;
use serde::Deserialize;
use tracing::debug;

/// MediaWiki API base URL for English Wikipedia.
const WIKI_API: &str = "https://en.wikipedia.org/w/api.php";

#[derive(Debug, Deserialize)]
struct WikiResponse {
    query: WikiQuery,
}

#[derive(Debug, Deserialize)]
struct WikiQuery {
    search: Vec<WikiSearchResult>,
}

#[derive(Debug, Deserialize)]
struct WikiSearchResult {
    title: String,
    snippet: String,
    #[serde(rename = "wordcount")]
    #[allow(dead_code)]
    word_count: Option<u32>,
    timestamp: Option<String>,
}

/// Wikipedia search backend using the MediaWiki search API.
///
/// Converts titles to article URLs and strips HTML highlight tags from snippets.
pub struct WikipediaBackend {
    http: HttpClient,
}

impl WikipediaBackend {
    /// Create a new Wikipedia backend with the given HTTP client.
    pub fn new(http: HttpClient) -> Self {
        Self { http }
    }
}

#[async_trait]
impl SearchBackend for WikipediaBackend {
    fn id(&self) -> BackendId {
        BackendId::Wikipedia
    }

    async fn search(&self, query: &str, max_results: u32) -> FetchiumResult<Vec<ResultItem>> {
        let limit = max_results.min(20);
        let url = format!(
            "{WIKI_API}?action=query&list=search&srsearch={}&srlimit={limit}\
             &srprop=snippet%7Cwordcount%7Ctimestamp&format=json&origin=*",
            urlencoding_encode(query)
        );

        let body = match self.http.fetch_text(&url).await {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!("Wikipedia request failed: {e}");
                return Ok(vec![]);
            }
        };

        let resp: WikiResponse = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Wikipedia JSON parse failed: {e}");
                return Ok(vec![]);
            }
        };

        let results = resp
            .query
            .search
            .into_iter()
            .enumerate()
            .map(|(i, r)| {
                // Build the canonical article URL from the title
                let title_slug = r.title.replace(' ', "_");
                let article_url = format!("https://en.wikipedia.org/wiki/{title_slug}");
                ResultItem {
                    title: r.title,
                    url: article_url,
                    // The snippet contains HTML highlight spans — strip them
                    snippet: strip_html(&r.snippet),
                    rank: (i + 1) as u32,
                    backend: BackendId::Wikipedia,
                    score: None,
                    published_date: r.timestamp,
                }
            })
            .collect::<Vec<_>>();

        debug!("Wikipedia: {} results for {:?}", results.len(), query);
        Ok(results)
    }
}

/// Strip HTML tags from a string (for MediaWiki highlight snippets).
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
    fn strip_html_removes_tags() {
        assert_eq!(strip_html("Hello <b>world</b>"), "Hello world");
        assert_eq!(
            strip_html(r#"<span class="searchmatch">Rust</span> programming"#),
            "Rust programming"
        );
    }

    #[test]
    fn strip_html_empty_string() {
        assert_eq!(strip_html(""), "");
    }

    #[test]
    fn strip_html_no_tags() {
        assert_eq!(strip_html("plain text"), "plain text");
    }

    #[test]
    fn parse_wiki_response() {
        let json = r#"{"query":{"search":[
            {"title":"Rust (programming language)","snippet":"Systems <b>language</b>","wordcount":5000,"timestamp":"2024-01-01T00:00:00Z"}
        ]}}"#;
        let resp: WikiResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.query.search.len(), 1);
        assert_eq!(resp.query.search[0].title, "Rust (programming language)");
        assert_eq!(
            resp.query.search[0].timestamp.as_deref(),
            Some("2024-01-01T00:00:00Z")
        );
    }

    #[test]
    fn article_url_from_title() {
        let title = "Rust (programming language)";
        let slug = title.replace(' ', "_");
        let url = format!("https://en.wikipedia.org/wiki/{slug}");
        assert_eq!(
            url,
            "https://en.wikipedia.org/wiki/Rust_(programming_language)"
        );
    }

    #[test]
    fn article_url_simple_title() {
        let title = "Quantum computing";
        let slug = title.replace(' ', "_");
        let url = format!("https://en.wikipedia.org/wiki/{slug}");
        assert_eq!(url, "https://en.wikipedia.org/wiki/Quantum_computing");
    }

    #[test]
    fn urlencoding_encodes_special_chars() {
        let encoded = urlencoding_encode("rust programming");
        assert!(!encoded.is_empty());
        // Should not contain raw spaces
        assert!(!encoded.contains(' '));
    }
}
