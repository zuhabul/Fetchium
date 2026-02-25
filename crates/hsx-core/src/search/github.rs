//! GitHub search backend — REST API v3 repository search.
//!
//! Searches GitHub repositories sorted by star count. Unauthenticated requests
//! are limited to 10 req/min; set `GITHUB_TOKEN` for 30 req/min.
//!
//! GitHub's API requires a `User-Agent` header and recommends an `Accept` header.

use crate::error::HsxResult;
use crate::http::HttpClient;
use crate::search::SearchBackend;
use crate::types::{BackendId, ResultItem};
use async_trait::async_trait;
use serde::Deserialize;
use tracing::debug;

/// GitHub REST API search repositories endpoint.
const GITHUB_API: &str = "https://api.github.com/search/repositories";

#[derive(Debug, Deserialize)]
struct GhSearchResponse {
    items: Vec<GhRepo>,
}

#[derive(Debug, Deserialize)]
struct GhRepo {
    full_name: String,
    html_url: String,
    description: Option<String>,
    stargazers_count: u32,
    language: Option<String>,
    updated_at: Option<String>,
    topics: Option<Vec<String>>,
}

/// GitHub repository search backend.
///
/// Uses the GitHub REST API v3. Set `GITHUB_TOKEN` for higher rate limits
/// (30 req/min authenticated vs 10 req/min unauthenticated).
pub struct GithubBackend {
    /// Stored so the struct field is used; requests use a fresh client for custom headers.
    #[allow(dead_code)]
    http: HttpClient,
    token: Option<String>,
}

impl GithubBackend {
    /// Create a new GitHub backend. Reads `GITHUB_TOKEN` from the environment.
    pub fn new(http: HttpClient) -> Self {
        let token = std::env::var("GITHUB_TOKEN").ok();
        Self { http, token }
    }

    /// Create a GitHub backend with an explicit token (for testing or config injection).
    pub fn with_token(http: HttpClient, token: impl Into<String>) -> Self {
        Self {
            http,
            token: Some(token.into()),
        }
    }
}

#[async_trait]
impl SearchBackend for GithubBackend {
    fn id(&self) -> BackendId {
        BackendId::Github
    }

    async fn search(&self, query: &str, max_results: u32) -> HsxResult<Vec<ResultItem>> {
        let per_page = max_results.min(30);
        let url = format!(
            "{GITHUB_API}?q={}&sort=stars&order=desc&per_page={per_page}",
            urlencoding_encode(query)
        );

        // GitHub API requires User-Agent and recommends the Accept header.
        // We also optionally attach an auth token for higher rate limits.
        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .user_agent("HyperSearchX/0.1")
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("GitHub: failed to build client: {e}");
                return Ok(vec![]);
            }
        };

        let mut req = client
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28");

        if let Some(ref token) = self.token {
            req = req.header("Authorization", format!("Bearer {token}"));
        }

        let resp = match req.send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("GitHub request failed: {e}");
                return Ok(vec![]);
            }
        };

        // Handle rate limiting gracefully
        let status = resp.status();
        if status == reqwest::StatusCode::FORBIDDEN
            || status == reqwest::StatusCode::TOO_MANY_REQUESTS
        {
            tracing::warn!("GitHub rate limited (HTTP {status})");
            return Ok(vec![]);
        }

        if !status.is_success() {
            tracing::warn!("GitHub HTTP {status}");
            return Ok(vec![]);
        }

        let body = match resp.text().await {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!("GitHub body read failed: {e}");
                return Ok(vec![]);
            }
        };

        let parsed: GhSearchResponse = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("GitHub JSON parse failed: {e}");
                return Ok(vec![]);
            }
        };

        let results = parsed
            .items
            .into_iter()
            .enumerate()
            .map(|(i, repo)| {
                let snippet = build_snippet(&repo);
                ResultItem {
                    title: repo.full_name,
                    url: repo.html_url,
                    snippet,
                    rank: (i + 1) as u32,
                    backend: BackendId::Github,
                    score: None,
                    published_date: repo.updated_at,
                }
            })
            .collect::<Vec<_>>();

        debug!("GitHub: {} results for {:?}", results.len(), query);
        Ok(results)
    }
}

/// Build a human-readable snippet from repository metadata.
fn build_snippet(repo: &GhRepo) -> String {
    let mut parts = Vec::new();

    if let Some(ref desc) = repo.description {
        if !desc.is_empty() {
            parts.push(desc.clone());
        }
    }

    let star_info = format!("{} stars", repo.stargazers_count);
    let lang_info = if let Some(ref lang) = repo.language {
        format!("{star_info} | {lang}")
    } else {
        star_info
    };
    parts.push(lang_info);

    if let Some(ref topics) = repo.topics {
        if !topics.is_empty() {
            parts.push(format!("Topics: {}", topics.join(", ")));
        }
    }

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
    fn parse_github_response() {
        let json = r#"{"items":[{
            "full_name":"rust-lang/rust",
            "html_url":"https://github.com/rust-lang/rust",
            "description":"Rust language",
            "stargazers_count":100000,
            "language":"Rust",
            "updated_at":"2024-01-01",
            "topics":["rust","systems"]
        }]}"#;
        let resp: GhSearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.items.len(), 1);
        assert_eq!(resp.items[0].full_name, "rust-lang/rust");
        assert_eq!(resp.items[0].stargazers_count, 100_000);
        assert_eq!(resp.items[0].language.as_deref(), Some("Rust"));
    }

    #[test]
    fn parse_github_response_no_description() {
        let json = r#"{"items":[{
            "full_name":"foo/bar",
            "html_url":"https://github.com/foo/bar",
            "description":null,
            "stargazers_count":10,
            "language":null,
            "updated_at":null,
            "topics":[]
        }]}"#;
        let resp: GhSearchResponse = serde_json::from_str(json).unwrap();
        assert!(resp.items[0].description.is_none());
        assert!(resp.items[0].language.is_none());
    }

    #[test]
    fn build_snippet_includes_stars_and_lang() {
        let repo = GhRepo {
            full_name: "foo/bar".to_string(),
            html_url: "https://github.com/foo/bar".to_string(),
            description: Some("A great project".to_string()),
            stargazers_count: 500,
            language: Some("Rust".to_string()),
            updated_at: None,
            topics: Some(vec!["rust".to_string(), "cli".to_string()]),
        };
        let s = build_snippet(&repo);
        assert!(s.contains("500 stars"), "Expected stars in: {s}");
        assert!(s.contains("Rust"), "Expected language in: {s}");
        assert!(s.contains("A great project"), "Expected desc in: {s}");
        assert!(s.contains("rust, cli"), "Expected topics in: {s}");
    }

    #[test]
    fn build_snippet_no_description_no_topics() {
        let repo = GhRepo {
            full_name: "foo/bar".to_string(),
            html_url: "https://github.com/foo/bar".to_string(),
            description: None,
            stargazers_count: 42,
            language: None,
            updated_at: None,
            topics: None,
        };
        let s = build_snippet(&repo);
        assert!(s.contains("42 stars"), "Expected stars in: {s}");
        // Without language or topics, snippet is just the star count
        assert!(!s.contains("Topics:"));
    }

    #[test]
    fn result_item_mapping() {
        let repo = GhRepo {
            full_name: "torvalds/linux".to_string(),
            html_url: "https://github.com/torvalds/linux".to_string(),
            description: Some("Linux kernel".to_string()),
            stargazers_count: 200_000,
            language: Some("C".to_string()),
            updated_at: Some("2024-02-01".to_string()),
            topics: Some(vec!["kernel".to_string()]),
        };
        let snippet = build_snippet(&repo);
        let item = ResultItem {
            title: repo.full_name.clone(),
            url: repo.html_url.clone(),
            snippet,
            rank: 1,
            backend: BackendId::Github,
            score: None,
            published_date: repo.updated_at.clone(),
        };
        assert_eq!(item.title, "torvalds/linux");
        assert_eq!(item.published_date.as_deref(), Some("2024-02-01"));
        assert!(item.snippet.contains("200000 stars"));
    }
}
