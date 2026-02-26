//! Google Search backend — HTTP scraper (default) + headless Chromium (--features headless).
//!
//! ## Default mode (no extra flags)
//! Uses `reqwest` with browser-realistic headers to scrape Google's standard
//! search endpoint. Works without any API keys or browser downloads. Google
//! occasionally serves CAPTCHA pages — the backend detects them and returns
//! empty results gracefully so the orchestrator falls back to other backends.
//!
//! ## Headless mode (`--features headless`)
//! Uses `chromiumoxide` BrowserPool for full JS rendering. Higher fidelity
//! but requires compiling Chromium bindings (~5 min) and a Chrome binary.
//!
//! ## Anti-detection
//! Chrome/121 User-Agent, Accept-Language, Referer headers; 500ms inter-page
//! delay; CAPTCHA detection on first-page empty or block-page markers.

use crate::error::HsxResult;
use crate::search::SearchBackend;
use crate::types::{BackendId, ResultItem};
use async_trait::async_trait;
#[cfg(feature = "headless")]
use tracing::debug;
use tracing::warn;

/// Browser User-Agent matching Chrome 121 on Windows — reduces bot-detection.
#[cfg(not(feature = "headless"))]
const BROWSER_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
    AppleWebKit/537.36 (KHTML, like Gecko) \
    Chrome/121.0.0.0 Safari/537.36";

/// Google SERP scraper.
///
/// Without `--features headless`: lightweight HTTP scraper, no extra setup.
/// With `--features headless`: headless Chromium for JS-rendered results.
pub struct GoogleBackend {
    #[cfg(feature = "headless")]
    pool: std::sync::Arc<crate::browser::pool::BrowserPool>,
    #[cfg(not(feature = "headless"))]
    http: crate::http::HttpClient,
}

impl GoogleBackend {
    /// Create a headless-Chromium backend (requires `--features headless`).
    #[cfg(feature = "headless")]
    pub fn new(pool: std::sync::Arc<crate::browser::pool::BrowserPool>) -> Self {
        Self { pool }
    }

    /// Create an HTTP-scraper backend (default, no extra features required).
    #[cfg(not(feature = "headless"))]
    pub fn new_http(http: crate::http::HttpClient) -> Self {
        Self { http }
    }

    /// Build a Google search URL for the given query and page (0-indexed).
    fn build_url(query: &str, page: usize) -> String {
        let start = page * 10;
        let encoded = url::form_urlencoded::Serializer::new(String::new())
            .append_key_only(query)
            .finish();
        // pws=0  — disable personalised results for consistent scraping
        format!("https://www.google.com/search?q={encoded}&start={start}&hl=en&num=10&gl=us&pws=0")
    }

    /// Parse a Google SERP HTML page into [`ResultItem`]s.
    ///
    /// Returns an empty vec on CAPTCHA / block pages — the caller should warn
    /// and return early rather than trying subsequent pages.
    fn parse_serp(html: &str, page: usize) -> Vec<ResultItem> {
        use scraper::{Html, Selector};

        // ── CAPTCHA / block detection ────────────────────────────────────────
        // Google serves several kinds of block pages:
        // 1. Redirects to `accounts.google.com` (sign-in wall)
        // 2. Inline CAPTCHA challenge with text "detected unusual traffic"
        // 3. "please enable cookies" / JS-only pages
        if html.contains("accounts.google.com")
            || html.contains("detected unusual traffic")
            || html.contains("please enable cookies")
            || (html.len() < 5_000 && html.to_lowercase().contains("captcha"))
        {
            return vec![];
        }

        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // Google's SERP DOM (stable across most Google updates):
        //  <div class="g">                   ← organic result wrapper
        //    <div class="yuRUbf">
        //      <a href="https://...">
        //        <h3 class="LC20lb">Title</h3>
        //      </a>
        //    </div>
        //    <div class="VwiC3b">Snippet</div>
        //  </div>
        let container_sel = Selector::parse("div.g, div[data-hveid]").unwrap();
        let title_sel = Selector::parse("h3").unwrap();
        let link_sel = Selector::parse("a[href]").unwrap();
        let snippet_sel = Selector::parse("div.VwiC3b, span.aCOpRe, div[data-sncf]").unwrap();

        let mut rank = (page * 10 + 1) as u32;

        for container in document.select(&container_sel) {
            let title = container
                .select(&title_sel)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            if title.is_empty() {
                continue;
            }

            let url = container
                .select(&link_sel)
                .next()
                .and_then(|e| e.value().attr("href"))
                .filter(|href| href.starts_with("http"))
                .filter(|href| !href.contains("google.com"))
                .map(|s| s.to_string())
                .unwrap_or_default();

            if url.is_empty() {
                continue;
            }

            let snippet = container
                .select(&snippet_sel)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            results.push(ResultItem {
                title,
                url,
                snippet,
                rank,
                backend: BackendId::Google,
                score: None,
                published_date: None,
            });
            rank += 1;
        }

        #[cfg(feature = "headless")]
        debug!(
            "Google parse_serp: {} results from page {}",
            results.len(),
            page
        );

        results
    }
}

#[async_trait]
impl SearchBackend for GoogleBackend {
    fn id(&self) -> BackendId {
        BackendId::Google
    }

    fn requires_headless(&self) -> bool {
        cfg!(feature = "headless")
    }

    async fn search(&self, query: &str, max_results: u32) -> HsxResult<Vec<ResultItem>> {
        // ── Non-headless path: lightweight HTTP scraper ───────────────────────
        #[cfg(not(feature = "headless"))]
        {
            // Limit to 2 pages to reduce CAPTCHA risk; 20 results is usually plenty.
            let pages = (max_results as usize).div_ceil(10).min(2);
            let mut all_results = Vec::new();

            for page in 0..pages {
                let url = Self::build_url(query, page);

                match self
                    .http
                    .client()
                    .get(&url)
                    .header("User-Agent", BROWSER_UA)
                    .header(
                        "Accept",
                        "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
                    )
                    .header("Accept-Language", "en-US,en;q=0.9")
                    .header("Referer", "https://www.google.com/")
                    .header("Cache-Control", "no-cache")
                    .send()
                    .await
                {
                    Ok(resp) if resp.status().is_success() => match resp.text().await {
                        Ok(html) => {
                            let page_results = Self::parse_serp(&html, page);
                            if page_results.is_empty() && page == 0 {
                                warn!(
                                    "Google: 0 results (CAPTCHA or bot detection) for {:?}",
                                    query
                                );
                                break;
                            }
                            all_results.extend(page_results);
                        }
                        Err(e) => {
                            warn!("Google: body read error for {query:?}: {e}");
                            break;
                        }
                    },
                    Ok(resp) => {
                        warn!("Google: HTTP {} for {query:?}", resp.status());
                        break;
                    }
                    Err(e) => {
                        warn!("Google: request error for {query:?}: {e}");
                        break;
                    }
                }

                // Brief inter-page delay to reduce bot-detection risk.
                if page + 1 < pages {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            }

            return Ok(all_results.into_iter().take(max_results as usize).collect());
        }

        // ── Headless path: chromiumoxide BrowserPool ─────────────────────────
        #[cfg(feature = "headless")]
        {
            let pages = (max_results as usize).div_ceil(10).min(3);
            let mut all_results = Vec::new();

            for page in 0..pages {
                let url = Self::build_url(query, page);

                match self.pool.acquire_tab().await {
                    Ok(tab) => {
                        if page > 0 {
                            let delay_ms = 200 + (rand_u64() % 600);
                            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                        }

                        // Wait for Google organic results to render (JS-heavy SERP)
                        match tab.navigate_and_wait(&url, "div.g", 15_000).await {
                            Ok(_) => match tab.content().await {
                                Ok(html) => {
                                    all_results.extend(Self::parse_serp(&html, page));
                                }
                                Err(e) => {
                                    warn!("Google content error page {page}: {e}");
                                }
                            },
                            Err(e) => {
                                warn!("Google navigate error page {page}: {e}");
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Google tab acquire failed: {e}");
                        break;
                    }
                }
            }

            Ok(all_results.into_iter().take(max_results as usize).collect())
        }
    }
}

/// Xorshift pseudo-random for anti-detection delays (no heavy dep).
#[cfg(feature = "headless")]
fn rand_u64() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let mut x = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "headless"))]
    fn make_backend() -> GoogleBackend {
        use crate::config::HsxConfig;
        use crate::http::HttpClient;
        GoogleBackend::new_http(HttpClient::new(&HsxConfig::default()).expect("http"))
    }

    #[cfg(not(feature = "headless"))]
    #[test]
    fn backend_id() {
        assert_eq!(make_backend().id(), BackendId::Google);
    }

    #[cfg(not(feature = "headless"))]
    #[test]
    fn parse_serp_empty_html() {
        let results = GoogleBackend::parse_serp("<html><body></body></html>", 0);
        assert!(results.is_empty());
    }

    #[cfg(not(feature = "headless"))]
    #[test]
    fn parse_serp_captcha_detected() {
        // Simulate a CAPTCHA page — should return empty, not panic.
        let captcha_html = r#"<html><body>
            <p>detected unusual traffic from your computer network</p>
        </body></html>"#;
        let results = GoogleBackend::parse_serp(captcha_html, 0);
        assert!(results.is_empty());
    }

    #[cfg(not(feature = "headless"))]
    #[test]
    fn parse_serp_filters_non_http_urls() {
        let html = r#"<html><body>
            <div class="g">
                <h3>Good Result</h3>
                <a href="https://example.com/page">link</a>
                <div class="VwiC3b">A useful snippet.</div>
            </div>
            <div class="g">
                <h3>Google Internal</h3>
                <a href="https://www.google.com/search?q=test">internal</a>
            </div>
        </body></html>"#;
        let results = GoogleBackend::parse_serp(html, 0);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "https://example.com/page");
        assert_eq!(results[0].backend, BackendId::Google);
    }

    #[cfg(not(feature = "headless"))]
    #[test]
    fn build_url_page_0() {
        let url = GoogleBackend::build_url("rust programming", 0);
        assert!(url.contains("google.com/search"));
        assert!(url.contains("start=0"));
        assert!(url.contains("rust"));
    }

    #[cfg(not(feature = "headless"))]
    #[test]
    fn build_url_page_1() {
        let url = GoogleBackend::build_url("test", 1);
        assert!(url.contains("start=10"));
    }

    #[cfg(feature = "headless")]
    #[test]
    fn build_url_headless() {
        let url = GoogleBackend::build_url("rust async", 0);
        assert!(url.contains("google.com/search"));
        assert!(url.contains("start=0"));
    }

    #[cfg(feature = "headless")]
    #[test]
    fn parse_serp_empty_headless() {
        let results = GoogleBackend::parse_serp("<html><body></body></html>", 0);
        assert!(results.is_empty());
    }
}
