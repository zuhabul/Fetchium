//! SearXNG meta-search backend — JSON API with local-first + public fallback.
//!
//! ## Priority order
//! 1. **`SEARXNG_URL` env var** — your self-hosted instance (unlimited, zero latency)
//! 2. **`***REMOVED***`** — default local Docker container (`~/searxng-local/`)
//! 3. **Public instances** — community fallbacks when local is unavailable
//!
//! ## Self-hosting (fully free, unlimited)
//! ```bash
//! cd ~/searxng-local && docker compose up -d
//! # SearXNG now available at ***REMOVED***
//! # Queries local instance first — no rate limits, no CAPTCHA, no API keys
//! ```
//!
//! Set `SEARXNG_URL=***REMOVED***` to make this the exclusive source.

use crate::error::FetchiumResult;
use crate::http::HttpClient;
use crate::search::{SearchBackend, SearchContext, TimeRange};
use crate::types::{BackendId, ResultItem};
use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;
#[allow(unused_imports)]
use tracing::info;
use tracing::{debug, warn};

/// Public SearXNG fallback instances (used only when local instance is unavailable).
///
/// The local instance at localhost:4040 is always tried first.
/// Public instances have JSON API confirmed as of early 2026.
const PUBLIC_SEARXNG_INSTANCES: &[&str] = &[
    "https://paulgo.io",         // Community, JSON API confirmed, no CF
    "https://search.inetol.net", // EU/DE, stable, JSON API confirmed
    "https://searxng.site",      // US, stable uptime
    "https://priv.au",           // AU, low latency Asia-Pacific
    "https://searx.be",          // Historical; occasionally offline or CF-blocked
    "https://search.sapti.me",   // Intermittent; kept as last resort
];

/// Fast high-signal engines for low-latency recall on the local SearXNG instance.
///
/// In practice, Brave/DuckDuckGo/Startpage are frequently suspended by CAPTCHA
/// or rate limits on self-hosted instances. Bias toward engines that remain
/// usable under load, then fall back to unrestricted search for extra recall.
const FAST_ENGINE_SET: &str = "google,bing,yandex,wikipedia";

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
/// Query priority:
/// 1. `SEARXNG_URL` environment variable (your self-hosted instance)
/// 2. `***REMOVED***` (default local Docker container)
/// 3. Public community instances (fallback only)
///
/// Self-hosted SearXNG is the recommended configuration: it aggregates Google,
/// Bing, Brave, DuckDuckGo and more — all in one request, no CAPTCHA, no limits.
pub struct SearxngBackend {
    http: HttpClient,
    /// Custom instance URL from SEARXNG_URL env var (takes priority).
    custom_url: Option<String>,
}

impl SearxngBackend {
    /// Create a new SearXNG backend.
    ///
    /// Reads `SEARXNG_URL` from the environment. If set, that instance is used
    /// exclusively. If not set, tries localhost:4040 then public instances.
    pub fn new(http: HttpClient) -> Self {
        let custom_url = std::env::var("SEARXNG_URL").ok();
        if let Some(ref url) = custom_url {
            info!("SearXNG: using custom instance from SEARXNG_URL: {url}");
        }
        Self { http, custom_url }
    }

    /// Build the ordered instance list to try.
    ///
    /// Order: custom env var → localhost:4040 → public fallbacks
    fn instance_list(&self) -> Vec<&str> {
        let mut list: Vec<&str> = Vec::new();

        // 1. Custom env var instance (highest priority)
        if let Some(ref url) = self.custom_url {
            list.push(url.as_str());
            return list; // Env var = exclusive — don't fall through to others
        }

        // 2. Local Docker instance (always try first when no env override)
        list.push("***REMOVED***");

        // 3. Public fallbacks (only used when local is down)
        list.extend_from_slice(PUBLIC_SEARXNG_INSTANCES);
        list
    }

    /// Parse a SearXNG JSON response body into ResultItems.
    fn parse_response(body: &str, max_results: usize) -> Option<Vec<ResultItem>> {
        match serde_json::from_str::<SearxResponse>(body) {
            Ok(resp) => {
                let results = resp
                    .results
                    .into_iter()
                    .take(max_results)
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
                Some(results)
            }
            Err(_) => None, // HTML response — JSON disabled on this instance
        }
    }
}

impl SearxngBackend {
    fn searx_language(ctx: Option<&SearchContext>) -> &'static str {
        match ctx.and_then(|ctx| ctx.language.as_deref()) {
            Some("fr") => "fr",
            Some("de") => "de",
            Some("es") => "es",
            Some("it") => "it",
            Some("pt") => "pt",
            Some("ru") => "ru",
            Some("jp") => "ja",
            Some("kr") => "ko",
            Some("cn") => "zh",
            Some("ae") => "ar",
            Some("in") => "hi",
            Some("th") => "th",
            Some("gr") => "el",
            _ => "auto",
        }
    }

    /// Internal search with optional time range and category filtering.
    async fn search_inner(
        &self,
        query: &str,
        max_results: u32,
        time_range: Option<TimeRange>,
        categories: Option<&str>,
        engines: Option<&str>,
        ctx: Option<&SearchContext>,
    ) -> FetchiumResult<Vec<ResultItem>> {
        let instances = self.instance_list();
        let is_local_first = self.custom_url.is_none();
        let language = Self::searx_language(ctx);

        let time_param = match time_range {
            Some(TimeRange::Day) => "&time_range=day",
            Some(TimeRange::Week) => "&time_range=week",
            Some(TimeRange::Month) => "&time_range=month",
            Some(TimeRange::Year) => "&time_range=year",
            None => "",
        };

        let cat_param = match categories {
            Some(cats) => format!("&categories={cats}"),
            None => String::new(),
        };
        let engines_param = match engines {
            Some(es) if !es.trim().is_empty() => format!("&engines={es}"),
            _ => String::new(),
        };

        for (i, instance) in instances.iter().enumerate() {
            let is_local = instance.contains("localhost") || instance.contains("127.0.0.1");
            let url_primary = format!(
                "{}/search?q={}&format=json&pageno=1&language={}{}{}{}",
                instance,
                urlencoding_encode(query),
                language,
                time_param,
                cat_param,
                engines_param,
            );

            match self.http.fetch_text(&url_primary).await {
                Ok(body) => {
                    if let Some(results) = Self::parse_response(&body, max_results as usize) {
                        if is_local && is_local_first && i == 0 {
                            info!("SearXNG local ({}): {} results ✓", instance, results.len());
                        } else {
                            debug!("SearXNG {}: {} results", instance, results.len());
                        }
                        return Ok(results);
                    }
                    warn!("SearXNG {instance}: JSON parse failed (HTML response? JSON disabled)");
                }
                Err(e) => {
                    if is_local {
                        debug!("SearXNG local unavailable, falling back to public instances: {e}");
                    } else {
                        warn!("SearXNG {instance} request failed: {e}");
                    }
                }
            }

            // If fast engine restriction produced no usable results, retry once without
            // restriction on the same instance for recall.
            if !engines_param.is_empty() {
                let url_fallback = format!(
                    "{}/search?q={}&format=json&pageno=1&language={}{}{}",
                    instance,
                    urlencoding_encode(query),
                    language,
                    time_param,
                    cat_param,
                );
                match self.http.fetch_text(&url_fallback).await {
                    Ok(body) => {
                        if let Some(results) = Self::parse_response(&body, max_results as usize) {
                            if !results.is_empty() {
                                debug!(
                                    "SearXNG {}: recovered {} results via unrestricted fallback",
                                    instance,
                                    results.len()
                                );
                                return Ok(results);
                            }
                        }
                    }
                    Err(e) => {
                        debug!("SearXNG {instance} unrestricted fallback failed: {e}");
                    }
                }
            }
        }

        warn!("SearXNG: all instances exhausted for query {:?}", query);
        Ok(vec![])
    }
}

#[async_trait]
impl SearchBackend for SearxngBackend {
    fn id(&self) -> BackendId {
        BackendId::Searxng
    }

    async fn search(&self, query: &str, max_results: u32) -> FetchiumResult<Vec<ResultItem>> {
        self.search_inner(query, max_results, None, None, Some(FAST_ENGINE_SET), None)
            .await
    }

    async fn search_with_context(
        &self,
        query: &str,
        max_results: u32,
        ctx: &SearchContext,
    ) -> FetchiumResult<Vec<ResultItem>> {
        use crate::rank::fusion::QueryIntent;

        match ctx.intent {
            QueryIntent::CurrentEvents => {
                // Current-events queries can trigger slow/brittle public instances.
                // Use bounded sequential fallbacks to avoid long-tail stalls.
                let news = tokio::time::timeout(
                    Duration::from_secs(5),
                    self.search_inner(
                        query,
                        max_results,
                        ctx.time_range,
                        Some("news"),
                        Some(FAST_ENGINE_SET),
                        Some(ctx),
                    ),
                )
                .await
                .ok()
                .and_then(Result::ok)
                .unwrap_or_default();
                if !news.is_empty() {
                    return Ok(news);
                }

                let general = tokio::time::timeout(
                    Duration::from_secs(5),
                    self.search_inner(
                        query,
                        max_results,
                        ctx.time_range,
                        Some("general"),
                        Some(FAST_ENGINE_SET),
                        Some(ctx),
                    ),
                )
                .await
                .ok()
                .and_then(Result::ok)
                .unwrap_or_default();
                let mut results = Vec::new();
                results.extend(general);
                Ok(results)
            }
            QueryIntent::Code => {
                self.search_inner(
                    query,
                    max_results,
                    ctx.time_range,
                    Some("it,general"),
                    Some(FAST_ENGINE_SET),
                    Some(ctx),
                )
                .await
            }
            QueryIntent::Academic => {
                self.search_inner(
                    query,
                    max_results,
                    ctx.time_range,
                    Some("science,general"),
                    Some(FAST_ENGINE_SET),
                    Some(ctx),
                )
                .await
            }
            _ => {
                self.search_inner(
                    query,
                    max_results,
                    ctx.time_range,
                    None,
                    Some(FAST_ENGINE_SET),
                    Some(ctx),
                )
                .await
            }
        }
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
    fn parse_response_basic() {
        let json = r#"{"results":[{"url":"https://example.com","title":"Example","content":"A test page"}]}"#;
        let results = SearxngBackend::parse_response(json, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Example");
        assert_eq!(results[0].url, "https://example.com");
        assert_eq!(results[0].snippet, "A test page");
    }

    #[test]
    fn parse_response_with_score_and_date() {
        let json = r#"{"results":[{"url":"https://example.com","title":"Example","content":"test","score":0.95,"publishedDate":"2024-01-01"}]}"#;
        let results = SearxngBackend::parse_response(json, 10).unwrap();
        assert_eq!(results[0].score, Some(0.95));
        assert_eq!(results[0].published_date.as_deref(), Some("2024-01-01"));
    }

    #[test]
    fn parse_response_filters_non_http() {
        let json = r#"{"results":[
            {"url":"https://good.com","title":"Good","content":"ok"},
            {"url":"javascript:void(0)","title":"Bad","content":"bad"}
        ]}"#;
        let results = SearxngBackend::parse_response(json, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "https://good.com");
    }

    #[test]
    fn parse_response_max_results() {
        let json = r#"{"results":[
            {"url":"https://a.com","title":"A","content":"a"},
            {"url":"https://b.com","title":"B","content":"b"},
            {"url":"https://c.com","title":"C","content":"c"}
        ]}"#;
        let results = SearxngBackend::parse_response(json, 2).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn parse_response_html_returns_none() {
        // HTML response (JSON disabled on that instance) → None
        let html = "<html><body>Not a JSON response</body></html>";
        assert!(SearxngBackend::parse_response(html, 10).is_none());
    }

    #[test]
    fn searx_language_defaults_to_auto() {
        assert_eq!(SearxngBackend::searx_language(None), "auto");
    }

    #[test]
    fn searx_language_maps_detected_languages() {
        let ctx = SearchContext {
            intent: crate::rank::fusion::QueryIntent::Factual,
            time_range: None,
            locale: Some("jp".to_string()),
            language: Some("jp".to_string()),
        };
        assert_eq!(SearxngBackend::searx_language(Some(&ctx)), "ja");
    }

    #[test]
    fn instance_list_local_first_by_default() {
        // Without env var, localhost:4040 should be first
        std::env::remove_var("SEARXNG_URL");
        let backend = SearxngBackend {
            http: crate::http::HttpClient::new(&crate::config::FetchiumConfig::default()).unwrap(),
            custom_url: None,
        };
        let instances = backend.instance_list();
        assert_eq!(instances[0], "***REMOVED***");
        assert!(instances.len() > 1, "Should include public fallbacks");
    }

    #[test]
    fn instance_list_custom_url_exclusive() {
        // With custom_url set, only that instance should be returned
        let backend = SearxngBackend {
            http: crate::http::HttpClient::new(&crate::config::FetchiumConfig::default()).unwrap(),
            custom_url: Some("http://my-searxng.example.com".to_string()),
        };
        let instances = backend.instance_list();
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0], "http://my-searxng.example.com");
    }

    #[test]
    fn urlencoding_spaces() {
        let encoded = urlencoding_encode("hello world");
        assert!(
            encoded.contains('+') || encoded.contains("%20"),
            "Expected '+' or '%20' in {:?}",
            encoded
        );
    }

    #[test]
    fn fast_engine_set_includes_google() {
        assert!(FAST_ENGINE_SET.split(',').any(|e| e == "google"));
    }
}
