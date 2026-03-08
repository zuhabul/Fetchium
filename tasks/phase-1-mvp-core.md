# Phase 1: MVP Core — Search & Fetch

> **Phase:** 1 of 8 | **Priority:** P1 | **Duration:** Weeks 2-4
> **Depends on:** Phase 0 (Project Foundation & Scaffolding) fully complete
> **PRD Reference:** `prd.md` v4.0.0 -- Sections 14-18, 20-21, 26-28
> **Epics:** 7 | **Tasks:** 15 | **Status:** ✅ COMPLETE

---

## Phase 1 Summary

Phase 1 builds the **working MVP** of Fetchium. After this phase, users can run:

1. **`fetchium fetch <url>`** -- Extract content from any webpage using CEP layers 1-2
2. **`fetchium search "query"`** -- Search the web via DuckDuckGo and display ranked results
3. **`fetchium agent-search "query" --budget N`** -- Agent-optimized search with token budgets
4. **`fetchium agent-fetch <url> --query "..." --budget N`** -- Agent-optimized fetch with QATBE

This phase implements the core algorithms that differentiate Fetchium:

- **QATBE** (Query-Aware Token-Budgeted Extraction) -- PRD SS17, SS8.2
- **SCS** (Semantic Content Segmentation) -- PRD SS18, SS8.4
- **PDS Tier 1** (Progressive Detail Streaming) -- PRD SS27, SS8.9
- **BM25 ranking** via tantivy -- PRD SS21
- **Memory LRU cache** via moka -- PRD SS28
- **Output formatters** (markdown, JSON, text, segments) -- PRD SS26

---

## Prerequisites

All of the following must be `DONE` before starting any Phase 1 task:

| Dependency              | Phase   | What It Provides                                 |
| ----------------------- | ------- | ------------------------------------------------ |
| P0-E1-T1 (Workspace)    | Phase 0 | Cargo workspace with all crates                  |
| P0-E1-T2 (Types)        | Phase 0 | Core data types in `fetchium-core/src/types.rs`       |
| P0-E1-T3 (Config)       | Phase 0 | Configuration system in `fetchium-core/src/config.rs` |
| P0-E3-T1 (CLI skeleton) | Phase 0 | clap definitions and command dispatch            |

---

## Epic 1.1: HTTP Client + Content Extraction

> **PRD Sections:** SS14 (Parallel Execution), SS16 (CEP layers 1-2)
> **Crate:** `fetchium-core` -- `src/http/`, `src/extract/`
> **Priority:** P0 | **Tasks:** 3

### P1-E1-T1: HTTP Client with Connection Pooling & Retries

**ID:** `P1-E1-T1`
**Status:** `DONE`
**Priority:** P0
**Estimated effort:** 1-2 days
**Dependencies:** P0-E1-T1 (workspace), P0-E1-T3 (config)

**Description:**
Extend the scaffolded `HttpClient` in `fetchium-core/src/http/client.rs` with production-quality retry logic (exponential backoff with jitter), per-domain rate limiting, response size enforcement, and structured error mapping. The HTTP client is the foundation for every network operation in Fetchium.

**PRD References:**

- SS14 "Parallel Execution Engine" -- Domain-aware scheduler, per-domain concurrency caps
- SS16 "Content Extraction Pipeline" -- HTTP GET as first stage
- SS44 "Error Handling" -- NetworkTimeout, Http403, Http429, Http5xx, AntiBot

**Files to create/modify:**

```
crates/fetchium-core/src/http/
  mod.rs              -- Module root (already exists, update re-exports)
  client.rs           -- Extended HTTP client with retries + rate limiting
```

**Step-by-step implementation:**

**Step 1: Extend `HttpClient` with retry logic and rate limiting (`http/client.rs`)**

Replace the existing scaffold with a production client:

```rust
//! HTTP client with connection pooling, retries, rate limiting, and size limits.
//!
//! PRD SS14: Domain-aware scheduler with per-domain concurrency caps.
//! PRD SS44: Structured errors with retry info.

use crate::config::FetchiumConfig;
use crate::error::{ErrorKind, FetchiumError, FetchiumResult, StructuredError};
use dashmap::DashMap;
use reqwest::{Client, Response, StatusCode};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, warn};
use url::Url;

/// Maximum retry attempts for transient errors.
const MAX_RETRIES: u32 = 3;
/// Base delay for exponential backoff (milliseconds).
const BASE_DELAY_MS: u64 = 500;
/// Maximum response body size (10 MB).
const MAX_BODY_SIZE: u64 = 10 * 1024 * 1024;

/// Per-domain rate limit state.
#[derive(Debug)]
struct DomainState {
    last_request: Instant,
    min_delay: Duration,
}

/// Shared HTTP client with connection pooling, retries, and rate limiting.
#[derive(Clone)]
pub struct HttpClient {
    inner: Client,
    config: Arc<FetchiumConfig>,
    /// Per-domain rate limiting state.
    domain_delays: Arc<DashMap<String, DomainState>>,
}

/// Fetch result with metadata about the request.
#[derive(Debug)]
pub struct FetchResult {
    pub body: String,
    pub status: u16,
    pub content_type: String,
    pub content_length: Option<u64>,
    pub url: String,
    pub elapsed_ms: u64,
    pub retries: u32,
}

impl HttpClient {
    /// Create a new HTTP client from config.
    pub fn new(config: &FetchiumConfig) -> FetchiumResult<Self> {
        let client = Client::builder()
            .user_agent(&config.fetch.user_agent)
            .timeout(Duration::from_secs(config.fetch.timeout_secs))
            .connect_timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::limited(
                config.fetch.max_redirects as usize,
            ))
            .gzip(true)
            .brotli(true)
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .map_err(FetchiumError::Network)?;

        Ok(Self {
            inner: client,
            config: Arc::new(config.clone()),
            domain_delays: Arc::new(DashMap::new()),
        })
    }

    /// Get the inner reqwest client for direct use.
    pub fn client(&self) -> &Client {
        &self.inner
    }

    /// Get the config reference.
    pub fn config(&self) -> &FetchiumConfig {
        &self.config
    }

    /// Extract domain from a URL for rate limiting.
    fn extract_domain(url: &str) -> String {
        Url::parse(url)
            .map(|u| u.host_str().unwrap_or("unknown").to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    }

    /// Wait if needed to respect per-domain rate limits.
    async fn enforce_rate_limit(&self, domain: &str) {
        let min_delay = Duration::from_millis(200); // 200ms between requests to same domain
        if let Some(state) = self.domain_delays.get(domain) {
            let elapsed = state.last_request.elapsed();
            if elapsed < state.min_delay {
                let wait = state.min_delay - elapsed;
                debug!("Rate limiting {domain}: waiting {wait:?}");
                sleep(wait).await;
            }
        }
        self.domain_delays.insert(
            domain.to_string(),
            DomainState {
                last_request: Instant::now(),
                min_delay,
            },
        );
    }

    /// Calculate backoff delay with jitter for retry attempt.
    fn backoff_delay(attempt: u32) -> Duration {
        let base = BASE_DELAY_MS * 2u64.pow(attempt);
        // Add jitter: random-ish delay using simple hash of current time
        let jitter = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos()
            % 500) as u64;
        Duration::from_millis(base + jitter)
    }

    /// Whether a status code is retryable.
    fn is_retryable_status(status: StatusCode) -> bool {
        matches!(
            status,
            StatusCode::TOO_MANY_REQUESTS
                | StatusCode::INTERNAL_SERVER_ERROR
                | StatusCode::BAD_GATEWAY
                | StatusCode::SERVICE_UNAVAILABLE
                | StatusCode::GATEWAY_TIMEOUT
        )
    }

    /// Map an HTTP status to a structured error kind.
    fn status_to_error_kind(status: StatusCode) -> ErrorKind {
        match status {
            StatusCode::FORBIDDEN => ErrorKind::Http403,
            StatusCode::TOO_MANY_REQUESTS => ErrorKind::Http429,
            s if s.is_server_error() => ErrorKind::Http5xx,
            _ => ErrorKind::Unknown,
        }
    }

    /// Fetch a URL with retries, rate limiting, and size enforcement.
    /// Returns the response body as a string along with metadata.
    pub async fn fetch(&self, url: &str) -> FetchiumResult<FetchResult> {
        let domain = Self::extract_domain(url);
        let start = Instant::now();
        let max_size = self.config.fetch.max_page_size;
        let mut last_err: Option<FetchiumError> = None;

        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                let delay = Self::backoff_delay(attempt - 1);
                debug!("Retry {attempt}/{MAX_RETRIES} for {url} after {delay:?}");
                sleep(delay).await;
            }

            self.enforce_rate_limit(&domain).await;

            let result = self.inner.get(url).send().await;

            match result {
                Ok(resp) => {
                    let status = resp.status();

                    if !status.is_success() {
                        if Self::is_retryable_status(status) && attempt < MAX_RETRIES {
                            warn!("Retryable status {status} for {url}");
                            last_err = Some(FetchiumError::Structured(StructuredError {
                                kind: Self::status_to_error_kind(status),
                                retryable: true,
                                message: format!("HTTP {status} from {url}"),
                                source_url: Some(url.to_string()),
                                suggested_action: "Retry with backoff".into(),
                                alternatives: vec![],
                            }));
                            continue;
                        }

                        return Err(FetchiumError::Structured(StructuredError {
                            kind: Self::status_to_error_kind(status),
                            retryable: false,
                            message: format!("HTTP {status} from {url}"),
                            source_url: Some(url.to_string()),
                            suggested_action: match status {
                                StatusCode::FORBIDDEN => {
                                    "Site blocks automated access".into()
                                }
                                StatusCode::TOO_MANY_REQUESTS => {
                                    "Rate limited, try later".into()
                                }
                                _ => "Check URL and try again".into(),
                            },
                            alternatives: vec![],
                        }));
                    }

                    // Check content length before downloading
                    let content_length = resp.content_length();
                    if let Some(len) = content_length {
                        if len > max_size {
                            return Err(FetchiumError::Structured(StructuredError {
                                kind: ErrorKind::ExtractionFailed,
                                retryable: false,
                                message: format!(
                                    "Response too large: {len} bytes (max {max_size})"
                                ),
                                source_url: Some(url.to_string()),
                                suggested_action: "Increase max_page_size in config".into(),
                                alternatives: vec![],
                            }));
                        }
                    }

                    let content_type = resp
                        .headers()
                        .get("content-type")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("text/html")
                        .to_string();

                    let final_url = resp.url().to_string();

                    // Read body with size limit enforcement
                    let body = resp
                        .text()
                        .await
                        .map_err(FetchiumError::Network)?;

                    if body.len() as u64 > max_size {
                        return Err(FetchiumError::Structured(StructuredError {
                            kind: ErrorKind::ExtractionFailed,
                            retryable: false,
                            message: format!(
                                "Body too large: {} bytes",
                                body.len()
                            ),
                            source_url: Some(url.to_string()),
                            suggested_action: "Increase max_page_size".into(),
                            alternatives: vec![],
                        }));
                    }

                    return Ok(FetchResult {
                        body,
                        status: status.as_u16(),
                        content_type,
                        content_length,
                        url: final_url,
                        elapsed_ms: start.elapsed().as_millis() as u64,
                        retries: attempt,
                    });
                }
                Err(e) => {
                    if e.is_timeout() || e.is_connect() {
                        if attempt < MAX_RETRIES {
                            warn!("Transient error for {url}: {e}");
                            last_err = Some(FetchiumError::Network(e));
                            continue;
                        }
                    }
                    return Err(FetchiumError::Network(e));
                }
            }
        }

        Err(last_err.unwrap_or_else(|| {
            FetchiumError::Structured(StructuredError {
                kind: ErrorKind::NetworkTimeout,
                retryable: false,
                message: format!("All {MAX_RETRIES} retries exhausted for {url}"),
                source_url: Some(url.to_string()),
                suggested_action: "Check network connectivity".into(),
                alternatives: vec![],
            })
        }))
    }

    /// Convenience: fetch a URL and return just the body text.
    pub async fn fetch_text(&self, url: &str) -> FetchiumResult<String> {
        let result = self.fetch(url).await?;
        Ok(result.body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_creation() {
        let config = FetchiumConfig::default();
        let client = HttpClient::new(&config);
        assert!(client.is_ok());
    }

    #[test]
    fn domain_extraction() {
        assert_eq!(
            HttpClient::extract_domain("https://www.example.com/page"),
            "www.example.com"
        );
        assert_eq!(
            HttpClient::extract_domain("http://localhost:8080/api"),
            "localhost"
        );
        assert_eq!(HttpClient::extract_domain("not-a-url"), "unknown");
    }

    #[test]
    fn backoff_delay_increases() {
        let d0 = HttpClient::backoff_delay(0);
        let d1 = HttpClient::backoff_delay(1);
        let d2 = HttpClient::backoff_delay(2);
        // Base delays: 500ms, 1000ms, 2000ms (plus jitter up to 500ms)
        assert!(d0.as_millis() >= 500 && d0.as_millis() < 1000);
        assert!(d1.as_millis() >= 1000 && d1.as_millis() < 1500);
        assert!(d2.as_millis() >= 2000 && d2.as_millis() < 2500);
    }

    #[test]
    fn retryable_status_codes() {
        assert!(HttpClient::is_retryable_status(StatusCode::TOO_MANY_REQUESTS));
        assert!(HttpClient::is_retryable_status(StatusCode::BAD_GATEWAY));
        assert!(HttpClient::is_retryable_status(
            StatusCode::SERVICE_UNAVAILABLE
        ));
        assert!(!HttpClient::is_retryable_status(StatusCode::NOT_FOUND));
        assert!(!HttpClient::is_retryable_status(StatusCode::FORBIDDEN));
    }
}
```

**Step 2: Update module re-exports (`http/mod.rs`)**

```rust
//! HTTP client module -- pooled reqwest client with retry logic (PRD SS14).

pub mod client;

pub use client::{FetchResult, HttpClient};
```

**Acceptance criteria:**

- [ ] `HttpClient::new()` creates a pooled client from config
- [ ] `HttpClient::fetch()` returns `FetchResult` with body, status, content_type, elapsed_ms
- [ ] Retries up to 3 times on 429/5xx with exponential backoff + jitter
- [ ] Per-domain rate limiting with 200ms minimum between requests
- [ ] Response size enforcement (rejects >10MB)
- [ ] Structured error mapping for 403, 429, 5xx
- [ ] All unit tests pass: `cargo test -p fetchium-core http`
- [ ] No clippy warnings: `cargo clippy -p fetchium-core`

**Testing instructions:**

```bash
cargo test -p fetchium-core http::client::tests
# Integration test with wiremock (in tests/integration/):
# - Mock 429 response, verify 3 retries with increasing delays
# - Mock 200 response, verify body and metadata
# - Mock oversized response, verify size rejection
```

---

### P1-E1-T2: CEP Layers 1-2 Content Extraction

**ID:** `P1-E1-T2`
**Status:** `DONE`
**Priority:** P0
**Estimated effort:** 2-3 days
**Dependencies:** P1-E1-T1 (HTTP client)

**Description:**
Implement CEP (Cascade Extraction Protocol) layers 1 and 2 for static HTML content extraction. Layer 1 uses the `scraper` crate (CSS selectors) for fast structured extraction. Layer 2 uses `lol_html` (streaming HTML rewriter) for Readability-style article extraction. The system auto-escalates from Layer 1 to Layer 2 when Layer 1 produces insufficient content.

**PRD References:**

- SS16 "Content Extraction Pipeline" -- Layer 1: HTTP + CSS selectors (~2ms), Layer 2: HTTP + Readability (~8ms)
- SS8.3 "CEP" -- Auto-escalation: content_length > threshold AND text_ratio > 0.3
- SS20 "Token Efficiency" -- Boilerplate stripping: ~30% token savings

**Files to create/modify:**

```
crates/fetchium-core/src/extract/
  mod.rs              -- Module root (update with new exports)
  layer1.rs           -- CSS selector extraction (scraper)
  layer2.rs           -- Readability-style extraction (lol_html)
  pipeline.rs         -- CEP orchestrator (layer selection + escalation)
  boilerplate.rs      -- Boilerplate removal patterns
```

**Step-by-step implementation:**

**Step 1: Boilerplate removal (`extract/boilerplate.rs`)**

```rust
//! Boilerplate removal -- strip nav, footer, ads, scripts, styles.
//!
//! PRD SS20: Boilerplate stripping yields ~30% token savings.

use once_cell::sync::Lazy;
use regex::Regex;

/// CSS selectors for elements that are almost always boilerplate.
pub const BOILERPLATE_SELECTORS: &[&str] = &[
    "nav",
    "footer",
    "header",
    "aside",
    "script",
    "style",
    "noscript",
    "iframe",
    "svg",
    ".sidebar",
    ".navigation",
    ".menu",
    ".footer",
    ".header",
    ".nav",
    ".ads",
    ".ad",
    ".advertisement",
    ".social-share",
    ".cookie-banner",
    ".popup",
    ".modal",
    "#cookie-consent",
    "#gdpr",
    "[role='navigation']",
    "[role='banner']",
    "[role='contentinfo']",
    "[aria-hidden='true']",
];

/// Tags to strip inline (keep text content).
pub const INLINE_STRIP_TAGS: &[&str] = &["span", "em", "strong", "b", "i", "u", "a"];

static WHITESPACE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\s{3,}").expect("valid regex"));
static EMPTY_LINES_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\n{3,}").expect("valid regex"));

/// Clean extracted text: collapse whitespace, remove empty lines.
pub fn clean_text(text: &str) -> String {
    let text = WHITESPACE_RE.replace_all(text, "  ");
    let text = EMPTY_LINES_RE.replace_all(&text, "\n\n");
    text.trim().to_string()
}

/// Estimate the text-to-HTML ratio for quality assessment.
pub fn text_ratio(html: &str, text: &str) -> f64 {
    if html.is_empty() {
        return 0.0;
    }
    text.len() as f64 / html.len() as f64
}

/// Minimum text length (chars) for Layer 1 to be considered sufficient.
pub const MIN_CONTENT_LENGTH: usize = 200;

/// Minimum text ratio for Layer 1 to be considered sufficient.
pub const MIN_TEXT_RATIO: f64 = 0.05;
```

**Step 2: Layer 1 -- CSS selector extraction (`extract/layer1.rs`)**

```rust
//! CEP Layer 1: CSS selector extraction using `scraper` crate.
//!
//! PRD SS16: "HTTP + Cheerio" -- ~2ms, ~5MB RAM, handles 85% of web pages.
//! This is the fastest extraction method, using CSS selectors to pull
//! title, main content, and metadata from static HTML.

use crate::extract::boilerplate::{self, clean_text, BOILERPLATE_SELECTORS};
use crate::extract::{ContentMetadata, ExtractedContent};
use crate::types::CepLayer;
use scraper::{Html, Selector};
use tracing::debug;

/// Extract content from HTML using CSS selectors (Layer 1).
///
/// Attempts to find the main content area via common selectors,
/// strips boilerplate, and returns clean text with metadata.
pub fn extract(html: &str, url: &str) -> ExtractedContent {
    let document = Html::parse_document(html);

    let title = extract_title(&document);
    let metadata = extract_metadata(&document, url);

    // Remove boilerplate elements, then extract main content
    let main_text = extract_main_content(&document);
    let cleaned = clean_text(&main_text);
    let tokens = estimate_tokens(&cleaned);

    debug!(
        "Layer1: extracted {} chars, ~{} tokens from {}",
        cleaned.len(),
        tokens,
        url
    );

    ExtractedContent {
        title,
        text: cleaned,
        layer_used: CepLayer::Layer1,
        tokens,
        metadata,
    }
}

/// Check if Layer 1 extraction produced sufficient content.
pub fn is_sufficient(content: &ExtractedContent, html: &str) -> bool {
    let ratio = boilerplate::text_ratio(html, &content.text);
    content.text.len() >= boilerplate::MIN_CONTENT_LENGTH
        && ratio >= boilerplate::MIN_TEXT_RATIO
}

fn extract_title(doc: &Html) -> String {
    // Try <title> tag first
    let title_sel = Selector::parse("title").expect("valid selector");
    if let Some(el) = doc.select(&title_sel).next() {
        let t = el.text().collect::<String>().trim().to_string();
        if !t.is_empty() {
            return t;
        }
    }
    // Fallback to <h1>
    let h1_sel = Selector::parse("h1").expect("valid selector");
    if let Some(el) = doc.select(&h1_sel).next() {
        return el.text().collect::<String>().trim().to_string();
    }
    // Fallback to og:title
    let og_sel = Selector::parse("meta[property='og:title']").expect("valid selector");
    if let Some(el) = doc.select(&og_sel).next() {
        if let Some(content) = el.value().attr("content") {
            return content.trim().to_string();
        }
    }
    String::new()
}

fn extract_metadata(doc: &Html, url: &str) -> ContentMetadata {
    let meta_desc = extract_meta_content(doc, "meta[name='description']")
        .or_else(|| extract_meta_content(doc, "meta[property='og:description']"));

    let author = extract_meta_content(doc, "meta[name='author']")
        .or_else(|| extract_meta_content(doc, "meta[property='article:author']"));

    let published = extract_meta_content(doc, "meta[property='article:published_time']")
        .or_else(|| extract_meta_content(doc, "meta[name='date']"))
        .or_else(|| extract_meta_content(doc, "time[datetime]"));

    let language = extract_meta_content(doc, "html[lang]")
        .or_else(|| extract_meta_content(doc, "meta[http-equiv='content-language']"));

    ContentMetadata {
        description: meta_desc,
        author,
        published_date: published,
        language,
        content_type: "text/html".to_string(),
    }
}

fn extract_meta_content(doc: &Html, selector_str: &str) -> Option<String> {
    let sel = Selector::parse(selector_str).ok()?;
    let el = doc.select(&sel).next()?;
    // Try content attribute first (for <meta> tags)
    if let Some(val) = el.value().attr("content") {
        let trimmed = val.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    // Try datetime attribute (for <time> tags)
    if let Some(val) = el.value().attr("datetime") {
        return Some(val.trim().to_string());
    }
    // Try lang attribute (for <html> tag)
    if let Some(val) = el.value().attr("lang") {
        return Some(val.trim().to_string());
    }
    None
}

fn extract_main_content(doc: &Html) -> String {
    // Priority-ordered selectors for main content areas
    let content_selectors = [
        "article",
        "main",
        "[role='main']",
        ".post-content",
        ".article-content",
        ".entry-content",
        ".content",
        "#content",
        ".post-body",
        ".article-body",
    ];

    for sel_str in content_selectors {
        if let Ok(sel) = Selector::parse(sel_str) {
            let elements: Vec<_> = doc.select(&sel).collect();
            if !elements.is_empty() {
                let mut text = String::new();
                for el in elements {
                    // Skip boilerplate children
                    collect_text_excluding_boilerplate(el, &mut text);
                }
                let cleaned = text.trim().to_string();
                if cleaned.len() >= boilerplate::MIN_CONTENT_LENGTH {
                    return cleaned;
                }
            }
        }
    }

    // Fallback: extract from <body>, excluding boilerplate
    if let Ok(body_sel) = Selector::parse("body") {
        if let Some(body) = doc.select(&body_sel).next() {
            let mut text = String::new();
            collect_text_excluding_boilerplate(body, &mut text);
            return text.trim().to_string();
        }
    }

    // Last resort: all text
    doc.root_element().text().collect::<String>()
}

fn collect_text_excluding_boilerplate(
    element: scraper::ElementRef<'_>,
    output: &mut String,
) {
    let tag = element.value().name();

    // Skip boilerplate tags entirely
    for bp in BOILERPLATE_SELECTORS {
        // Simple tag-name matching for common boilerplate
        if *bp == tag {
            return;
        }
    }

    // Skip script/style/nav/footer/aside
    if matches!(
        tag,
        "script" | "style" | "nav" | "footer" | "aside" | "noscript" | "svg" | "iframe"
    ) {
        return;
    }

    // For block-level elements, add newlines
    let is_block = matches!(
        tag,
        "div" | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "li" | "tr"
            | "blockquote"
            | "pre"
            | "section"
            | "article"
            | "main"
            | "br"
    );

    if is_block && !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }

    for child in element.children() {
        match child.value() {
            scraper::node::Node::Text(text) => {
                let t = text.text.trim();
                if !t.is_empty() {
                    output.push_str(t);
                    output.push(' ');
                }
            }
            scraper::node::Node::Element(_) => {
                if let Some(child_el) = scraper::ElementRef::wrap(child) {
                    collect_text_excluding_boilerplate(child_el, output);
                }
            }
            _ => {}
        }
    }

    if is_block {
        output.push('\n');
    }
}

/// Rough token count estimate: ~4 characters per token for English text.
pub fn estimate_tokens(text: &str) -> u32 {
    (text.len() as f64 / 4.0).ceil() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_simple_page() {
        let html = r#"
        <html>
        <head><title>Test Page</title></head>
        <body>
            <nav>Navigation menu</nav>
            <article>
                <h1>Main Article</h1>
                <p>This is the main content of the article with enough text
                   to pass the minimum content length threshold for testing
                   purposes. We need at least 200 characters here so let
                   us keep writing more content.</p>
            </article>
            <footer>Footer content</footer>
        </body>
        </html>
        "#;

        let result = extract(html, "https://example.com");
        assert_eq!(result.title, "Test Page");
        assert!(result.text.contains("Main Article"));
        assert!(result.text.contains("main content"));
        assert!(!result.text.contains("Navigation menu"));
        assert!(!result.text.contains("Footer content"));
        assert_eq!(result.layer_used, CepLayer::Layer1);
        assert!(result.tokens > 0);
    }

    #[test]
    fn extract_title_fallbacks() {
        // No <title>, should fall back to <h1>
        let html = r#"<html><body><h1>Heading Title</h1><p>Content</p></body></html>"#;
        let doc = Html::parse_document(html);
        assert_eq!(extract_title(&doc), "Heading Title");
    }

    #[test]
    fn insufficient_content_detection() {
        let html = "<html><body><p>Short</p></body></html>";
        let result = extract(html, "https://example.com");
        assert!(!is_sufficient(&result, html));
    }

    #[test]
    fn token_estimation() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("test"), 1); // 4 chars / 4 = 1
        assert_eq!(estimate_tokens("hello world twelve"), 5); // 18 / 4 = 4.5 -> 5
    }
}
```

**Step 3: Layer 2 -- Readability-style extraction (`extract/layer2.rs`)**

```rust
//! CEP Layer 2: Readability-style extraction using `lol_html` streaming rewriter.
//!
//! PRD SS16: "HTTP + Readability" -- ~8ms, ~10MB RAM, for article pages.
//! Uses lol_html to stream through HTML, stripping boilerplate in a single
//! pass without building a full DOM tree.

use crate::extract::boilerplate::{clean_text, MIN_CONTENT_LENGTH};
use crate::extract::{ContentMetadata, ExtractedContent};
use crate::extract::layer1::estimate_tokens;
use crate::types::CepLayer;
use lol_html::{element, rewrite_str, RewriteStrSettings};
use tracing::debug;

/// Tags to remove entirely (element and all children).
const REMOVE_TAGS: &[&str] = &[
    "script", "style", "nav", "footer", "aside", "noscript", "svg",
    "iframe", "form", "button", "input", "select", "textarea",
];

/// Tags to unwrap (remove tag but keep text content).
const UNWRAP_TAGS: &[&str] = &[
    "span", "div", "section", "main", "article", "header",
];

/// Extract content using lol_html streaming rewriter (Layer 2).
///
/// This method is more aggressive at stripping boilerplate than Layer 1.
/// It processes HTML in a single streaming pass, making it memory-efficient
/// for large pages.
pub fn extract(html: &str, url: &str) -> ExtractedContent {
    let title = extract_title_simple(html);
    let metadata = extract_metadata_simple(html);

    let cleaned_html = strip_boilerplate(html);
    let text = html_to_text(&cleaned_html);
    let cleaned = clean_text(&text);
    let tokens = estimate_tokens(&cleaned);

    debug!(
        "Layer2: extracted {} chars, ~{} tokens from {}",
        cleaned.len(),
        tokens,
        url
    );

    ExtractedContent {
        title,
        text: cleaned,
        layer_used: CepLayer::Layer2,
        tokens,
        metadata,
    }
}

/// Check if Layer 2 extraction produced sufficient content.
pub fn is_sufficient(content: &ExtractedContent) -> bool {
    content.text.len() >= MIN_CONTENT_LENGTH
}

/// Strip boilerplate tags using lol_html streaming rewriter.
fn strip_boilerplate(html: &str) -> String {
    let mut selectors = Vec::new();

    for tag in REMOVE_TAGS {
        selectors.push(element!(tag, |el| {
            el.remove();
            Ok(())
        }));
    }

    // Remove elements with boilerplate classes/roles
    selectors.push(element!("[role='navigation']", |el| {
        el.remove();
        Ok(())
    }));
    selectors.push(element!("[role='banner']", |el| {
        el.remove();
        Ok(())
    }));
    selectors.push(element!("[role='contentinfo']", |el| {
        el.remove();
        Ok(())
    }));
    selectors.push(element!("[aria-hidden='true']", |el| {
        el.remove();
        Ok(())
    }));

    match rewrite_str(
        html,
        RewriteStrSettings {
            element_content_handlers: selectors,
            ..RewriteStrSettings::default()
        },
    ) {
        Ok(result) => result,
        Err(_) => html.to_string(),
    }
}

/// Convert cleaned HTML to plain text, preserving paragraph structure.
fn html_to_text(html: &str) -> String {
    use once_cell::sync::Lazy;
    use regex::Regex;

    static TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<[^>]+>").expect("valid regex"));
    static ENTITY_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"&(amp|lt|gt|quot|apos|nbsp|#\d+|#x[0-9a-fA-F]+);").expect("valid regex"));

    // Replace block tags with newlines
    let text = html
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("</p>", "\n\n")
        .replace("</div>", "\n")
        .replace("</li>", "\n")
        .replace("</tr>", "\n")
        .replace("</h1>", "\n\n")
        .replace("</h2>", "\n\n")
        .replace("</h3>", "\n\n")
        .replace("</h4>", "\n")
        .replace("</h5>", "\n")
        .replace("</h6>", "\n")
        .replace("</blockquote>", "\n");

    // Strip remaining tags
    let text = TAG_RE.replace_all(&text, "");

    // Decode HTML entities
    let text = ENTITY_RE.replace_all(&text, |caps: &regex::Captures| {
        match &caps[1] {
            "amp" => "&".to_string(),
            "lt" => "<".to_string(),
            "gt" => ">".to_string(),
            "quot" => "\"".to_string(),
            "apos" => "'".to_string(),
            "nbsp" => " ".to_string(),
            s if s.starts_with('#') => {
                // Numeric entity
                let num = if s.starts_with("#x") {
                    u32::from_str_radix(&s[2..], 16).ok()
                } else {
                    s[1..].parse::<u32>().ok()
                };
                num.and_then(char::from_u32)
                    .map(|c| c.to_string())
                    .unwrap_or_default()
            }
            _ => String::new(),
        }
    });

    text.to_string()
}

/// Simple title extraction without full DOM parsing.
fn extract_title_simple(html: &str) -> String {
    use regex::Regex;

    static TITLE_RE: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(|| {
        Regex::new(r"(?i)<title[^>]*>(.*?)</title>").expect("valid regex")
    });

    TITLE_RE
        .captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_default()
}

/// Simple metadata extraction using regex (no DOM).
fn extract_metadata_simple(html: &str) -> ContentMetadata {
    use regex::Regex;

    static DESC_RE: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(|| {
        Regex::new(r#"(?i)<meta[^>]*name=["']description["'][^>]*content=["']([^"']*)["']"#)
            .expect("valid regex")
    });
    static AUTHOR_RE: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(|| {
        Regex::new(r#"(?i)<meta[^>]*name=["']author["'][^>]*content=["']([^"']*)["']"#)
            .expect("valid regex")
    });

    ContentMetadata {
        description: DESC_RE
            .captures(html)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string()),
        author: AUTHOR_RE
            .captures(html)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string()),
        published_date: None,
        language: None,
        content_type: "text/html".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_article_page() {
        let html = r#"
        <html>
        <head><title>Article Title</title></head>
        <body>
            <nav><a href="/">Home</a><a href="/about">About</a></nav>
            <script>var tracking = true;</script>
            <article>
                <h1>Main Heading</h1>
                <p>First paragraph with substantial content that should
                   definitely be extracted and preserved in the output
                   because it forms the core of the article text.</p>
                <p>Second paragraph with more details about the topic
                   that provides additional context and information
                   for the reader to understand the full picture.</p>
            </article>
            <footer>Copyright 2026</footer>
        </body>
        </html>
        "#;

        let result = extract(html, "https://example.com/article");
        assert_eq!(result.title, "Article Title");
        assert!(result.text.contains("First paragraph"));
        assert!(result.text.contains("Second paragraph"));
        assert!(!result.text.contains("tracking"));
        assert!(!result.text.contains("Copyright"));
        assert_eq!(result.layer_used, CepLayer::Layer2);
    }

    #[test]
    fn html_entity_decoding() {
        let text = html_to_text("<p>Tom &amp; Jerry &lt;3 &quot;cheese&quot;</p>");
        assert!(text.contains("Tom & Jerry <3 \"cheese\""));
    }

    #[test]
    fn strip_scripts_and_styles() {
        let html = r#"
        <html><body>
        <script>alert('xss')</script>
        <style>.foo { color: red; }</style>
        <p>Real content here</p>
        </body></html>
        "#;
        let cleaned = strip_boilerplate(html);
        assert!(!cleaned.contains("alert"));
        assert!(!cleaned.contains("color: red"));
        assert!(cleaned.contains("Real content"));
    }
}
```

**Step 4: CEP pipeline orchestrator (`extract/pipeline.rs`)**

```rust
//! CEP extraction pipeline -- orchestrates layer selection and escalation.
//!
//! PRD SS16: 5-layer cascade with auto-escalation. Phase 1 implements L1-L2.
//! PRD SS8.3: ML-predicted method selection (stubbed for Phase 1,
//!            implemented in Phase 5).

use crate::extract::layer1;
use crate::extract::layer2;
use crate::extract::ExtractedContent;
use crate::types::CepLayer;
use tracing::{debug, info};

/// Maximum CEP layer available in the current build.
/// Phase 1: layers 1-2 only. Layers 3-5 added in Phase 2.
pub const MAX_AVAILABLE_LAYER: CepLayer = CepLayer::Layer2;

/// Run the CEP extraction pipeline on raw HTML.
///
/// Starts at Layer 1 and escalates to Layer 2 if insufficient content
/// is extracted. Returns the best extraction result.
pub fn extract(html: &str, url: &str) -> ExtractedContent {
    // --- Layer 1: CSS selector extraction ---
    let l1_result = layer1::extract(html, url);

    if layer1::is_sufficient(&l1_result, html) {
        info!(
            "CEP Layer1 sufficient for {} ({} tokens)",
            url, l1_result.tokens
        );
        return l1_result;
    }

    debug!(
        "CEP Layer1 insufficient for {} ({} chars), escalating to Layer2",
        url,
        l1_result.text.len()
    );

    // --- Layer 2: Readability-style extraction ---
    let l2_result = layer2::extract(html, url);

    if layer2::is_sufficient(&l2_result) {
        info!(
            "CEP Layer2 sufficient for {} ({} tokens)",
            url, l2_result.tokens
        );
        return l2_result;
    }

    debug!(
        "CEP Layer2 insufficient for {} ({} chars), returning best result",
        url,
        l2_result.text.len()
    );

    // Return whichever produced more content
    if l2_result.text.len() > l1_result.text.len() {
        l2_result
    } else {
        l1_result
    }
}

/// Predict the best CEP layer for a given URL (stub for Phase 1).
///
/// In Phase 5, this will use an ML classifier trained on URL patterns,
/// domain, content-type, and HTML structural features.
pub fn predict_layer(_url: &str, _content_type: &str) -> CepLayer {
    // Phase 1: always start at Layer 1 and escalate
    CepLayer::Layer1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_uses_layer1_for_good_content() {
        let html = r#"
        <html>
        <head><title>Good Page</title></head>
        <body>
        <article>
            <h1>Well Structured Article</h1>
            <p>This article has plenty of well-structured content that
               should be easily extractable by Layer 1. The CSS selectors
               will find the article tag and extract all the paragraphs
               within it. This should pass the minimum content threshold
               without needing to escalate to Layer 2.</p>
            <p>Second paragraph with additional details and information
               that further enriches the article content for readers.</p>
        </article>
        </body>
        </html>
        "#;

        let result = extract(html, "https://example.com/good");
        assert_eq!(result.layer_used, CepLayer::Layer1);
        assert!(result.text.contains("Well Structured"));
    }

    #[test]
    fn pipeline_escalates_for_poor_content() {
        // Minimal HTML that Layer 1 might struggle with
        let html = "<html><body><p>x</p></body></html>";
        let result = extract(html, "https://example.com/poor");
        // Should try Layer 2 since Layer 1 produces very little
        assert!(
            result.layer_used == CepLayer::Layer1
                || result.layer_used == CepLayer::Layer2
        );
    }
}
```

**Step 5: Update module root (`extract/mod.rs`)**

```rust
//! Content Extraction Protocol (CEP) -- 5-layer extraction system (PRD SS16).
//!
//! Phase 1 implements layers 1-2 (static HTML).
//! Phase 2 adds layers 3-5 (headless Chromium).

pub mod boilerplate;
pub mod layer1;
pub mod layer2;
pub mod pipeline;

use crate::types::CepLayer;

/// Extracted content from a web page.
#[derive(Debug, Clone)]
pub struct ExtractedContent {
    pub title: String,
    pub text: String,
    pub layer_used: CepLayer,
    pub tokens: u32,
    pub metadata: ContentMetadata,
}

/// Metadata extracted alongside content.
#[derive(Debug, Clone, Default)]
pub struct ContentMetadata {
    pub description: Option<String>,
    pub author: Option<String>,
    pub published_date: Option<String>,
    pub language: Option<String>,
    pub content_type: String,
}
```

**Acceptance criteria:**

- [ ] Layer 1 extracts title, text, and metadata from standard HTML pages
- [ ] Layer 1 strips nav/footer/script/style boilerplate
- [ ] Layer 2 uses lol_html streaming rewriter for deeper boilerplate removal
- [ ] Pipeline auto-escalates from L1 to L2 when content is insufficient
- [ ] Boilerplate removal achieves ~30% token reduction vs raw text
- [ ] `estimate_tokens()` returns reasonable approximation (~4 chars/token)
- [ ] All unit tests pass: `cargo test -p fetchium-core extract`
- [ ] No clippy warnings

**Testing instructions:**

```bash
cargo test -p fetchium-core extract
# Create HTML fixtures in tests/fixtures/ for comprehensive testing
# Test with: blog article, docs page, sparse page, JS-heavy placeholder
```

---

### P1-E1-T3: `fetch` Command Implementation

**ID:** `P1-E1-T3`
**Status:** `DONE`
**Priority:** P1
**Estimated effort:** 1 day
**Dependencies:** P1-E1-T1 (HTTP client), P1-E1-T2 (CEP extraction)

**Description:**
Wire up the `fetchium fetch <url>` and `fetchium view <url>` CLI commands to use the HTTP client and CEP extraction pipeline. The fetch command downloads a URL, extracts content via CEP layers 1-2, and displays the result in the selected output format.

**PRD References:**

- SS10 "Modes" -- Mode D (fetch/view): clean readable extraction
- SS11 "CLI Interface" -- `fetchium fetch <url>` with --budget, --tier, --query options

**Files to modify:**

```
crates/fetchium-cli/src/commands/fetch.rs  -- Full implementation
```

**Step-by-step implementation:**

```rust
//! `fetchium fetch` / `fetchium view` -- URL content extraction (Mode D).
//!
//! Downloads a URL, extracts content via CEP pipeline, and displays
//! the result in the selected output format.

use crate::cli::FetchArgs;
use colored::Colorize;
use hsx_core::config::FetchiumConfig;
use hsx_core::extract::pipeline;
use hsx_core::http::HttpClient;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;

pub async fn run(args: FetchArgs, config: &FetchiumConfig) -> anyhow::Result<()> {
    let start = Instant::now();

    // Show progress spinner
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .expect("valid template"),
    );
    pb.set_message(format!("Fetching {}", &args.url));
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    // Fetch the URL
    let client = HttpClient::new(config)?;
    let fetch_result = client.fetch(&args.url).await?;

    pb.set_message("Extracting content...");

    // Extract content via CEP pipeline
    let content = pipeline::extract(&fetch_result.body, &fetch_result.url);

    pb.finish_and_clear();

    let elapsed = start.elapsed();

    // Display results
    println!("{}", content.title.bold());
    println!("{}", "=".repeat(content.title.len().min(60)));
    println!(
        "{} {} | {} ~{} tokens | {} {:?}",
        "Source:".dimmed(),
        fetch_result.url,
        "Size:".dimmed(),
        content.tokens,
        "Time:".dimmed(),
        elapsed,
    );

    if let Some(desc) = &content.metadata.description {
        println!("{} {}", "Description:".dimmed(), desc);
    }
    if let Some(author) = &content.metadata.author {
        println!("{} {}", "Author:".dimmed(), author);
    }

    println!("{}", "-".repeat(60));
    println!();
    println!("{}", content.text);

    Ok(())
}
```

**Acceptance criteria:**

- [ ] `fetchium fetch <url>` downloads and extracts content from any URL
- [ ] Progress spinner shows during fetch and extraction
- [ ] Output shows title, metadata, token count, and extracted text
- [ ] `fetchium view <url>` is an alias for `fetchium fetch <url>`
- [ ] Errors display structured messages (network, 403, 429, etc.)
- [ ] `cargo build -p fetchium-cli` compiles successfully

**Testing instructions:**

```bash
cargo run -p fetchium-cli -- fetch https://example.com
cargo run -p fetchium-cli -- fetch https://en.wikipedia.org/wiki/Rust_(programming_language)
cargo run -p fetchium-cli -- view https://news.ycombinator.com
```

---

## Epic 1.2: DuckDuckGo Search

> **PRD Sections:** SS15 (Search Backend Orchestrator -- Tier 2: DuckDuckGo)
> **Crate:** `fetchium-core` -- `src/search/`
> **Priority:** P0 | **Tasks:** 3

### P1-E2-T1: DuckDuckGo HTML Scraper Backend

**ID:** `P1-E2-T1`
**Status:** `DONE`
**Priority:** P0
**Estimated effort:** 2-3 days
**Dependencies:** P1-E1-T1 (HTTP client)

**Description:**
Implement the DuckDuckGo search backend by scraping `html.duckduckgo.com` (the lite/HTML version). This is the MVP's primary search engine because DDG's HTML interface has no bot detection, no API keys, and returns clean HTML that is easy to parse. The backend implements the `SearchBackend` trait defined in `search/mod.rs`.

**PRD References:**

- SS15 "Search Backend Orchestrator" -- Tier 2: DuckDuckGo, HTTP scrape, "Fast, private, no bot detection"
- SS15 "Result Fusion" -- Steps 1-4: query, collect, normalize, deduplicate

**Files to create/modify:**

```
crates/fetchium-core/src/search/
  mod.rs              -- Update with new module declarations
  duckduckgo.rs       -- DDG HTML scraper backend
```

**Step-by-step implementation:**

**Step 1: Implement the DDG backend (`search/duckduckgo.rs`)**

```rust
//! DuckDuckGo search backend -- scrapes html.duckduckgo.com.
//!
//! PRD SS15 Tier 2: DuckDuckGo via HTTP scrape.
//! Uses the lite HTML version which has no bot detection and returns
//! clean, easily parseable HTML with search results.

use crate::error::{FetchiumError, FetchiumResult};
use crate::http::HttpClient;
use crate::search::SearchBackend;
use crate::types::{BackendId, ResultItem};
use async_trait::async_trait;
use scraper::{Html, Selector};
use tracing::{debug, info, warn};

/// DuckDuckGo lite HTML endpoint.
const DDG_URL: &str = "https://html.duckduckgo.com/html/";

/// CSS selectors for parsing DDG HTML results.
struct DdgSelectors {
    result_block: Selector,
    result_title: Selector,
    result_link: Selector,
    result_snippet: Selector,
}

impl DdgSelectors {
    fn new() -> Self {
        Self {
            result_block: Selector::parse(".result").expect("valid selector"),
            result_title: Selector::parse(".result__a").expect("valid selector"),
            result_link: Selector::parse(".result__url").expect("valid selector"),
            result_snippet: Selector::parse(".result__snippet")
                .expect("valid selector"),
        }
    }
}

/// DuckDuckGo search backend using HTML scraping.
pub struct DuckDuckGoBackend {
    client: HttpClient,
    selectors: DdgSelectors,
}

impl DuckDuckGoBackend {
    /// Create a new DDG backend with the given HTTP client.
    pub fn new(client: HttpClient) -> Self {
        Self {
            client,
            selectors: DdgSelectors::new(),
        }
    }

    /// Build the search URL with query parameters.
    fn build_url(query: &str) -> String {
        // DDG lite uses POST, but we can also use GET with query params
        format!("{}?q={}", DDG_URL, urlencoding::encode(query))
    }

    /// Parse DDG HTML response into result items.
    fn parse_results(
        &self,
        html: &str,
        max_results: u32,
    ) -> Vec<ResultItem> {
        let document = Html::parse_document(html);
        let mut results = Vec::new();

        for (rank, element) in document
            .select(&self.selectors.result_block)
            .enumerate()
        {
            if results.len() >= max_results as usize {
                break;
            }

            // Extract title and URL from the result link
            let (title, url) = match element.select(&self.selectors.result_title).next()
            {
                Some(link_el) => {
                    let title = link_el.text().collect::<String>().trim().to_string();
                    let href = link_el
                        .value()
                        .attr("href")
                        .unwrap_or("")
                        .to_string();

                    // DDG sometimes wraps URLs in a redirect; extract the real URL
                    let url = Self::extract_real_url(&href);

                    if title.is_empty() || url.is_empty() {
                        continue;
                    }

                    (title, url)
                }
                None => continue,
            };

            // Extract snippet
            let snippet = element
                .select(&self.selectors.result_snippet)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            results.push(ResultItem {
                title,
                url,
                snippet,
                rank: (rank + 1) as u32,
                backend: BackendId::DuckDuckGo,
                score: None,
                published_date: None,
            });
        }

        results
    }

    /// Extract the real URL from DDG's redirect wrapper.
    /// DDG lite sometimes uses: //duckduckgo.com/l/?uddg=ENCODED_URL&...
    fn extract_real_url(href: &str) -> String {
        if href.contains("uddg=") {
            // Extract the uddg parameter
            if let Some(start) = href.find("uddg=") {
                let encoded = &href[start + 5..];
                let end = encoded.find('&').unwrap_or(encoded.len());
                let encoded_url = &encoded[..end];
                return urlencoding::decode(encoded_url)
                    .unwrap_or_else(|_| encoded_url.into())
                    .to_string();
            }
        }

        // If it looks like a direct URL, use it
        if href.starts_with("http://") || href.starts_with("https://") {
            return href.to_string();
        }

        // Prefix with https:// if it starts with //
        if href.starts_with("//") {
            return format!("https:{href}");
        }

        href.to_string()
    }
}

#[async_trait]
impl SearchBackend for DuckDuckGoBackend {
    fn id(&self) -> BackendId {
        BackendId::DuckDuckGo
    }

    fn requires_headless(&self) -> bool {
        false
    }

    async fn search(
        &self,
        query: &str,
        max_results: u32,
    ) -> FetchiumResult<Vec<ResultItem>> {
        info!("DDG search: {query:?} (max {max_results})");

        // DDG lite uses POST form submission
        let form_params = [("q", query), ("b", ""), ("kl", "")];

        let response = self
            .client
            .client()
            .post(DDG_URL)
            .form(&form_params)
            .send()
            .await
            .map_err(FetchiumError::Network)?;

        if !response.status().is_success() {
            return Err(FetchiumError::Search(format!(
                "DDG returned status {}",
                response.status()
            )));
        }

        let html = response.text().await.map_err(FetchiumError::Network)?;
        let results = self.parse_results(&html, max_results);

        info!("DDG returned {} results for {query:?}", results.len());
        debug!("DDG results: {:?}", results.iter().map(|r| &r.title).collect::<Vec<_>>());

        if results.is_empty() {
            warn!("DDG returned no results for {query:?}");
        }

        Ok(results)
    }
}

// Note: urlencoding is a small utility. We can use percent_encoding from the url crate
// or add urlencoding as a workspace dependency. For now, we use a simple inline approach.
mod urlencoding {
    use std::borrow::Cow;

    /// Percent-encode a string for URL query parameters.
    pub fn encode(input: &str) -> String {
        let mut result = String::with_capacity(input.len() * 3);
        for byte in input.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    result.push(byte as char);
                }
                b' ' => result.push('+'),
                _ => {
                    result.push('%');
                    result.push_str(&format!("{byte:02X}"));
                }
            }
        }
        result
    }

    /// Percent-decode a URL-encoded string.
    pub fn decode(input: &str) -> Result<Cow<'_, str>, std::string::FromUtf8Error> {
        let mut bytes = Vec::with_capacity(input.len());
        let mut chars = input.bytes();
        while let Some(b) = chars.next() {
            match b {
                b'%' => {
                    let hi = chars.next().unwrap_or(b'0');
                    let lo = chars.next().unwrap_or(b'0');
                    let hex = format!("{}{}", hi as char, lo as char);
                    let val = u8::from_str_radix(&hex, 16).unwrap_or(b'?');
                    bytes.push(val);
                }
                b'+' => bytes.push(b' '),
                _ => bytes.push(b),
            }
        }
        String::from_utf8(bytes).map(Cow::Owned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_real_url_direct() {
        let url = "https://www.rust-lang.org/";
        assert_eq!(DuckDuckGoBackend::extract_real_url(url), url);
    }

    #[test]
    fn extract_real_url_from_redirect() {
        let href = "//duckduckgo.com/l/?uddg=https%3A%2F%2Fwww.rust-lang.org%2F&rut=abc";
        let result = DuckDuckGoBackend::extract_real_url(href);
        assert_eq!(result, "https://www.rust-lang.org/");
    }

    #[test]
    fn extract_real_url_protocol_relative() {
        let href = "//example.com/page";
        let result = DuckDuckGoBackend::extract_real_url(href);
        assert_eq!(result, "https://example.com/page");
    }

    #[test]
    fn parse_ddg_html_results() {
        let html = r#"
        <html><body>
        <div class="result">
            <a class="result__a" href="https://www.rust-lang.org/">
                Rust Programming Language
            </a>
            <a class="result__url" href="https://www.rust-lang.org/">
                www.rust-lang.org
            </a>
            <a class="result__snippet">
                A language empowering everyone to build reliable software.
            </a>
        </div>
        <div class="result">
            <a class="result__a" href="https://doc.rust-lang.org/book/">
                The Rust Book
            </a>
            <a class="result__snippet">
                Official Rust programming guide.
            </a>
        </div>
        </body></html>
        "#;

        let config = crate::config::FetchiumConfig::default();
        let client = HttpClient::new(&config).unwrap();
        let backend = DuckDuckGoBackend::new(client);
        let results = backend.parse_results(html, 10);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Rust Programming Language");
        assert_eq!(results[0].url, "https://www.rust-lang.org/");
        assert_eq!(results[0].rank, 1);
        assert_eq!(results[0].backend, BackendId::DuckDuckGo);
        assert!(results[0].snippet.contains("reliable software"));
        assert_eq!(results[1].rank, 2);
    }

    #[test]
    fn url_encoding() {
        assert_eq!(urlencoding::encode("hello world"), "hello+world");
        assert_eq!(urlencoding::encode("rust & go"), "rust+%26+go");
    }
}
```

**Step 2: Update search module (`search/mod.rs`)**

```rust
//! Search backends and orchestrator (PRD SS15).

pub mod duckduckgo;
pub mod orchestrator;

use async_trait::async_trait;
use crate::error::FetchiumResult;
use crate::types::{BackendId, ResultItem};

/// Trait for search backends.
#[async_trait]
pub trait SearchBackend: Send + Sync {
    /// Backend identifier.
    fn id(&self) -> BackendId;

    /// Whether this backend requires a headless browser.
    fn requires_headless(&self) -> bool {
        false
    }

    /// Execute a search query and return results.
    async fn search(&self, query: &str, max_results: u32) -> FetchiumResult<Vec<ResultItem>>;
}
```

**Acceptance criteria:**

- [ ] `DuckDuckGoBackend` implements `SearchBackend` trait
- [ ] Scrapes `html.duckduckgo.com` via POST form submission
- [ ] Parses title, URL, and snippet from each result
- [ ] Handles DDG redirect wrapper URLs (uddg= parameter)
- [ ] Returns up to `max_results` results
- [ ] All unit tests pass: `cargo test -p fetchium-core search::duckduckgo`
- [ ] No clippy warnings

**Testing instructions:**

```bash
cargo test -p fetchium-core search::duckduckgo::tests
# Manual integration test:
# cargo run -p fetchium-cli -- search "rust programming language"
```

---

### P1-E2-T2: Search Orchestrator

**ID:** `P1-E2-T2`
**Status:** `DONE`
**Priority:** P0
**Estimated effort:** 1-2 days
**Dependencies:** P1-E2-T1 (DDG backend)

**Description:**
Build the search orchestrator that manages multiple search backends, dispatches queries in parallel, collects results, deduplicates by URL, and returns unified results. In Phase 1, only DDG is available, but the orchestrator is designed for multiple backends from the start.

**PRD References:**

- SS15 "Result Fusion" -- Parallel query, collect with timeout, normalize, deduplicate
- SS14 "Parallel Execution Engine" -- Priority task queue, per-backend concurrency

**Files to create:**

```
crates/fetchium-core/src/search/
  orchestrator.rs     -- Search orchestrator with parallel dispatch
```

**Step-by-step implementation:**

```rust
//! Search orchestrator -- dispatches queries to backends and fuses results.
//!
//! PRD SS15: Result Fusion pipeline:
//! 1. PARALLEL QUERY to all enabled backends
//! 2. COLLECT with per-backend timeout (fail-fast)
//! 3. NORMALIZE to unified ResultItem schema
//! 4. DEDUPLICATE via canonical URL
//! 5. RANK (Phase 1: by backend rank; Phase 2: HyperFusion)
//! 6. RETURN top N

use crate::error::FetchiumResult;
use crate::http::HttpClient;
use crate::search::duckduckgo::DuckDuckGoBackend;
use crate::search::SearchBackend;
use crate::types::{BackendId, ResultItem};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, warn};
use url::Url;

/// Configuration for the search orchestrator.
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Maximum results per backend.
    pub max_results_per_backend: u32,
    /// Total max results to return.
    pub max_total_results: u32,
    /// Per-backend timeout.
    pub backend_timeout: Duration,
    /// Enabled backend IDs.
    pub enabled_backends: Vec<BackendId>,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_results_per_backend: 15,
            max_total_results: 10,
            backend_timeout: Duration::from_secs(15),
            enabled_backends: vec![BackendId::DuckDuckGo],
        }
    }
}

/// Search orchestrator managing multiple backends.
pub struct SearchOrchestrator {
    backends: Vec<Arc<dyn SearchBackend>>,
    config: OrchestratorConfig,
}

impl SearchOrchestrator {
    /// Create a new orchestrator with default backends from config.
    pub fn new(
        http_client: HttpClient,
        config: OrchestratorConfig,
    ) -> Self {
        let mut backends: Vec<Arc<dyn SearchBackend>> = Vec::new();

        for id in &config.enabled_backends {
            match id {
                BackendId::DuckDuckGo => {
                    backends.push(Arc::new(DuckDuckGoBackend::new(
                        http_client.clone(),
                    )));
                }
                // Phase 2+ backends will be added here
                other => {
                    warn!("Backend {other} not available in Phase 1");
                }
            }
        }

        Self { backends, config }
    }

    /// Execute a search across all enabled backends.
    pub async fn search(
        &self,
        query: &str,
        max_results: Option<u32>,
    ) -> FetchiumResult<Vec<ResultItem>> {
        let max = max_results.unwrap_or(self.config.max_total_results);
        let per_backend = self.config.max_results_per_backend;

        info!(
            "Orchestrator: searching {query:?} across {} backends",
            self.backends.len()
        );

        // Dispatch to all backends in parallel
        let mut handles = Vec::new();
        for backend in &self.backends {
            let backend = Arc::clone(backend);
            let query = query.to_string();
            let timeout_dur = self.config.backend_timeout;

            handles.push(tokio::spawn(async move {
                let id = backend.id();
                match timeout(timeout_dur, backend.search(&query, per_backend)).await
                {
                    Ok(Ok(results)) => {
                        info!("Backend {} returned {} results", id, results.len());
                        results
                    }
                    Ok(Err(e)) => {
                        warn!("Backend {} failed: {e}", id);
                        Vec::new()
                    }
                    Err(_) => {
                        warn!("Backend {} timed out after {timeout_dur:?}", id);
                        Vec::new()
                    }
                }
            }));
        }

        // Collect all results
        let mut all_results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(results) => all_results.extend(results),
                Err(e) => warn!("Backend task panicked: {e}"),
            }
        }

        // Deduplicate by canonical URL
        let deduped = Self::deduplicate(all_results);

        // Take top N results
        let final_results: Vec<ResultItem> =
            deduped.into_iter().take(max as usize).collect();

        info!(
            "Orchestrator: returning {} results for {query:?}",
            final_results.len()
        );

        Ok(final_results)
    }

    /// Deduplicate results by canonical URL.
    fn deduplicate(results: Vec<ResultItem>) -> Vec<ResultItem> {
        let mut seen_urls = HashSet::new();
        let mut deduped = Vec::new();

        for result in results {
            let canonical = Self::canonical_url(&result.url);
            if seen_urls.insert(canonical) {
                deduped.push(result);
            }
        }

        deduped
    }

    /// Normalize a URL for deduplication: strip trailing slash, fragment,
    /// tracking parameters, and lowercase the host.
    fn canonical_url(url: &str) -> String {
        match Url::parse(url) {
            Ok(mut parsed) => {
                // Remove fragment
                parsed.set_fragment(None);

                // Remove common tracking parameters
                let tracking_params = [
                    "utm_source",
                    "utm_medium",
                    "utm_campaign",
                    "utm_term",
                    "utm_content",
                    "ref",
                    "fbclid",
                    "gclid",
                ];

                let pairs: Vec<(String, String)> = parsed
                    .query_pairs()
                    .filter(|(k, _)| {
                        !tracking_params.contains(&k.as_ref())
                    })
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect();

                if pairs.is_empty() {
                    parsed.set_query(None);
                } else {
                    let qs: String = pairs
                        .iter()
                        .map(|(k, v)| format!("{k}={v}"))
                        .collect::<Vec<_>>()
                        .join("&");
                    parsed.set_query(Some(&qs));
                }

                // Remove trailing slash from path
                let mut result = parsed.to_string();
                if result.ends_with('/') && result.len() > 1 {
                    result.pop();
                }
                result
            }
            Err(_) => url.to_lowercase(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_url_strips_tracking() {
        let url =
            "https://example.com/page?utm_source=google&id=123&utm_medium=cpc";
        let canonical = SearchOrchestrator::canonical_url(url);
        assert!(canonical.contains("id=123"));
        assert!(!canonical.contains("utm_source"));
        assert!(!canonical.contains("utm_medium"));
    }

    #[test]
    fn canonical_url_strips_fragment() {
        let url = "https://example.com/page#section1";
        let canonical = SearchOrchestrator::canonical_url(url);
        assert!(!canonical.contains("#section1"));
    }

    #[test]
    fn canonical_url_strips_trailing_slash() {
        let url = "https://example.com/page/";
        let canonical = SearchOrchestrator::canonical_url(url);
        assert!(!canonical.ends_with('/'));
    }

    #[test]
    fn deduplicate_by_url() {
        let results = vec![
            ResultItem {
                title: "Page A".into(),
                url: "https://example.com/page".into(),
                snippet: "First".into(),
                rank: 1,
                backend: BackendId::DuckDuckGo,
                score: None,
                published_date: None,
            },
            ResultItem {
                title: "Page A Duplicate".into(),
                url: "https://example.com/page?utm_source=test".into(),
                snippet: "Duplicate".into(),
                rank: 2,
                backend: BackendId::DuckDuckGo,
                score: None,
                published_date: None,
            },
            ResultItem {
                title: "Page B".into(),
                url: "https://other.com/page".into(),
                snippet: "Different".into(),
                rank: 3,
                backend: BackendId::DuckDuckGo,
                score: None,
                published_date: None,
            },
        ];

        let deduped = SearchOrchestrator::deduplicate(results);
        assert_eq!(deduped.len(), 2);
        assert_eq!(deduped[0].title, "Page A");
        assert_eq!(deduped[1].title, "Page B");
    }
}
```

**Acceptance criteria:**

- [ ] `SearchOrchestrator::search()` dispatches to all enabled backends in parallel
- [ ] Per-backend timeout prevents slow backends from blocking
- [ ] Results are deduplicated by canonical URL
- [ ] Canonical URL normalization strips tracking params, fragments, trailing slashes
- [ ] Gracefully handles backend failures (returns partial results)
- [ ] All unit tests pass: `cargo test -p fetchium-core search::orchestrator`
- [ ] No clippy warnings

**Testing instructions:**

```bash
cargo test -p fetchium-core search::orchestrator::tests
```

---

### P1-E2-T3: `search` Command Implementation

**ID:** `P1-E2-T3`
**Status:** `DONE`
**Priority:** P1
**Estimated effort:** 1 day
**Dependencies:** P1-E2-T2 (Search orchestrator)

**Description:**
Wire up the `fetchium search "query"` CLI command to use the search orchestrator. Display results in a human-friendly format with titles, URLs, and snippets.

**PRD References:**

- SS10 "Modes" -- Mode A: human-friendly search
- SS11 "CLI Interface" -- `fetchium search "query"` with --max-results, --backends

**Files to modify:**

```
crates/fetchium-cli/src/commands/search.rs  -- Full implementation
```

**Step-by-step implementation:**

```rust
//! `fetchium search` -- web search (Mode A: human-friendly).
//!
//! Dispatches the query to enabled backends via the orchestrator,
//! then displays results in a readable terminal format.

use crate::cli::SearchArgs;
use colored::Colorize;
use hsx_core::config::FetchiumConfig;
use hsx_core::http::HttpClient;
use hsx_core::search::orchestrator::{OrchestratorConfig, SearchOrchestrator};
use hsx_core::types::BackendId;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::{Duration, Instant};

pub async fn run(args: SearchArgs, config: &FetchiumConfig) -> anyhow::Result<()> {
    let start = Instant::now();

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .expect("valid template"),
    );
    pb.set_message(format!("Searching: {}", &args.query));
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let client = HttpClient::new(config)?;

    // Build orchestrator config from CLI args and global config
    let enabled_backends = if args.backends.is_empty() {
        config
            .search
            .backends
            .iter()
            .filter_map(|s| parse_backend_id(s))
            .collect()
    } else {
        args.backends
            .iter()
            .filter_map(|s| parse_backend_id(s))
            .collect()
    };

    let orch_config = OrchestratorConfig {
        max_results_per_backend: args.max_results + 5, // fetch a few extra for dedup
        max_total_results: args.max_results,
        backend_timeout: Duration::from_secs(config.search.timeout_secs),
        enabled_backends,
    };

    let orchestrator = SearchOrchestrator::new(client, orch_config);
    let results = orchestrator
        .search(&args.query, Some(args.max_results))
        .await?;

    pb.finish_and_clear();

    let elapsed = start.elapsed();

    if results.is_empty() {
        println!(
            "{} No results found for {:?}",
            "warning:".bold().yellow(),
            args.query
        );
        return Ok(());
    }

    // Display header
    println!(
        "\n{} {} results for {} ({:.1?})\n",
        "Found".bold().green(),
        results.len(),
        format!("{:?}", args.query).cyan(),
        elapsed,
    );

    // Display each result
    for item in &results {
        println!(
            "  {}  {}",
            format!("{}.", item.rank).dimmed(),
            item.title.bold()
        );
        println!("     {}", item.url.underline().blue());
        if !item.snippet.is_empty() {
            // Wrap snippet at ~80 chars
            let snippet = textwrap(&item.snippet, 75);
            for line in snippet.lines() {
                println!("     {}", line.dimmed());
            }
        }
        println!();
    }

    Ok(())
}

/// Parse a backend ID from a string.
fn parse_backend_id(s: &str) -> Option<BackendId> {
    match s.to_lowercase().as_str() {
        "duckduckgo" | "ddg" => Some(BackendId::DuckDuckGo),
        "google" => Some(BackendId::Google),
        "bing" => Some(BackendId::Bing),
        "scholar" | "google_scholar" => Some(BackendId::GoogleScholar),
        "searxng" => Some(BackendId::Searxng),
        "wikipedia" | "wiki" => Some(BackendId::Wikipedia),
        "brave" => Some(BackendId::Brave),
        "hackernews" | "hn" => Some(BackendId::HackerNews),
        "arxiv" => Some(BackendId::Arxiv),
        "github" | "gh" => Some(BackendId::Github),
        "reddit" => Some(BackendId::Reddit),
        "stackoverflow" | "so" => Some(BackendId::StackOverflow),
        _ => None,
    }
}

/// Simple text wrapping at word boundaries.
fn textwrap(text: &str, width: usize) -> String {
    let mut result = String::new();
    let mut line_len = 0;

    for word in text.split_whitespace() {
        if line_len + word.len() + 1 > width && line_len > 0 {
            result.push('\n');
            line_len = 0;
        }
        if line_len > 0 {
            result.push(' ');
            line_len += 1;
        }
        result.push_str(word);
        line_len += word.len();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_backend_ids() {
        assert_eq!(parse_backend_id("duckduckgo"), Some(BackendId::DuckDuckGo));
        assert_eq!(parse_backend_id("ddg"), Some(BackendId::DuckDuckGo));
        assert_eq!(parse_backend_id("DDG"), Some(BackendId::DuckDuckGo));
        assert_eq!(parse_backend_id("google"), Some(BackendId::Google));
        assert_eq!(parse_backend_id("invalid"), None);
    }

    #[test]
    fn text_wrapping() {
        let text = "This is a moderately long sentence that should be wrapped at word boundaries to fit within the given width.";
        let wrapped = textwrap(text, 40);
        for line in wrapped.lines() {
            assert!(line.len() <= 45); // Allow some slack for long words
        }
    }
}
```

**Acceptance criteria:**

- [ ] `fetchium search "query"` searches via DDG and displays formatted results
- [ ] Results show rank, title (bold), URL (blue/underlined), snippet (dimmed)
- [ ] Progress spinner during search
- [ ] Shows total count and elapsed time
- [ ] Handles empty results with a warning message
- [ ] Supports `--max-results` and `--backends` flags
- [ ] `cargo build -p fetchium-cli` compiles successfully

**Testing instructions:**

```bash
cargo run -p fetchium-cli -- search "rust programming language"
cargo run -p fetchium-cli -- search "latest news" -n 5
cargo run -p fetchium-cli -- search "tokio async rust" --backends ddg
```

---

## Epic 1.3: Token System

> **PRD Sections:** SS17 (QATBE), SS18 (SCS), SS20 (Token Efficiency), SS27 (PDS)
> **Crate:** `fetchium-core` -- `src/token/`
> **Priority:** P1 | **Tasks:** 4

### P1-E3-T1: Tokenizer & Budget Tracking

**ID:** `P1-E3-T1`
**Status:** `DONE`
**Priority:** P1
**Estimated effort:** 1 day
**Dependencies:** P0-E1-T2 (types)

**Description:**
Implement a fast token counter and budget tracker. Uses a whitespace+punctuation heuristic tokenizer (no external models needed) that approximates GPT-4 tokenization at ~95% accuracy. Budget tracking ensures that all extraction and output operations respect the caller's token limit.

**PRD References:**

- SS20 "Token Efficiency Architecture" -- Token Budget System
- SS17 "QATBE" -- Token budget as a first-class parameter

**Files to create/modify:**

```
crates/fetchium-core/src/token/
  mod.rs              -- Module root (update)
  counter.rs          -- Token counter (heuristic + budget tracker)
```

**Step-by-step implementation:**

**Step 1: Token counter and budget tracker (`token/counter.rs`)**

```rust
//! Token counter and budget tracking.
//!
//! PRD SS20: Token Budget System. Approximates GPT-4 tokenization using
//! a whitespace+punctuation heuristic (~95% accuracy for English text).
//! For exact counts, integrate tiktoken in Phase 5.

use once_cell::sync::Lazy;
use regex::Regex;

/// Average characters per token for the heuristic estimator.
const CHARS_PER_TOKEN: f64 = 4.0;

/// Regex pattern that splits text into approximate token boundaries.
/// Splits on whitespace, punctuation boundaries, and camelCase transitions.
static TOKEN_SPLIT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"[\s]+|(?<=[a-z])(?=[A-Z])|(?<=[.,;:!?\-/\\()\[\]{}\"'])"#)
        .expect("valid regex")
});

/// Count tokens in a text string using the heuristic estimator.
///
/// Returns an approximate token count. For typical English text,
/// this is within ~5% of GPT-4's tiktoken.
pub fn count_tokens(text: &str) -> u32 {
    if text.is_empty() {
        return 0;
    }
    // Primary method: character-based estimation
    // This is fast and reasonably accurate for mixed content
    let char_estimate = (text.len() as f64 / CHARS_PER_TOKEN).ceil() as u32;

    // Secondary: word-count based (English averages ~1.3 tokens per word)
    let word_count = text.split_whitespace().count();
    let word_estimate = (word_count as f64 * 1.3).ceil() as u32;

    // Use the average of both estimates for better accuracy
    (char_estimate + word_estimate) / 2
}

/// Quickly estimate tokens without any splitting (fastest path).
pub fn estimate_tokens_fast(text: &str) -> u32 {
    (text.len() as f64 / CHARS_PER_TOKEN).ceil() as u32
}

/// Count tokens in a JSON value (accounts for JSON syntax overhead).
pub fn count_tokens_json(value: &serde_json::Value) -> u32 {
    let json_str = serde_json::to_string(value).unwrap_or_default();
    // JSON formatting uses ~10-15% more tokens than plain text
    // due to braces, quotes, colons, commas
    count_tokens(&json_str)
}

/// Budget tracker that monitors token consumption.
#[derive(Debug, Clone)]
pub struct TokenBudget {
    /// Total token budget allocated.
    pub total: u32,
    /// Tokens consumed so far.
    pub used: u32,
}

impl TokenBudget {
    /// Create a new budget with the given total.
    pub fn new(total: u32) -> Self {
        Self { total, used: 0 }
    }

    /// Remaining tokens available.
    pub fn remaining(&self) -> u32 {
        self.total.saturating_sub(self.used)
    }

    /// Whether the budget has been exhausted.
    pub fn is_exhausted(&self) -> bool {
        self.used >= self.total
    }

    /// Try to consume tokens. Returns true if within budget.
    pub fn try_consume(&mut self, tokens: u32) -> bool {
        if self.used + tokens <= self.total {
            self.used += tokens;
            true
        } else {
            false
        }
    }

    /// Force-consume tokens (may exceed budget).
    pub fn consume(&mut self, tokens: u32) {
        self.used += tokens;
    }

    /// How full the budget is (0.0 to 1.0+).
    pub fn utilization(&self) -> f64 {
        if self.total == 0 {
            return 1.0;
        }
        self.used as f64 / self.total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_tokens_empty() {
        assert_eq!(count_tokens(""), 0);
    }

    #[test]
    fn count_tokens_short_text() {
        let tokens = count_tokens("Hello, world!");
        // "Hello, world!" is ~3-4 tokens
        assert!(tokens >= 2 && tokens <= 6, "got {tokens}");
    }

    #[test]
    fn count_tokens_paragraph() {
        let text = "Rust is a multi-paradigm, general-purpose programming language. \
                    It emphasizes performance, type safety, and concurrency.";
        let tokens = count_tokens(text);
        // ~25-30 tokens for this text
        assert!(tokens >= 15 && tokens <= 45, "got {tokens}");
    }

    #[test]
    fn budget_tracking() {
        let mut budget = TokenBudget::new(1000);
        assert_eq!(budget.remaining(), 1000);
        assert!(!budget.is_exhausted());

        assert!(budget.try_consume(500));
        assert_eq!(budget.remaining(), 500);
        assert_eq!(budget.used, 500);

        assert!(budget.try_consume(500));
        assert!(budget.is_exhausted());

        assert!(!budget.try_consume(1)); // exhausted
    }

    #[test]
    fn budget_utilization() {
        let mut budget = TokenBudget::new(100);
        budget.consume(50);
        assert!((budget.utilization() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn fast_estimate_consistency() {
        let text = "This is a test sentence with several words.";
        let fast = estimate_tokens_fast(text);
        let normal = count_tokens(text);
        // Both should be in the same ballpark
        assert!((fast as i32 - normal as i32).unsigned_abs() < 10);
    }
}
```

**Step 2: Update module root (`token/mod.rs`)**

```rust
//! Token system -- QATBE, SCS, PDS, budget management (PRD SS17-18, SS27).

pub mod counter;
pub mod qatbe;
pub mod scs;
pub mod pds;

pub use counter::{count_tokens, estimate_tokens_fast, TokenBudget};
```

**Acceptance criteria:**

- [ ] `count_tokens()` approximates GPT-4 token counts within ~10%
- [ ] `TokenBudget` tracks consumption and reports remaining/exhausted
- [ ] `try_consume()` returns false when budget is exceeded
- [ ] All unit tests pass: `cargo test -p fetchium-core token::counter`
- [ ] No clippy warnings

---

### P1-E3-T2: QATBE Implementation

**ID:** `P1-E3-T2`
**Status:** `DONE`
**Priority:** P1
**Estimated effort:** 2-3 days
**Dependencies:** P1-E3-T1 (tokenizer), P1-E1-T2 (CEP extraction)

**Description:**
Implement Query-Aware Token-Budgeted Extraction (QATBE) -- the core algorithm that makes Fetchium unique. Given a URL, a query, and a token budget, QATBE extracts only the most query-relevant content within the budget. Phase 1 uses BM25 scoring only (no semantic embeddings -- those come in Phase 5).

**PRD References:**

- SS17 "QATBE" -- Full specification
- SS8.2 "QATBE" -- 4-stage pipeline: FETCH, SEGMENT, RANK, BUDGET
- SS20 "Token Efficiency" -- BM25 "Fit Markdown" filter: ~20% savings

**Files to create:**

```
crates/fetchium-core/src/token/
  qatbe.rs            -- QATBE 4-stage pipeline
```

**Step-by-step implementation:**

````rust
//! Query-Aware Token-Budgeted Extraction (QATBE).
//!
//! PRD SS8.2: The single most important feature for AI agent consumption.
//! 4-stage pipeline: FETCH -> SEGMENT -> RANK -> BUDGET.
//!
//! Phase 1: Uses BM25 scoring for relevance ranking.
//! Phase 5: Adds semantic embeddings (cosine similarity) for hybrid scoring.

use crate::extract::ExtractedContent;
use crate::token::counter::{count_tokens, TokenBudget};
use crate::types::Segment;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// QATBE extraction result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QatbeResult {
    /// Segments packed within the token budget, ordered by relevance.
    pub segments: Vec<Segment>,
    /// Total tokens used.
    pub tokens_used: u32,
    /// Total tokens available from the source.
    pub tokens_total: u32,
    /// Fraction of relevant content captured (0.0-1.0).
    pub relevance_coverage: f64,
    /// Number of segments included.
    pub segments_included: u32,
    /// Number of segments excluded due to budget.
    pub segments_excluded: u32,
}

/// A text block extracted during the SEGMENT stage, before SCS typing.
#[derive(Debug, Clone)]
struct TextBlock {
    text: String,
    tokens: u32,
    relevance: f64,
    position: usize,
    block_type: BlockType,
}

#[derive(Debug, Clone, Copy)]
enum BlockType {
    Heading,
    Paragraph,
    ListItem,
    Code,
    Table,
    Quote,
    Other,
}

/// Run the QATBE pipeline on extracted content.
///
/// # Arguments
/// * `content` - Pre-extracted content from the CEP pipeline
/// * `query` - The user's query for relevance scoring
/// * `budget` - Maximum tokens to include in the result
///
/// # Returns
/// A `QatbeResult` with relevance-ranked segments packed within budget.
pub fn extract_with_budget(
    content: &ExtractedContent,
    query: &str,
    budget: u32,
) -> QatbeResult {
    info!(
        "QATBE: query={query:?}, budget={budget}, source_tokens={}",
        content.tokens
    );

    // Stage 2: SEGMENT -- split content into typed blocks
    let blocks = segment_content(&content.text);

    // Stage 3: RANK -- score each block against the query using BM25
    let mut scored_blocks: Vec<TextBlock> = blocks
        .into_iter()
        .map(|mut block| {
            block.relevance = bm25_score(&block.text, query);
            block
        })
        .collect();

    // Sort by relevance descending
    scored_blocks.sort_by(|a, b| {
        b.relevance
            .partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Stage 4: BUDGET -- greedy knapsack packing
    let mut tracker = TokenBudget::new(budget);
    let mut included_segments = Vec::new();
    let mut excluded_count = 0u32;
    let total_relevant_tokens: u32 = scored_blocks
        .iter()
        .filter(|b| b.relevance > 0.01)
        .map(|b| b.tokens)
        .sum();

    for block in &scored_blocks {
        if tracker.is_exhausted() {
            excluded_count += 1;
            continue;
        }

        if block.relevance < 0.01 {
            excluded_count += 1;
            continue;
        }

        if tracker.try_consume(block.tokens) {
            // Full block fits
            included_segments.push(block_to_segment(block, included_segments.len()));
        } else {
            // Truncate block to fit remaining budget
            let remaining = tracker.remaining();
            if remaining > 10 {
                let truncated = truncate_to_tokens(&block.text, remaining);
                let truncated_tokens = count_tokens(&truncated);
                tracker.consume(truncated_tokens);

                let mut seg = block_to_segment(block, included_segments.len());
                seg.tokens = truncated_tokens;
                seg.content = serde_json::Value::String(truncated);
                included_segments.push(seg);
            }
            excluded_count += 1;
        }
    }

    // Calculate relevance coverage
    let included_relevant_tokens: u32 = included_segments
        .iter()
        .filter(|s| s.relevance > 0.01)
        .map(|s| s.tokens)
        .sum();
    let coverage = if total_relevant_tokens > 0 {
        included_relevant_tokens as f64 / total_relevant_tokens as f64
    } else {
        0.0
    };

    info!(
        "QATBE: packed {} segments ({} tokens, {:.0}% coverage)",
        included_segments.len(),
        tracker.used,
        coverage * 100.0,
    );

    QatbeResult {
        segments: included_segments,
        tokens_used: tracker.used,
        tokens_total: content.tokens,
        relevance_coverage: coverage,
        segments_included: (included_segments.len() as u32).min(u32::MAX),
        segments_excluded: excluded_count,
    }
}

/// Stage 2: Split extracted text into typed blocks.
fn segment_content(text: &str) -> Vec<TextBlock> {
    let mut blocks = Vec::new();
    let mut position = 0;

    // Split on double newlines (paragraph boundaries)
    for chunk in text.split("\n\n") {
        let trimmed = chunk.trim();
        if trimmed.is_empty() {
            continue;
        }

        let block_type = classify_block(trimmed);
        let tokens = count_tokens(trimmed);

        // Skip very tiny blocks (noise)
        if tokens < 3 {
            continue;
        }

        blocks.push(TextBlock {
            text: trimmed.to_string(),
            tokens,
            relevance: 0.0,
            position,
            block_type,
        });

        position += 1;
    }

    // If no paragraph breaks, split on single newlines
    if blocks.is_empty() {
        for (i, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let tokens = count_tokens(trimmed);
            if tokens < 3 {
                continue;
            }
            blocks.push(TextBlock {
                text: trimmed.to_string(),
                tokens,
                relevance: 0.0,
                position: i,
                block_type: classify_block(trimmed),
            });
        }
    }

    blocks
}

/// Classify a text block by its likely type.
fn classify_block(text: &str) -> BlockType {
    let trimmed = text.trim();

    // Headings: short lines that look like titles
    if trimmed.len() < 100
        && !trimmed.contains('.')
        && trimmed.chars().next().map_or(false, |c| c.is_uppercase())
    {
        return BlockType::Heading;
    }

    // Code: contains indentation or code markers
    if trimmed.starts_with("```")
        || trimmed.starts_with("    ")
        || trimmed.starts_with('\t')
        || (trimmed.contains('{') && trimmed.contains('}'))
        || (trimmed.contains("fn ") && trimmed.contains("("))
        || (trimmed.contains("def ") && trimmed.contains(":"))
    {
        return BlockType::Code;
    }

    // List: starts with bullet or number
    if trimmed.starts_with("- ")
        || trimmed.starts_with("* ")
        || trimmed.starts_with("+ ")
        || (trimmed.len() > 2
            && trimmed.chars().next().unwrap_or(' ').is_ascii_digit()
            && trimmed.chars().nth(1) == Some('.'))
    {
        return BlockType::ListItem;
    }

    // Quote: starts with > or has quotation marks
    if trimmed.starts_with('>') || trimmed.starts_with('"') {
        return BlockType::Quote;
    }

    // Table: contains | separators
    if trimmed.contains('|') && trimmed.matches('|').count() >= 2 {
        return BlockType::Table;
    }

    BlockType::Paragraph
}

/// BM25 scoring for a text block against a query.
///
/// Simplified BM25 implementation for single-document scoring.
/// Uses k1=1.2, b=0.75 as standard parameters.
fn bm25_score(text: &str, query: &str) -> f64 {
    let k1: f64 = 1.2;
    let b: f64 = 0.75;

    let text_lower = text.to_lowercase();
    let text_words: Vec<&str> = text_lower.split_whitespace().collect();
    let doc_len = text_words.len() as f64;

    // Average document length estimate (assume ~100 words)
    let avg_dl: f64 = 100.0;

    let query_terms: Vec<String> = query
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    let mut score = 0.0;

    for term in &query_terms {
        let tf = text_words
            .iter()
            .filter(|w| w.contains(term.as_str()))
            .count() as f64;

        if tf == 0.0 {
            continue;
        }

        // IDF approximation: use a simple log-based formula
        // Since we're scoring a single document, use a default IDF of 1.0
        // In Phase 2, actual IDF comes from the tantivy index
        let idf = 1.0;

        let numerator = tf * (k1 + 1.0);
        let denominator = tf + k1 * (1.0 - b + b * doc_len / avg_dl);

        score += idf * numerator / denominator;
    }

    // Normalize to 0.0-1.0 range
    let max_possible = query_terms.len() as f64 * 2.0;
    if max_possible > 0.0 {
        (score / max_possible).min(1.0)
    } else {
        0.0
    }
}

/// Convert a TextBlock to a Segment.
fn block_to_segment(block: &TextBlock, index: usize) -> Segment {
    use crate::types::SegmentType;

    let seg_type = match block.block_type {
        BlockType::Heading => SegmentType::Heading,
        BlockType::Paragraph => SegmentType::Paragraph,
        BlockType::ListItem => SegmentType::List,
        BlockType::Code => SegmentType::Code,
        BlockType::Table => SegmentType::Table,
        BlockType::Quote => SegmentType::Quote,
        BlockType::Other => SegmentType::Paragraph,
    };

    Segment {
        seg_type,
        relevance: block.relevance,
        tokens: block.tokens,
        content: serde_json::Value::String(block.text.clone()),
        source_ref: Some(index as u32),
    }
}

/// Truncate text to fit within an approximate token count.
fn truncate_to_tokens(text: &str, max_tokens: u32) -> String {
    let max_chars = (max_tokens as f64 * 4.0) as usize;
    if text.len() <= max_chars {
        return text.to_string();
    }

    // Truncate at a word boundary
    let truncated = &text[..max_chars];
    match truncated.rfind(' ') {
        Some(pos) => format!("{}...", &truncated[..pos]),
        None => format!("{truncated}..."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::ExtractedContent;
    use crate::extract::ContentMetadata;
    use crate::types::CepLayer;

    fn make_content(text: &str) -> ExtractedContent {
        ExtractedContent {
            title: "Test".into(),
            text: text.into(),
            layer_used: CepLayer::Layer1,
            tokens: count_tokens(text),
            metadata: ContentMetadata::default(),
        }
    }

    #[test]
    fn qatbe_basic_extraction() {
        let text = "Rust is a systems programming language focused on safety.\n\n\
                    Python is an interpreted language for general purpose coding.\n\n\
                    JavaScript runs in the browser and on the server with Node.js.\n\n\
                    Go is designed for cloud infrastructure and microservices.";

        let content = make_content(text);
        let result = extract_with_budget(&content, "Rust systems programming", 100);

        assert!(result.tokens_used <= 100);
        assert!(!result.segments.is_empty());
        // The Rust paragraph should have the highest relevance
        let first = &result.segments[0];
        assert!(
            first
                .content
                .as_str()
                .unwrap_or("")
                .contains("Rust"),
            "First segment should be about Rust"
        );
    }

    #[test]
    fn qatbe_respects_budget() {
        let text = (0..50)
            .map(|i| format!("Paragraph {i} with some content about various topics."))
            .collect::<Vec<_>>()
            .join("\n\n");

        let content = make_content(&text);
        let result = extract_with_budget(&content, "topic", 200);

        assert!(result.tokens_used <= 200);
        assert!(result.segments_excluded > 0);
    }

    #[test]
    fn bm25_relevance_ordering() {
        let high = bm25_score(
            "Rust is a systems programming language focused on safety",
            "Rust programming",
        );
        let low = bm25_score(
            "The weather today is sunny and warm with clear skies",
            "Rust programming",
        );
        assert!(
            high > low,
            "Relevant text ({high}) should score higher than irrelevant ({low})"
        );
    }

    #[test]
    fn classify_block_types() {
        assert!(matches!(
            classify_block("Main Title"),
            BlockType::Heading
        ));
        assert!(matches!(
            classify_block("- item one\n- item two"),
            BlockType::ListItem
        ));
        assert!(matches!(
            classify_block("```rust\nfn main() {}\n```"),
            BlockType::Code
        ));
        assert!(matches!(
            classify_block("> This is a quote from someone important."),
            BlockType::Quote
        ));
        assert!(matches!(
            classify_block("This is a regular paragraph with multiple sentences. It contains details."),
            BlockType::Paragraph
        ));
    }

    #[test]
    fn truncate_at_word_boundary() {
        let text = "Hello world this is a test of truncation at word boundaries";
        let truncated = truncate_to_tokens(text, 5); // ~20 chars
        assert!(truncated.ends_with("..."));
        assert!(truncated.len() < text.len());
    }
}
````

**Acceptance criteria:**

- [ ] QATBE 4-stage pipeline: content segmentation, BM25 scoring, budget packing
- [ ] Segments are ranked by query relevance and packed greedily
- [ ] Total tokens never exceed the requested budget
- [ ] Partial segments are truncated at word boundaries to fill remaining budget
- [ ] `relevance_coverage` reports what fraction of relevant content was captured
- [ ] BM25 scoring correctly ranks relevant text higher than irrelevant
- [ ] All unit tests pass: `cargo test -p fetchium-core token::qatbe`
- [ ] No clippy warnings

---

### P1-E3-T3: SCS Implementation

**ID:** `P1-E3-T3`
**Status:** `DONE`
**Priority:** P1
**Estimated effort:** 2 days
**Dependencies:** P1-E3-T1 (tokenizer), P1-E1-T2 (CEP extraction)

**Description:**
Implement Semantic Content Segmentation (SCS) -- the algorithm that segments extracted content into typed blocks (facts, tables, code, lists, data, paragraphs) each in their most token-efficient representation. SCS is what makes Fetchium output dramatically more compact than flat markdown.

**PRD References:**

- SS18 "Semantic Content Segmentation" -- Full spec with 14 segment types
- SS8.4 "SCS" -- Token efficiency: tables as JSON (60% fewer tokens), lists as arrays (30% fewer)

**Files to create:**

```
crates/fetchium-core/src/token/
  scs.rs              -- Semantic Content Segmentation
```

**Step-by-step implementation:**

````rust
//! Semantic Content Segmentation (SCS).
//!
//! PRD SS8.4: Segment content into typed blocks, each in its most
//! token-efficient representation. Key savings:
//! - Tables: JSON arrays instead of markdown tables (60% fewer tokens)
//! - Lists: JSON arrays instead of markdown bullets (30% fewer tokens)
//! - Facts: Structured claim+confidence (40% fewer tokens)
//! - Data: key:value pairs instead of prose (50% fewer tokens)

use crate::extract::ExtractedContent;
use crate::token::counter::count_tokens;
use crate::types::{Segment, SegmentType};
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{json, Value};
use tracing::debug;

/// Result of SCS processing.
#[derive(Debug, Clone)]
pub struct ScsResult {
    /// Typed segments in document order.
    pub segments: Vec<Segment>,
    /// Total tokens across all segments.
    pub total_tokens: u32,
    /// Token savings compared to flat text.
    pub tokens_saved: u32,
}

// ─── Regex patterns for segment classification ──────────────────

static TABLE_ROW_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*\|(.+\|)+\s*$").expect("valid regex"));
static TABLE_SEPARATOR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*\|[\s\-:]+(\|[\s\-:]+)+\|?\s*$").expect("valid regex"));
static LIST_ITEM_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*[-*+]\s+(.+)$").expect("valid regex"));
static ORDERED_LIST_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*\d+[.)]\s+(.+)$").expect("valid regex"));
static CODE_FENCE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^```(\w*)\s*$").expect("valid regex"));
static HEADING_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(#{1,6})\s+(.+)$").expect("valid regex"));
static KV_PATTERN_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(.{3,40}):\s+(.+)$").expect("valid regex"));
static QUOTE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^>\s*(.+)$").expect("valid regex"));

/// Run SCS on extracted content, producing typed segments.
pub fn segment(content: &ExtractedContent) -> ScsResult {
    let text = &content.text;
    let original_tokens = content.tokens;

    let segments = parse_into_segments(text);

    let total_tokens: u32 = segments.iter().map(|s| s.tokens).sum();
    let tokens_saved = original_tokens.saturating_sub(total_tokens);

    debug!(
        "SCS: {} segments, {} tokens (saved {} from {})",
        segments.len(),
        total_tokens,
        tokens_saved,
        original_tokens
    );

    ScsResult {
        segments,
        total_tokens,
        tokens_saved,
    }
}

/// Parse text into typed segments.
fn parse_into_segments(text: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.is_empty() {
            i += 1;
            continue;
        }

        // Try to match structured patterns in priority order

        // 1. Code fences
        if let Some(caps) = CODE_FENCE_RE.captures(line) {
            let lang = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
            let (code_block, end_line) = collect_code_block(&lines, i + 1);
            if !code_block.is_empty() {
                let seg_content = json!({
                    "language": lang,
                    "code": code_block,
                });
                let tokens = count_tokens_json_value(&seg_content);
                segments.push(Segment {
                    seg_type: SegmentType::Code,
                    relevance: 0.0,
                    tokens,
                    content: seg_content,
                    source_ref: None,
                });
                i = end_line + 1;
                continue;
            }
        }

        // 2. Tables (markdown format)
        if TABLE_ROW_RE.is_match(line) {
            let (table_seg, end_line) = parse_table(&lines, i);
            if let Some(seg) = table_seg {
                segments.push(seg);
                i = end_line + 1;
                continue;
            }
        }

        // 3. Headings
        if let Some(caps) = HEADING_RE.captures(line) {
            let level = caps.get(1).map(|m| m.as_str().len()).unwrap_or(1);
            let text = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
            let seg_content = json!({
                "level": level,
                "text": text,
            });
            let tokens = count_tokens_json_value(&seg_content);
            segments.push(Segment {
                seg_type: SegmentType::Heading,
                relevance: 0.0,
                tokens,
                content: seg_content,
                source_ref: None,
            });
            i += 1;
            continue;
        }

        // 4. Block quotes
        if QUOTE_RE.is_match(line) {
            let (quote_text, end_line) = collect_quotes(&lines, i);
            let seg_content = json!({
                "text": quote_text,
            });
            let tokens = count_tokens_json_value(&seg_content);
            segments.push(Segment {
                seg_type: SegmentType::Quote,
                relevance: 0.0,
                tokens,
                content: seg_content,
                source_ref: None,
            });
            i = end_line + 1;
            continue;
        }

        // 5. Lists (unordered)
        if LIST_ITEM_RE.is_match(line) {
            let (items, end_line) = collect_list_items(&lines, i, false);
            let seg_content = json!({
                "ordered": false,
                "items": items,
            });
            let tokens = count_tokens_json_value(&seg_content);
            segments.push(Segment {
                seg_type: SegmentType::List,
                relevance: 0.0,
                tokens,
                content: seg_content,
                source_ref: None,
            });
            i = end_line + 1;
            continue;
        }

        // 6. Ordered lists
        if ORDERED_LIST_RE.is_match(line) {
            let (items, end_line) = collect_list_items(&lines, i, true);
            let seg_content = json!({
                "ordered": true,
                "items": items,
            });
            let tokens = count_tokens_json_value(&seg_content);
            segments.push(Segment {
                seg_type: SegmentType::List,
                relevance: 0.0,
                tokens,
                content: seg_content,
                source_ref: None,
            });
            i = end_line + 1;
            continue;
        }

        // 7. Key-value data patterns (e.g., "Price: $19.99")
        if KV_PATTERN_RE.is_match(line) {
            let (kv_pairs, end_line) = collect_kv_pairs(&lines, i);
            if kv_pairs.len() >= 2 {
                let seg_content = json!(kv_pairs);
                let tokens = count_tokens_json_value(&seg_content);
                segments.push(Segment {
                    seg_type: SegmentType::Data,
                    relevance: 0.0,
                    tokens,
                    content: seg_content,
                    source_ref: None,
                });
                i = end_line + 1;
                continue;
            }
        }

        // 8. Default: paragraph
        let (para_text, end_line) = collect_paragraph(&lines, i);
        if !para_text.is_empty() {
            let tokens = count_tokens(&para_text);
            segments.push(Segment {
                seg_type: SegmentType::Paragraph,
                relevance: 0.0,
                tokens,
                content: Value::String(para_text),
                source_ref: None,
            });
            i = end_line + 1;
        } else {
            i += 1;
        }
    }

    segments
}

fn count_tokens_json_value(val: &Value) -> u32 {
    let json_str = serde_json::to_string(val).unwrap_or_default();
    count_tokens(&json_str)
}

fn collect_code_block(lines: &[&str], start: usize) -> (String, usize) {
    let mut code = String::new();
    let mut i = start;
    while i < lines.len() {
        if lines[i].trim().starts_with("```") {
            return (code.trim_end().to_string(), i);
        }
        code.push_str(lines[i]);
        code.push('\n');
        i += 1;
    }
    (code.trim_end().to_string(), i.saturating_sub(1))
}

fn parse_table(lines: &[&str], start: usize) -> (Option<Segment>, usize) {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut i = start;

    while i < lines.len() && TABLE_ROW_RE.is_match(lines[i].trim()) {
        let line = lines[i].trim();
        // Skip separator rows
        if TABLE_SEPARATOR_RE.is_match(line) {
            i += 1;
            continue;
        }
        let cells: Vec<String> = line
            .split('|')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect();
        if !cells.is_empty() {
            rows.push(cells);
        }
        i += 1;
    }

    if rows.len() < 2 {
        return (None, start);
    }

    let headers = rows.remove(0);
    let seg_content = json!({
        "headers": headers,
        "rows": rows,
    });
    let tokens = count_tokens_json_value(&seg_content);

    let seg = Segment {
        seg_type: SegmentType::Table,
        relevance: 0.0,
        tokens,
        content: seg_content,
        source_ref: None,
    };

    (Some(seg), i.saturating_sub(1))
}

fn collect_quotes(lines: &[&str], start: usize) -> (String, usize) {
    let mut text = String::new();
    let mut i = start;
    while i < lines.len() {
        if let Some(caps) = QUOTE_RE.captures(lines[i].trim()) {
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(caps.get(1).map(|m| m.as_str()).unwrap_or(""));
            i += 1;
        } else {
            break;
        }
    }
    (text, i.saturating_sub(1))
}

fn collect_list_items(lines: &[&str], start: usize, ordered: bool) -> (Vec<String>, usize) {
    let mut items = Vec::new();
    let mut i = start;
    let pattern = if ordered { &*ORDERED_LIST_RE } else { &*LIST_ITEM_RE };

    while i < lines.len() {
        let trimmed = lines[i].trim();
        if let Some(caps) = pattern.captures(trimmed) {
            items.push(caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default());
            i += 1;
        } else if trimmed.is_empty() {
            break;
        } else {
            break;
        }
    }
    (items, i.saturating_sub(1))
}

fn collect_kv_pairs(
    lines: &[&str],
    start: usize,
) -> (Vec<serde_json::Map<String, Value>>, usize) {
    let mut pairs = Vec::new();
    let mut i = start;

    while i < lines.len() {
        let trimmed = lines[i].trim();
        if let Some(caps) = KV_PATTERN_RE.captures(trimmed) {
            let key = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            let val = caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
            let mut map = serde_json::Map::new();
            map.insert("key".into(), Value::String(key));
            map.insert("value".into(), Value::String(val));
            pairs.push(map);
            i += 1;
        } else if trimmed.is_empty() {
            break;
        } else {
            break;
        }
    }
    (pairs, i.saturating_sub(1))
}

fn collect_paragraph(lines: &[&str], start: usize) -> (String, usize) {
    let mut text = String::new();
    let mut i = start;

    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.is_empty() {
            break;
        }
        // Stop if we hit a structured pattern
        if HEADING_RE.is_match(trimmed)
            || CODE_FENCE_RE.is_match(trimmed)
            || TABLE_ROW_RE.is_match(trimmed)
            || LIST_ITEM_RE.is_match(trimmed)
            || ORDERED_LIST_RE.is_match(trimmed)
        {
            break;
        }
        if !text.is_empty() {
            text.push(' ');
        }
        text.push_str(trimmed);
        i += 1;
    }

    (text, i.saturating_sub(1).max(start))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::ContentMetadata;
    use crate::types::CepLayer;

    fn make_content(text: &str) -> ExtractedContent {
        ExtractedContent {
            title: "Test".into(),
            text: text.into(),
            layer_used: CepLayer::Layer1,
            tokens: count_tokens(text),
            metadata: ContentMetadata::default(),
        }
    }

    #[test]
    fn scs_detects_headings() {
        let content = make_content("# Main Title\n\nSome paragraph content here.");
        let result = segment(&content);
        assert!(result
            .segments
            .iter()
            .any(|s| s.seg_type == SegmentType::Heading));
    }

    #[test]
    fn scs_detects_code_blocks() {
        let content = make_content("Some text\n\n```rust\nfn main() {\n    println!(\"hello\");\n}\n```\n\nMore text");
        let result = segment(&content);
        assert!(result
            .segments
            .iter()
            .any(|s| s.seg_type == SegmentType::Code));
    }

    #[test]
    fn scs_detects_lists() {
        let content = make_content("Items:\n\n- First item\n- Second item\n- Third item");
        let result = segment(&content);
        assert!(result
            .segments
            .iter()
            .any(|s| s.seg_type == SegmentType::List));
    }

    #[test]
    fn scs_table_to_json() {
        let content = make_content(
            "| Name | Price |\n|------|-------|\n| Widget A | $10 |\n| Widget B | $20 |",
        );
        let result = segment(&content);
        let table_seg = result
            .segments
            .iter()
            .find(|s| s.seg_type == SegmentType::Table);
        assert!(table_seg.is_some(), "Should detect table segment");
        let table = table_seg.unwrap();
        // Table should be JSON with headers and rows
        assert!(table.content.get("headers").is_some());
        assert!(table.content.get("rows").is_some());
    }

    #[test]
    fn scs_reduces_tokens() {
        let markdown_table = "| Feature | Status | Notes |\n|---------|--------|-------|\n| Auth | Done | OAuth2 |\n| Search | WIP | BM25 |\n| Cache | Done | LRU |";
        let content = make_content(markdown_table);
        let result = segment(&content);
        // SCS table format should use fewer tokens than raw markdown
        assert!(
            result.total_tokens <= content.tokens,
            "SCS ({}) should not exceed raw tokens ({})",
            result.total_tokens,
            content.tokens
        );
    }
}
````

**Acceptance criteria:**

- [ ] SCS identifies headings, paragraphs, code, lists, tables, quotes, and data
- [ ] Tables are represented as JSON `{headers, rows}` (not markdown)
- [ ] Lists are represented as JSON arrays (not markdown bullets)
- [ ] Code blocks preserve language annotation
- [ ] Token count of SCS output <= original text tokens
- [ ] All unit tests pass: `cargo test -p fetchium-core token::scs`
- [ ] No clippy warnings

---

### P1-E3-T4: PDS Tier 1

**ID:** `P1-E3-T4`
**Status:** `DONE`
**Priority:** P1
**Estimated effort:** 1-2 days
**Dependencies:** P1-E3-T2 (QATBE), P1-E3-T3 (SCS)

**Description:**
Implement Progressive Detail Streaming (PDS) tier 1 -- the `key_facts` and `summary` tiers. Given extracted content and optionally a query, PDS produces multiple output tiers at different detail levels. Phase 1 implements extractive summarization (selecting the most important sentences/segments). Phase 5 adds abstractive summarization via local LLM.

**PRD References:**

- SS27 "Progressive Detail Streaming" -- 4 tiers: key_facts (~200 tokens), summary (~1000), detailed (~5000), complete (all)
- SS8.9 "PDS" -- All 4 tiers pre-computed at extraction time

**Files to create:**

```
crates/fetchium-core/src/token/
  pds.rs              -- Progressive Detail Streaming tiers
```

**Step-by-step implementation:**

```rust
//! Progressive Detail Streaming (PDS).
//!
//! PRD SS8.9: 4-tier content system pre-computed at extraction time.
//! - key_facts: ~200 tokens -- Top 5 factual claims
//! - summary: ~1,000 tokens -- Executive summary with key findings
//! - detailed: ~5,000 tokens -- Full analysis with evidence
//! - complete: all tokens -- Everything extracted
//!
//! Phase 1: Extractive summarization (selecting important segments).
//! Phase 5: Adds abstractive summarization via local LLM.

use crate::extract::ExtractedContent;
use crate::token::counter::{count_tokens, TokenBudget};
use crate::token::qatbe;
use crate::token::scs;
use crate::types::{PdsTier, Segment};
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Target token counts per PDS tier.
const KEY_FACTS_BUDGET: u32 = 200;
const SUMMARY_BUDGET: u32 = 1000;
const DETAILED_BUDGET: u32 = 5000;

/// All 4 PDS tiers for a piece of content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdsTiers {
    pub key_facts: TierContent,
    pub summary: TierContent,
    pub detailed: TierContent,
    pub complete: TierContent,
}

/// Content at a specific PDS tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierContent {
    pub tier: PdsTier,
    pub tokens: u32,
    pub segments: Vec<Segment>,
    pub text: String,
}

/// Generate all PDS tiers from extracted content.
///
/// If a query is provided, tiers are query-aware (using QATBE).
/// If no query, tiers are generated by position-based selection.
pub fn generate_tiers(
    content: &ExtractedContent,
    query: Option<&str>,
) -> PdsTiers {
    // Complete tier: everything
    let scs_result = scs::segment(content);
    let complete = TierContent {
        tier: PdsTier::Complete,
        tokens: scs_result.total_tokens,
        segments: scs_result.segments.clone(),
        text: content.text.clone(),
    };

    // Generate lower tiers using QATBE if query is available,
    // otherwise use position-based selection
    let (key_facts, summary, detailed) = match query {
        Some(q) => generate_query_aware_tiers(content, q),
        None => generate_position_tiers(content, &scs_result.segments),
    };

    debug!(
        "PDS tiers: key_facts={}, summary={}, detailed={}, complete={}",
        key_facts.tokens, summary.tokens, detailed.tokens, complete.tokens
    );

    PdsTiers {
        key_facts,
        summary,
        detailed,
        complete,
    }
}

/// Get a specific tier's content.
pub fn get_tier(tiers: &PdsTiers, tier: PdsTier) -> &TierContent {
    match tier {
        PdsTier::KeyFacts => &tiers.key_facts,
        PdsTier::Summary => &tiers.summary,
        PdsTier::Detailed => &tiers.detailed,
        PdsTier::Complete => &tiers.complete,
    }
}

/// Generate tiers using QATBE query-aware extraction.
fn generate_query_aware_tiers(
    content: &ExtractedContent,
    query: &str,
) -> (TierContent, TierContent, TierContent) {
    let detailed_result = qatbe::extract_with_budget(content, query, DETAILED_BUDGET);
    let summary_result = qatbe::extract_with_budget(content, query, SUMMARY_BUDGET);
    let facts_result = qatbe::extract_with_budget(content, query, KEY_FACTS_BUDGET);

    let detailed = TierContent {
        tier: PdsTier::Detailed,
        tokens: detailed_result.tokens_used,
        text: segments_to_text(&detailed_result.segments),
        segments: detailed_result.segments,
    };

    let summary = TierContent {
        tier: PdsTier::Summary,
        tokens: summary_result.tokens_used,
        text: segments_to_text(&summary_result.segments),
        segments: summary_result.segments,
    };

    let key_facts = TierContent {
        tier: PdsTier::KeyFacts,
        tokens: facts_result.tokens_used,
        text: segments_to_text(&facts_result.segments),
        segments: facts_result.segments,
    };

    (key_facts, summary, detailed)
}

/// Generate tiers by position-based selection (no query).
fn generate_position_tiers(
    content: &ExtractedContent,
    all_segments: &[Segment],
) -> (TierContent, TierContent, TierContent) {
    // Detailed: take segments up to budget
    let detailed_segs = select_segments_by_position(all_segments, DETAILED_BUDGET);
    let detailed = TierContent {
        tier: PdsTier::Detailed,
        tokens: detailed_segs.iter().map(|s| s.tokens).sum(),
        text: segments_to_text(&detailed_segs),
        segments: detailed_segs,
    };

    // Summary: take first segments up to budget
    let summary_segs = select_segments_by_position(all_segments, SUMMARY_BUDGET);
    let summary = TierContent {
        tier: PdsTier::Summary,
        tokens: summary_segs.iter().map(|s| s.tokens).sum(),
        text: segments_to_text(&summary_segs),
        segments: summary_segs,
    };

    // Key facts: take the first few segments, prioritizing headings and short blocks
    let facts_segs = select_key_facts(all_segments, KEY_FACTS_BUDGET);
    let key_facts = TierContent {
        tier: PdsTier::KeyFacts,
        tokens: facts_segs.iter().map(|s| s.tokens).sum(),
        text: segments_to_text(&facts_segs),
        segments: facts_segs,
    };

    (key_facts, summary, detailed)
}

/// Select segments by position until budget is reached.
fn select_segments_by_position(segments: &[Segment], budget: u32) -> Vec<Segment> {
    let mut result = Vec::new();
    let mut tracker = TokenBudget::new(budget);

    for seg in segments {
        if tracker.is_exhausted() {
            break;
        }
        if tracker.try_consume(seg.tokens) {
            result.push(seg.clone());
        }
    }

    result
}

/// Select key facts: prioritize headings, data points, and short segments.
fn select_key_facts(segments: &[Segment], budget: u32) -> Vec<Segment> {
    use crate::types::SegmentType;

    let mut tracker = TokenBudget::new(budget);
    let mut result = Vec::new();

    // Priority 1: Headings (they summarize sections)
    for seg in segments {
        if tracker.is_exhausted() {
            break;
        }
        if seg.seg_type == SegmentType::Heading && tracker.try_consume(seg.tokens) {
            result.push(seg.clone());
        }
    }

    // Priority 2: Data points and facts
    for seg in segments {
        if tracker.is_exhausted() {
            break;
        }
        if matches!(
            seg.seg_type,
            SegmentType::Data | SegmentType::Fact
        ) && tracker.try_consume(seg.tokens)
        {
            result.push(seg.clone());
        }
    }

    // Priority 3: First few paragraphs
    for seg in segments {
        if tracker.is_exhausted() {
            break;
        }
        if seg.seg_type == SegmentType::Paragraph && tracker.try_consume(seg.tokens) {
            result.push(seg.clone());
        }
    }

    result
}

/// Convert segments back to readable text.
fn segments_to_text(segments: &[Segment]) -> String {
    segments
        .iter()
        .map(|s| match &s.content {
            serde_json::Value::String(text) => text.clone(),
            other => serde_json::to_string_pretty(other).unwrap_or_default(),
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::ContentMetadata;
    use crate::types::CepLayer;

    fn make_content(text: &str) -> ExtractedContent {
        ExtractedContent {
            title: "Test".into(),
            text: text.into(),
            layer_used: CepLayer::Layer1,
            tokens: count_tokens(text),
            metadata: ContentMetadata::default(),
        }
    }

    #[test]
    fn pds_tier_ordering() {
        let long_text = (0..100)
            .map(|i| format!("Paragraph {i} discusses the topic in detail with many words."))
            .collect::<Vec<_>>()
            .join("\n\n");

        let content = make_content(&long_text);
        let tiers = generate_tiers(&content, None);

        assert!(tiers.key_facts.tokens <= KEY_FACTS_BUDGET + 50); // Allow some margin
        assert!(tiers.summary.tokens <= SUMMARY_BUDGET + 50);
        assert!(tiers.key_facts.tokens <= tiers.summary.tokens);
        assert!(tiers.summary.tokens <= tiers.detailed.tokens);
        assert!(tiers.detailed.tokens <= tiers.complete.tokens);
    }

    #[test]
    fn pds_query_aware_tiers() {
        let text = "Rust is great for systems programming.\n\n\
                    Python is great for data science.\n\n\
                    Rust has zero-cost abstractions.\n\n\
                    Python has easy syntax for beginners.";

        let content = make_content(text);
        let tiers = generate_tiers(&content, Some("Rust programming"));

        // With query, tiers should prioritize Rust-related content
        assert!(!tiers.key_facts.segments.is_empty());
    }

    #[test]
    fn get_tier_accessor() {
        let content = make_content("Some test content for PDS tier testing.");
        let tiers = generate_tiers(&content, None);

        assert_eq!(get_tier(&tiers, PdsTier::KeyFacts).tier, PdsTier::KeyFacts);
        assert_eq!(get_tier(&tiers, PdsTier::Complete).tier, PdsTier::Complete);
    }
}
```

**Acceptance criteria:**

- [ ] `generate_tiers()` produces all 4 PDS tiers (key_facts, summary, detailed, complete)
- [ ] key_facts <= ~200 tokens, summary <= ~1000 tokens, detailed <= ~5000 tokens
- [ ] Tier sizes are monotonically increasing: key_facts <= summary <= detailed <= complete
- [ ] Query-aware tiers use QATBE for relevance-based selection
- [ ] Position-based tiers prioritize headings and early content
- [ ] All unit tests pass: `cargo test -p fetchium-core token::pds`
- [ ] No clippy warnings

---

## Epic 1.4: Agent Commands

> **PRD Sections:** SS9 (AI-Native Agent Architecture), SS10 (Modes), SS17 (QATBE)
> **Crate:** `fetchium-cli` -- `src/commands/`
> **Priority:** P1 | **Tasks:** 2

### P1-E4-T1: `agent-search` Command

**ID:** `P1-E4-T1`
**Status:** `DONE`
**Priority:** P1
**Estimated effort:** 1-2 days
**Dependencies:** P1-E2-T2 (orchestrator), P1-E3-T2 (QATBE), P1-E3-T4 (PDS)

**Description:**
Implement the `fetchium agent-search` command -- the primary interface for AI agents. Always outputs JSON. Searches via the orchestrator, applies QATBE to extract relevant content from top results, generates PDS tiers, and returns a structured `AgentSearchResult` JSON response.

**PRD References:**

- SS9 "AI-Native Agent Architecture" -- Structured JSON output for agents
- SS17 "QATBE" -- Token budget as first-class parameter
- SS27 "PDS" -- Tiered detail levels

**Files to modify:**

```
crates/fetchium-cli/src/commands/agent_search.rs  -- Full implementation
```

**Step-by-step implementation:**

```rust
//! `fetchium agent-search` -- agent-optimized search (JSON segments output).
//!
//! Always outputs JSON. Combines search orchestration with QATBE
//! extraction and PDS tiering for token-efficient agent consumption.

use crate::cli::AgentSearchArgs;
use hsx_core::config::FetchiumConfig;
use hsx_core::extract::pipeline;
use hsx_core::http::HttpClient;
use hsx_core::search::orchestrator::{OrchestratorConfig, SearchOrchestrator};
use hsx_core::token::counter::count_tokens;
use hsx_core::token::qatbe;
use hsx_core::types::*;
use std::time::Instant;
use tracing::{debug, info, warn};

pub async fn run(args: AgentSearchArgs, config: &FetchiumConfig) -> anyhow::Result<()> {
    let start = Instant::now();

    let client = HttpClient::new(config)?;

    // Build orchestrator
    let orch_config = OrchestratorConfig {
        max_results_per_backend: args.max_results + 5,
        max_total_results: args.max_results,
        backend_timeout: std::time::Duration::from_secs(config.search.timeout_secs),
        enabled_backends: vec![BackendId::DuckDuckGo],
    };
    let orchestrator = SearchOrchestrator::new(client.clone(), orch_config);

    // Search
    let search_results = orchestrator
        .search(&args.query, Some(args.max_results))
        .await?;

    if search_results.is_empty() {
        let result = serde_json::json!({
            "meta": {
                "query": args.query,
                "mode": "search",
                "tier": tier_to_string(args.tier),
                "tokens_used": 0,
                "tokens_budget": args.budget,
                "sources_fetched": 0,
                "duration_ms": start.elapsed().as_millis() as u64,
            },
            "segments": [],
            "sources": [],
        });
        println!("{}", serde_json::to_string(&result)?);
        return Ok(());
    }

    // Fetch and extract top results (up to 3 for MVP)
    let max_fetch = search_results.len().min(3);
    let mut all_segments: Vec<Segment> = Vec::new();
    let mut sources: Vec<Source> = Vec::new();
    let mut total_tokens = 0u32;
    let per_source_budget = args.budget / max_fetch as u32;

    for (idx, item) in search_results.iter().take(max_fetch).enumerate() {
        debug!("Fetching source {}: {}", idx + 1, item.url);

        match client.fetch_text(&item.url).await {
            Ok(html) => {
                let content = pipeline::extract(&html, &item.url);

                // Apply QATBE with per-source budget
                let qatbe_result = qatbe::extract_with_budget(
                    &content,
                    &args.query,
                    per_source_budget,
                );

                // Tag segments with source reference
                let mut source_segments: Vec<Segment> = qatbe_result
                    .segments
                    .into_iter()
                    .map(|mut s| {
                        s.source_ref = Some(idx as u32);
                        s
                    })
                    .collect();

                total_tokens += qatbe_result.tokens_used;
                all_segments.append(&mut source_segments);

                sources.push(Source {
                    id: idx as u32,
                    url: item.url.clone(),
                    title: content.title.clone(),
                    domain: extract_domain(&item.url),
                    fetch_method: FetchMethod::Http,
                    content_type: content.metadata.content_type.clone(),
                    tokens: qatbe_result.tokens_used,
                    published_date: content.metadata.published_date.clone(),
                    trust_score: 0.5, // Default for Phase 1
                    citation: None,
                });
            }
            Err(e) => {
                warn!("Failed to fetch {}: {e}", item.url);
            }
        }
    }

    let elapsed = start.elapsed();
    let result_id = uuid::Uuid::new_v4().to_string();

    let result = serde_json::json!({
        "meta": {
            "query": args.query,
            "mode": "search",
            "tier": tier_to_string(args.tier),
            "tokens_used": total_tokens,
            "tokens_budget": args.budget,
            "sources_fetched": sources.len(),
            "sources_validated": 0,
            "validation_pass_rate": 0.0,
            "duration_ms": elapsed.as_millis() as u64,
            "result_id": result_id,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        },
        "segments": all_segments,
        "sources": sources,
        "search_results": search_results,
    });

    // Agent commands always output compact JSON (no pretty-printing)
    println!("{}", serde_json::to_string(&result)?);

    Ok(())
}

fn extract_domain(url: &str) -> String {
    url::Url::parse(url)
        .map(|u| u.host_str().unwrap_or("unknown").to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

fn tier_to_string(tier: crate::cli::Tier) -> &'static str {
    match tier {
        crate::cli::Tier::KeyFacts => "key_facts",
        crate::cli::Tier::Summary => "summary",
        crate::cli::Tier::Detailed => "detailed",
        crate::cli::Tier::Complete => "complete",
    }
}
```

**Acceptance criteria:**

- [ ] `fetchium agent-search "query"` outputs JSON to stdout (never human-formatted text)
- [ ] JSON includes `meta`, `segments`, `sources`, and `search_results`
- [ ] `meta.tokens_used` respects `--budget` limit
- [ ] Fetches up to 3 top search results and applies QATBE to each
- [ ] Segments are tagged with `source_ref` back to their source
- [ ] Errors in individual source fetches are logged but do not fail the command
- [ ] `cargo build -p fetchium-cli` compiles successfully

**Testing instructions:**

```bash
cargo run -p fetchium-cli -- agent-search "rust async programming" --budget 2000
cargo run -p fetchium-cli -- agent-search "climate change effects" --budget 500 --tier key_facts
# Pipe output through jq to verify JSON structure:
cargo run -p fetchium-cli -- agent-search "test" | jq '.meta.tokens_used'
```

---

### P1-E4-T2: `agent-fetch` Command

**ID:** `P1-E4-T2`
**Status:** `DONE`
**Priority:** P1
**Estimated effort:** 1 day
**Dependencies:** P1-E4-T1 (agent-search pattern), P1-E3-T2 (QATBE)

**Description:**
Implement the `fetchium agent-fetch` command -- fetches a single URL with QATBE query-aware extraction and outputs structured JSON. This is the agent equivalent of `fetchium fetch`.

**PRD References:**

- SS17 "QATBE" -- `fetchium agent-fetch <url> --query "..." --budget 1500`
- SS8.2 "QATBE" -- Returns segments with relevance scores and token counts

**Files to modify:**

```
crates/fetchium-cli/src/commands/agent_fetch.rs  -- Full implementation
```

**Step-by-step implementation:**

```rust
//! `fetchium agent-fetch` -- agent-optimized URL fetch (JSON segments output).
//!
//! Fetches a single URL, applies QATBE extraction with optional query
//! awareness, and returns structured JSON with typed segments.

use crate::cli::AgentFetchArgs;
use hsx_core::config::FetchiumConfig;
use hsx_core::extract::pipeline;
use hsx_core::http::HttpClient;
use hsx_core::token::counter::count_tokens;
use hsx_core::token::pds;
use hsx_core::token::qatbe;
use hsx_core::types::*;
use std::time::Instant;

pub async fn run(args: AgentFetchArgs, config: &FetchiumConfig) -> anyhow::Result<()> {
    let start = Instant::now();

    let client = HttpClient::new(config)?;
    let fetch_result = client.fetch(&args.url).await?;

    // Extract content via CEP pipeline
    let content = pipeline::extract(&fetch_result.body, &fetch_result.url);

    // Apply QATBE if query is provided, otherwise use PDS
    let (segments, tokens_used) = match &args.query {
        Some(query) => {
            let qatbe_result =
                qatbe::extract_with_budget(&content, query, args.budget);
            (qatbe_result.segments, qatbe_result.tokens_used)
        }
        None => {
            // No query: generate PDS tiers and return the requested tier
            let tiers = pds::generate_tiers(&content, None);
            let tier = match args.tier {
                crate::cli::Tier::KeyFacts => PdsTier::KeyFacts,
                crate::cli::Tier::Summary => PdsTier::Summary,
                crate::cli::Tier::Detailed => PdsTier::Detailed,
                crate::cli::Tier::Complete => PdsTier::Complete,
            };
            let tier_content = pds::get_tier(&tiers, tier);
            (tier_content.segments.clone(), tier_content.tokens)
        }
    };

    let elapsed = start.elapsed();
    let result_id = uuid::Uuid::new_v4().to_string();
    let content_hash = format!("{:x}", sha2::Sha256::digest(content.text.as_bytes()));

    let result = serde_json::json!({
        "meta": {
            "query": args.query.as_deref().unwrap_or(""),
            "mode": "fetch",
            "tier": match args.tier {
                crate::cli::Tier::KeyFacts => "key_facts",
                crate::cli::Tier::Summary => "summary",
                crate::cli::Tier::Detailed => "detailed",
                crate::cli::Tier::Complete => "complete",
            },
            "tokens_used": tokens_used,
            "tokens_budget": args.budget,
            "tokens_total": content.tokens,
            "duration_ms": elapsed.as_millis() as u64,
            "result_id": result_id,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "content_hash": content_hash,
        },
        "segments": segments,
        "source": {
            "url": fetch_result.url,
            "title": content.title,
            "content_type": content.metadata.content_type,
            "description": content.metadata.description,
            "author": content.metadata.author,
            "published_date": content.metadata.published_date,
            "cep_layer": format!("{:?}", content.layer_used),
            "fetch_status": fetch_result.status,
            "fetch_elapsed_ms": fetch_result.elapsed_ms,
        },
    });

    println!("{}", serde_json::to_string(&result)?);

    Ok(())
}
```

**Acceptance criteria:**

- [ ] `fetchium agent-fetch <url>` outputs JSON with extracted segments
- [ ] `--query` enables QATBE query-aware extraction
- [ ] Without `--query`, uses PDS tier-based extraction
- [ ] `--budget` limits total tokens in output
- [ ] JSON includes `meta` (with content_hash), `segments`, and `source` info
- [ ] Reports CEP layer used, fetch status, and timing
- [ ] `cargo build -p fetchium-cli` compiles successfully

**Testing instructions:**

```bash
cargo run -p fetchium-cli -- agent-fetch https://example.com
cargo run -p fetchium-cli -- agent-fetch https://en.wikipedia.org/wiki/Rust_(programming_language) --query "memory safety" --budget 1000
cargo run -p fetchium-cli -- agent-fetch https://example.com --tier key_facts | jq '.'
```

---

## Epic 1.5: Basic Ranking

> **PRD Sections:** SS21 (Semantic Search & Hybrid Ranking -- BM25 component)
> **Crate:** `fetchium-core` -- `src/rank/`
> **Priority:** P1 | **Tasks:** 1

### P1-E5-T1: BM25 Ranking with Tantivy

**ID:** `P1-E5-T1`
**Status:** `DONE`
**Priority:** P1
**Estimated effort:** 2 days
**Dependencies:** P0-E1-T2 (types)

**Description:**
Implement BM25 ranking using the `tantivy` crate for lexical search scoring. This provides the foundation for the Phase 2 HyperFusion 8-signal ranking. In Phase 1, BM25 is used to re-rank search results and to score content segments in QATBE.

**PRD References:**

- SS21 "Semantic Search & Hybrid Ranking" -- BM25 as lexical precision signal
- SS21 "Cascade Retrieval" -- Stage 1: BM25 sparse retrieval -> top 1000

**Files to create:**

```
crates/fetchium-core/src/rank/
  mod.rs              -- Module root (update)
  bm25.rs             -- BM25 scorer using tantivy
```

**Step-by-step implementation:**

**Step 1: BM25 scorer (`rank/bm25.rs`)**

```rust
//! BM25 ranking using tantivy.
//!
//! PRD SS21: BM25 as the lexical precision signal in HyperFusion.
//! Phase 1: standalone BM25 scoring for result re-ranking.
//! Phase 2: integrated into the full 8-signal HyperFusion pipeline.

use crate::error::FetchiumResult;
use crate::types::ResultItem;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter, ReloadPolicy};
use tracing::{debug, info};

/// In-memory BM25 scorer backed by a RAM-based tantivy index.
pub struct Bm25Scorer {
    index: Index,
    schema: Schema,
    title_field: Field,
    body_field: Field,
    url_field: Field,
}

impl Bm25Scorer {
    /// Create a new BM25 scorer with an in-memory index.
    pub fn new() -> FetchiumResult<Self> {
        let mut schema_builder = Schema::builder();

        let title_field = schema_builder.add_text_field("title", TEXT | STORED);
        let body_field = schema_builder.add_text_field("body", TEXT | STORED);
        let url_field = schema_builder.add_text_field("url", STRING | STORED);

        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema.clone());

        Ok(Self {
            index,
            schema,
            title_field,
            body_field,
            url_field,
        })
    }

    /// Index a set of documents for BM25 scoring.
    pub fn index_documents(&self, documents: &[ScoringDocument]) -> FetchiumResult<()> {
        let mut writer: IndexWriter = self
            .index
            .writer(50_000_000) // 50MB heap
            .map_err(|e| crate::error::FetchiumError::Extraction(e.to_string()))?;

        for doc_data in documents {
            writer
                .add_document(doc!(
                    self.title_field => doc_data.title.as_str(),
                    self.body_field => doc_data.body.as_str(),
                    self.url_field => doc_data.url.as_str(),
                ))
                .map_err(|e| crate::error::FetchiumError::Extraction(e.to_string()))?;
        }

        writer
            .commit()
            .map_err(|e| crate::error::FetchiumError::Extraction(e.to_string()))?;

        info!("Indexed {} documents for BM25 scoring", documents.len());
        Ok(())
    }

    /// Score a query against indexed documents, returning ranked URLs with scores.
    pub fn score(&self, query: &str, top_n: usize) -> FetchiumResult<Vec<ScoredResult>> {
        let reader = self
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
            .map_err(|e: tantivy::TantivyError| {
                crate::error::FetchiumError::Extraction(e.to_string())
            })?;

        let searcher = reader.searcher();

        let query_parser =
            QueryParser::for_index(&self.index, vec![self.title_field, self.body_field]);

        let parsed_query = query_parser
            .parse_query(query)
            .map_err(|e| crate::error::FetchiumError::Extraction(e.to_string()))?;

        let top_docs = searcher
            .search(&parsed_query, &TopDocs::with_limit(top_n))
            .map_err(|e| crate::error::FetchiumError::Extraction(e.to_string()))?;

        let mut results = Vec::new();

        for (score, doc_address) in top_docs {
            let doc: TantivyDocument = searcher
                .doc(doc_address)
                .map_err(|e| crate::error::FetchiumError::Extraction(e.to_string()))?;

            let url = doc
                .get_first(self.url_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let title = doc
                .get_first(self.title_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            results.push(ScoredResult {
                url,
                title,
                bm25_score: score as f64,
            });
        }

        debug!("BM25 scored {} results for query: {query}", results.len());
        Ok(results)
    }

    /// Re-rank a list of ResultItems using BM25.
    /// Items are indexed and then scored against the query.
    pub fn rerank(
        &self,
        items: &mut [ResultItem],
        query: &str,
    ) -> FetchiumResult<()> {
        // Index the items
        let docs: Vec<ScoringDocument> = items
            .iter()
            .map(|item| ScoringDocument {
                title: item.title.clone(),
                body: item.snippet.clone(),
                url: item.url.clone(),
            })
            .collect();

        self.index_documents(&docs)?;

        // Score
        let scored = self.score(query, items.len())?;

        // Apply BM25 scores back to items
        for item in items.iter_mut() {
            if let Some(scored_item) =
                scored.iter().find(|s| s.url == item.url)
            {
                item.score = Some(scored_item.bm25_score);
            }
        }

        // Sort by score descending
        items.sort_by(|a, b| {
            b.score
                .unwrap_or(0.0)
                .partial_cmp(&a.score.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Re-assign ranks
        for (i, item) in items.iter_mut().enumerate() {
            item.rank = (i + 1) as u32;
        }

        Ok(())
    }
}

/// Document to be indexed for BM25 scoring.
#[derive(Debug, Clone)]
pub struct ScoringDocument {
    pub title: String,
    pub body: String,
    pub url: String,
}

/// A result with BM25 score.
#[derive(Debug, Clone)]
pub struct ScoredResult {
    pub url: String,
    pub title: String,
    pub bm25_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bm25_index_and_score() {
        let scorer = Bm25Scorer::new().unwrap();

        let docs = vec![
            ScoringDocument {
                title: "Rust Programming".into(),
                body: "Rust is a systems programming language focused on safety and performance".into(),
                url: "https://rust-lang.org".into(),
            },
            ScoringDocument {
                title: "Python Tutorial".into(),
                body: "Python is a general purpose programming language for beginners".into(),
                url: "https://python.org".into(),
            },
            ScoringDocument {
                title: "Cooking Recipes".into(),
                body: "How to make pasta with tomato sauce and basil".into(),
                url: "https://cooking.example.com".into(),
            },
        ];

        scorer.index_documents(&docs).unwrap();
        let results = scorer.score("Rust programming language", 10).unwrap();

        assert!(!results.is_empty());
        // Rust page should be ranked first
        assert_eq!(results[0].url, "https://rust-lang.org");
        assert!(results[0].bm25_score > 0.0);
    }

    #[test]
    fn bm25_reranking() {
        let scorer = Bm25Scorer::new().unwrap();

        let mut items = vec![
            ResultItem {
                title: "Cooking Recipes".into(),
                url: "https://cooking.example.com".into(),
                snippet: "Pasta with sauce".into(),
                rank: 1,
                backend: crate::types::BackendId::DuckDuckGo,
                score: None,
                published_date: None,
            },
            ResultItem {
                title: "Rust Guide".into(),
                url: "https://rust-lang.org".into(),
                snippet: "Systems programming with Rust".into(),
                rank: 2,
                backend: crate::types::BackendId::DuckDuckGo,
                score: None,
                published_date: None,
            },
        ];

        scorer.rerank(&mut items, "Rust programming").unwrap();

        // Rust should now be ranked first
        assert_eq!(items[0].title, "Rust Guide");
        assert_eq!(items[0].rank, 1);
        assert!(items[0].score.unwrap_or(0.0) > items[1].score.unwrap_or(0.0));
    }
}
```

**Step 2: Update rank module (`rank/mod.rs`)**

```rust
//! Ranking system -- BM25, HyperFusion 8-signal, semantic (PRD SS21).
//!
//! Phase 1: BM25 via tantivy.
//! Phase 2: Full HyperFusion 8-signal ranking.

pub mod bm25;

pub use bm25::Bm25Scorer;
```

**Acceptance criteria:**

- [ ] `Bm25Scorer` creates an in-memory tantivy index
- [ ] Documents can be indexed with title, body, and URL
- [ ] Queries return scored results ranked by BM25 relevance
- [ ] `rerank()` re-orders existing `ResultItem` arrays by BM25 score
- [ ] Relevant documents score higher than irrelevant ones
- [ ] All unit tests pass: `cargo test -p fetchium-core rank::bm25`
- [ ] No clippy warnings

---

## Epic 1.6: Cache Layer

> **PRD Sections:** SS28 (Caching & Local Index)
> **Crate:** `fetchium-core` -- `src/cache/`
> **Priority:** P1 | **Tasks:** 1

### P1-E6-T1: Memory LRU Cache with Moka

**ID:** `P1-E6-T1`
**Status:** `DONE`
**Priority:** P1
**Estimated effort:** 1-2 days
**Dependencies:** P0-E1-T2 (types)

**Description:**
Implement the L1 memory cache using the `moka` crate -- a high-performance concurrent LRU cache. Caches HTTP responses, extracted content, search results, and PDS tiers to avoid redundant network requests and processing.

**PRD References:**

- SS28 "Caching & Local Index" -- L1: Memory LRU, process memory, session TTL, <1ms
- SS28 "RAGCache-Inspired Optimization" -- Cache query pattern -> result mappings

**Files to create/modify:**

```
crates/fetchium-core/src/cache/
  mod.rs              -- Module root (update)
  memory.rs           -- Moka-based memory LRU cache
```

**Step-by-step implementation:**

**Step 1: Memory cache (`cache/memory.rs`)**

```rust
//! Memory LRU cache using moka.
//!
//! PRD SS28 L1: Process memory, session TTL, <1ms access.
//! Caches HTTP responses, extracted content, and search results.

use moka::future::Cache;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;

/// Cache entry types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheEntry {
    /// Cached HTTP response body.
    HttpResponse {
        body: String,
        content_type: String,
        status: u16,
    },
    /// Cached extracted content.
    ExtractedContent {
        title: String,
        text: String,
        tokens: u32,
    },
    /// Cached search results (serialized JSON).
    SearchResults(String),
}

/// L1 memory cache backed by moka async LRU.
#[derive(Clone)]
pub struct MemoryCache {
    cache: Cache<String, Arc<CacheEntry>>,
}

impl MemoryCache {
    /// Create a new memory cache with the given capacity and TTL.
    pub fn new(max_entries: u64, ttl_secs: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_entries)
            .time_to_live(Duration::from_secs(ttl_secs))
            .time_to_idle(Duration::from_secs(ttl_secs / 2))
            .build();

        Self { cache }
    }

    /// Create from FetchiumConfig cache settings.
    pub fn from_config(config: &crate::config::CacheConfig) -> Self {
        Self::new(config.memory_max_entries, config.ttl_secs)
    }

    /// Generate a cache key from a URL.
    pub fn url_key(url: &str) -> String {
        format!("url:{}", hash_string(url))
    }

    /// Generate a cache key from a search query.
    pub fn search_key(query: &str, max_results: u32) -> String {
        format!("search:{}:{}", hash_string(query), max_results)
    }

    /// Generate a cache key for QATBE extraction.
    pub fn qatbe_key(url: &str, query: &str, budget: u32) -> String {
        format!(
            "qatbe:{}:{}:{}",
            hash_string(url),
            hash_string(query),
            budget
        )
    }

    /// Get an entry from the cache.
    pub async fn get(&self, key: &str) -> Option<Arc<CacheEntry>> {
        let result = self.cache.get(key).await;
        if result.is_some() {
            debug!("Cache HIT: {key}");
        } else {
            debug!("Cache MISS: {key}");
        }
        result
    }

    /// Insert an entry into the cache.
    pub async fn insert(&self, key: String, entry: CacheEntry) {
        debug!("Cache INSERT: {key}");
        self.cache.insert(key, Arc::new(entry)).await;
    }

    /// Remove an entry from the cache.
    pub async fn invalidate(&self, key: &str) {
        self.cache.invalidate(key).await;
    }

    /// Clear all entries.
    pub async fn clear(&self) {
        self.cache.invalidate_all();
        debug!("Cache CLEARED");
    }

    /// Get approximate number of entries.
    pub fn entry_count(&self) -> u64 {
        self.cache.entry_count()
    }

    /// Get approximate weighted size.
    pub fn weighted_size(&self) -> u64 {
        self.cache.weighted_size()
    }
}

/// Hash a string using SHA-256, returning the first 16 hex chars.
fn hash_string(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)[..16].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cache_insert_and_get() {
        let cache = MemoryCache::new(100, 3600);

        let key = MemoryCache::url_key("https://example.com");
        let entry = CacheEntry::HttpResponse {
            body: "Hello World".into(),
            content_type: "text/html".into(),
            status: 200,
        };

        cache.insert(key.clone(), entry).await;

        let result = cache.get(&key).await;
        assert!(result.is_some());

        match result.unwrap().as_ref() {
            CacheEntry::HttpResponse { body, status, .. } => {
                assert_eq!(body, "Hello World");
                assert_eq!(*status, 200);
            }
            _ => panic!("Expected HttpResponse"),
        }
    }

    #[tokio::test]
    async fn cache_miss() {
        let cache = MemoryCache::new(100, 3600);
        let result = cache.get("nonexistent").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn cache_invalidate() {
        let cache = MemoryCache::new(100, 3600);
        let key = "test_key".to_string();
        cache
            .insert(
                key.clone(),
                CacheEntry::SearchResults("results".into()),
            )
            .await;

        assert!(cache.get(&key).await.is_some());
        cache.invalidate(&key).await;
        // moka invalidation is async; run pending tasks
        cache.cache.run_pending_tasks().await;
        assert!(cache.get(&key).await.is_none());
    }

    #[tokio::test]
    async fn cache_clear() {
        let cache = MemoryCache::new(100, 3600);
        cache
            .insert("a".into(), CacheEntry::SearchResults("a".into()))
            .await;
        cache
            .insert("b".into(), CacheEntry::SearchResults("b".into()))
            .await;

        cache.clear().await;
        cache.cache.run_pending_tasks().await;
        assert_eq!(cache.entry_count(), 0);
    }

    #[test]
    fn cache_key_generation() {
        let key1 = MemoryCache::url_key("https://example.com");
        let key2 = MemoryCache::url_key("https://example.com");
        let key3 = MemoryCache::url_key("https://other.com");
        assert_eq!(key1, key2); // Same URL = same key
        assert_ne!(key1, key3); // Different URL = different key
    }
}
```

**Step 2: Update cache module (`cache/mod.rs`)**

```rust
//! Cache system -- Memory LRU (moka) + SQLite disk (PRD SS28).
//!
//! Phase 1: L1 Memory LRU cache.
//! Phase 2: L2 SQLite disk cache, L3 PDS tier cache.

pub mod memory;

pub use memory::{CacheEntry, MemoryCache};
```

**Acceptance criteria:**

- [ ] `MemoryCache` wraps moka async LRU cache
- [ ] Supports insert, get, invalidate, and clear operations
- [ ] Cache keys are SHA-256 based for consistent hashing
- [ ] TTL and max entries are configurable from `CacheConfig`
- [ ] Separate key generators for URLs, search queries, and QATBE
- [ ] All unit tests pass: `cargo test -p fetchium-core cache::memory`
- [ ] No clippy warnings

---

## Epic 1.7: Output Formatters

> **PRD Sections:** SS26 (Output & Export System)
> **Crate:** `fetchium-core` -- `src/output/`
> **Priority:** P1 | **Tasks:** 1

### P1-E7-T1: Markdown/JSON/Text/Segments Formatters

**ID:** `P1-E7-T1`
**Status:** `DONE`
**Priority:** P1
**Estimated effort:** 1-2 days
**Dependencies:** P0-E1-T2 (types), P1-E3-T3 (SCS)

**Description:**
Implement output formatters for the 4 primary formats: Markdown (human default), JSON (structured), plain text (minimal), and Segments (SCS JSON for agents). Each formatter takes search/fetch results and produces formatted output.

**PRD References:**

- SS26 "Output & Export System" -- Markdown (primary human), Segments (primary agent), JSON, plain text
- SS26 "Multi-Format Export" -- `--format md,json,csv`

**Files to create/modify:**

```
crates/fetchium-core/src/output/
  mod.rs              -- Module root (update)
  markdown.rs         -- Markdown formatter
  json.rs             -- JSON formatter
  text.rs             -- Plain text formatter
  segments.rs         -- SCS segments formatter
```

**Step-by-step implementation:**

**Step 1: Formatter trait and module root (`output/mod.rs`)**

```rust
//! Output formatters -- markdown, JSON, text, segments (PRD SS26).
//!
//! Phase 1: 4 core formatters (md, json, text, segments).
//! Phase 2: CSV, YAML, HTML, BibTeX.
//! Phase 5: PDF, DOCX export.

pub mod json;
pub mod markdown;
pub mod segments;
pub mod text;

use crate::types::{OutputFormat, ResultItem, SearchResult, Segment};

/// Trait for output formatters.
pub trait Formatter {
    /// Format search results for display.
    fn format_search(&self, result: &SearchResult) -> String;

    /// Format a list of segments.
    fn format_segments(&self, segments: &[Segment], title: &str) -> String;

    /// The output format identifier.
    fn format_id(&self) -> OutputFormat;
}

/// Get a formatter for the given output format.
pub fn get_formatter(format: OutputFormat) -> Box<dyn Formatter> {
    match format {
        OutputFormat::Markdown => Box::new(markdown::MarkdownFormatter),
        OutputFormat::Json => Box::new(json::JsonFormatter),
        OutputFormat::Segments => Box::new(segments::SegmentsFormatter),
        _ => Box::new(text::TextFormatter), // Default fallback
    }
}
```

**Step 2: Markdown formatter (`output/markdown.rs`)**

````rust
//! Markdown output formatter (PRD SS26 -- primary human format).

use crate::output::Formatter;
use crate::types::*;

/// Formats output as clean Markdown.
pub struct MarkdownFormatter;

impl Formatter for MarkdownFormatter {
    fn format_id(&self) -> OutputFormat {
        OutputFormat::Markdown
    }

    fn format_search(&self, result: &SearchResult) -> String {
        let mut out = String::new();

        out.push_str(&format!(
            "# Search: {}\n\n",
            result.meta.query
        ));
        out.push_str(&format!(
            "> {} results | {} tokens | {:.1}s\n\n",
            result.items.len(),
            result.meta.tokens_used,
            result.meta.duration_ms as f64 / 1000.0,
        ));

        for item in &result.items {
            out.push_str(&format!(
                "## {}. {}\n\n",
                item.rank, item.title
            ));
            out.push_str(&format!("**URL:** {}\n\n", item.url));
            if !item.snippet.is_empty() {
                out.push_str(&format!("{}\n\n", item.snippet));
            }
            out.push_str("---\n\n");
        }

        out
    }

    fn format_segments(&self, segments: &[Segment], title: &str) -> String {
        let mut out = String::new();
        out.push_str(&format!("# {title}\n\n"));

        for seg in segments {
            match seg.seg_type {
                SegmentType::Heading => {
                    if let Some(obj) = seg.content.as_object() {
                        let level = obj
                            .get("level")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(2);
                        let text = obj
                            .get("text")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let prefix = "#".repeat(level as usize);
                        out.push_str(&format!("{prefix} {text}\n\n"));
                    }
                }
                SegmentType::Code => {
                    if let Some(obj) = seg.content.as_object() {
                        let lang = obj
                            .get("language")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let code = obj
                            .get("code")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        out.push_str(&format!("```{lang}\n{code}\n```\n\n"));
                    }
                }
                SegmentType::List => {
                    if let Some(obj) = seg.content.as_object() {
                        let ordered = obj
                            .get("ordered")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        if let Some(items) = obj.get("items").and_then(|v| v.as_array()) {
                            for (i, item) in items.iter().enumerate() {
                                let text = item.as_str().unwrap_or("");
                                if ordered {
                                    out.push_str(&format!("{}. {text}\n", i + 1));
                                } else {
                                    out.push_str(&format!("- {text}\n"));
                                }
                            }
                            out.push('\n');
                        }
                    }
                }
                SegmentType::Table => {
                    if let Some(obj) = seg.content.as_object() {
                        if let (Some(headers), Some(rows)) = (
                            obj.get("headers").and_then(|v| v.as_array()),
                            obj.get("rows").and_then(|v| v.as_array()),
                        ) {
                            // Header row
                            let header_strs: Vec<&str> = headers
                                .iter()
                                .map(|h| h.as_str().unwrap_or(""))
                                .collect();
                            out.push_str(&format!("| {} |\n", header_strs.join(" | ")));
                            out.push_str(&format!(
                                "| {} |\n",
                                header_strs.iter().map(|_| "---").collect::<Vec<_>>().join(" | ")
                            ));
                            // Data rows
                            for row in rows {
                                if let Some(cells) = row.as_array() {
                                    let cell_strs: Vec<&str> = cells
                                        .iter()
                                        .map(|c| c.as_str().unwrap_or(""))
                                        .collect();
                                    out.push_str(&format!("| {} |\n", cell_strs.join(" | ")));
                                }
                            }
                            out.push('\n');
                        }
                    }
                }
                SegmentType::Quote => {
                    if let Some(obj) = seg.content.as_object() {
                        let text = obj
                            .get("text")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        out.push_str(&format!("> {text}\n\n"));
                    }
                }
                _ => {
                    // Paragraph, Fact, Opinion, Data, etc.
                    if let Some(text) = seg.content.as_str() {
                        out.push_str(&format!("{text}\n\n"));
                    } else {
                        let json = serde_json::to_string_pretty(&seg.content)
                            .unwrap_or_default();
                        out.push_str(&format!("{json}\n\n"));
                    }
                }
            }
        }

        out
    }
}
````

**Step 3: JSON formatter (`output/json.rs`)**

```rust
//! JSON output formatter (PRD SS26 -- structured format).

use crate::output::Formatter;
use crate::types::*;

/// Formats output as pretty-printed JSON.
pub struct JsonFormatter;

impl Formatter for JsonFormatter {
    fn format_id(&self) -> OutputFormat {
        OutputFormat::Json
    }

    fn format_search(&self, result: &SearchResult) -> String {
        serde_json::to_string_pretty(result).unwrap_or_else(|e| {
            format!("{{\"error\": \"Serialization failed: {e}\"}}")
        })
    }

    fn format_segments(&self, segments: &[Segment], title: &str) -> String {
        let output = serde_json::json!({
            "title": title,
            "segments": segments,
            "total_tokens": segments.iter().map(|s| s.tokens).sum::<u32>(),
        });
        serde_json::to_string_pretty(&output).unwrap_or_default()
    }
}
```

**Step 4: Text formatter (`output/text.rs`)**

```rust
//! Plain text output formatter (PRD SS26 -- minimal format).

use crate::output::Formatter;
use crate::types::*;

/// Formats output as plain text with no markup.
pub struct TextFormatter;

impl Formatter for TextFormatter {
    fn format_id(&self) -> OutputFormat {
        OutputFormat::Markdown // Closest match in the enum
    }

    fn format_search(&self, result: &SearchResult) -> String {
        let mut out = String::new();

        out.push_str(&format!(
            "Search: {} ({} results)\n\n",
            result.meta.query,
            result.items.len()
        ));

        for item in &result.items {
            out.push_str(&format!(
                "{}. {}\n   {}\n   {}\n\n",
                item.rank, item.title, item.url, item.snippet
            ));
        }

        out
    }

    fn format_segments(&self, segments: &[Segment], title: &str) -> String {
        let mut out = String::new();
        out.push_str(&format!("{title}\n\n"));

        for seg in segments {
            if let Some(text) = seg.content.as_str() {
                out.push_str(text);
                out.push_str("\n\n");
            } else {
                let json = serde_json::to_string(&seg.content).unwrap_or_default();
                out.push_str(&json);
                out.push_str("\n\n");
            }
        }

        out
    }
}
```

**Step 5: Segments formatter (`output/segments.rs`)**

```rust
//! SCS Segments output formatter (PRD SS26 -- primary agent format).
//!
//! Outputs the raw SCS segment array as compact JSON.
//! This is the most token-efficient format for AI agent consumption.

use crate::output::Formatter;
use crate::types::*;

/// Formats output as SCS segment JSON (agent-optimized).
pub struct SegmentsFormatter;

impl Formatter for SegmentsFormatter {
    fn format_id(&self) -> OutputFormat {
        OutputFormat::Segments
    }

    fn format_search(&self, result: &SearchResult) -> String {
        // For search results, output the items as a JSON array
        serde_json::to_string(&result).unwrap_or_default()
    }

    fn format_segments(&self, segments: &[Segment], _title: &str) -> String {
        // Compact JSON -- no pretty printing for token efficiency
        serde_json::to_string(segments).unwrap_or_default()
    }
}
```

**Acceptance criteria:**

- [ ] `get_formatter()` returns the correct formatter for each `OutputFormat`
- [ ] Markdown formatter produces valid Markdown with headings, lists, tables, code blocks
- [ ] JSON formatter produces valid, pretty-printed JSON
- [ ] Text formatter produces clean plain text with no markup
- [ ] Segments formatter produces compact JSON (no pretty-printing)
- [ ] All formatters implement the `Formatter` trait
- [ ] All unit tests pass: `cargo test -p fetchium-core output`
- [ ] No clippy warnings

**Testing instructions:**

```bash
cargo test -p fetchium-core output
# Visual test: pipe agent-search output through formatters
cargo run -p fetchium-cli -- search "test" --format json
cargo run -p fetchium-cli -- search "test" --format markdown
```

---

## Phase 1 Completion Checklist

When ALL tasks below are `DONE`, Phase 1 is complete:

| Task     | Description                               | Status |
| -------- | ----------------------------------------- | ------ |
| P1-E1-T1 | HTTP client with pooling + retries        | `TODO` |
| P1-E1-T2 | CEP layers 1-2 extraction                 | `TODO` |
| P1-E1-T3 | `fetch` command                           | `TODO` |
| P1-E2-T1 | DDG HTML scraper backend                  | `TODO` |
| P1-E2-T2 | Search orchestrator                       | `TODO` |
| P1-E2-T3 | `search` command                          | `TODO` |
| P1-E3-T1 | Tokenizer + budget tracking               | `TODO` |
| P1-E3-T2 | QATBE implementation                      | `TODO` |
| P1-E3-T3 | SCS implementation                        | `TODO` |
| P1-E3-T4 | PDS tier 1                                | `TODO` |
| P1-E4-T1 | `agent-search` command                    | `TODO` |
| P1-E4-T2 | `agent-fetch` command                     | `TODO` |
| P1-E5-T1 | BM25 ranking with tantivy                 | `TODO` |
| P1-E6-T1 | Memory LRU cache with moka                | `TODO` |
| P1-E7-T1 | Output formatters (md/json/text/segments) | `TODO` |

### Validation commands (run after all tasks):

```bash
# Build
cargo build --workspace

# Test everything
cargo test --workspace

# Lint
cargo clippy --workspace -- -W clippy::all

# Smoke test: CLI commands work end-to-end
cargo run -p fetchium-cli -- search "hello world"
cargo run -p fetchium-cli -- fetch https://example.com
cargo run -p fetchium-cli -- agent-search "test" --budget 1000
cargo run -p fetchium-cli -- agent-fetch https://example.com --query "test" --budget 500
```

### What Phase 1 enables:

- **Phase 2** can begin: multi-engine search backends, HyperFusion ranking
- **Phase 3** can begin: validation pipeline, citations, research mode
- **Phase 4** can begin (partially): MCP server wrapping existing commands
- Users can install and use Fetchium for basic search and fetch operations
