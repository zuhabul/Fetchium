//! Bing Search backend — HTTP scraper (default) + headless Chromium (--features headless).
//!
//! ## Default mode (no extra flags)
//! Bing is significantly more scraper-tolerant than Google. Plain HTTP requests
//! with browser headers reliably return full SERP HTML without CAPTCHA, making
//! Bing a robust fallback when Google bot-detection triggers.
//!
//! ## Headless mode (`--features headless`)
//! Uses `chromiumoxide` BrowserPool for fully JS-rendered results.
//!
//! ## Selectors
//!
//! - `li.b_algo` — organic result container
//! - `li.b_algo h2 a` — title + URL
//!   - Non-headless: direct URLs (starts with https://real-site.com)
//!   - Headless: bing.com/ck/a?...&u=a1BASE64 redirect → decoded via decode_bing_redirect()
//! - `.b_algoSlug` — snippet text

use crate::error::HsxResult;
use crate::search::SearchBackend;
use crate::types::{BackendId, ResultItem};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use tracing::debug;
#[cfg(feature = "headless")]
use tracing::warn;

/// Browser User-Agent — helps avoid bot-detection on Bing's CDN.
#[cfg(not(feature = "headless"))]
const BROWSER_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
    AppleWebKit/537.36 (KHTML, like Gecko) \
    Chrome/121.0.0.0 Safari/537.36";

/// Bing SERP scraper.
///
/// Without `--features headless`: lightweight HTTP scraper, no extra setup.
/// With `--features headless`: headless Chromium for JS-rendered results.
pub struct BingBackend {
    #[cfg(feature = "headless")]
    pool: std::sync::Arc<crate::browser::pool::BrowserPool>,
    #[cfg(not(feature = "headless"))]
    http: crate::http::HttpClient,
}

impl BingBackend {
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

    /// Build a Bing search URL for the given query and page (0-indexed).
    fn build_url(query: &str, page: usize) -> String {
        let first = page * 10 + 1;
        let encoded = url::form_urlencoded::Serializer::new(String::new())
            .append_key_only(query)
            .finish();
        format!("https://www.bing.com/search?q={encoded}&first={first}&setlang=en&cc=US")
    }

    /// Parse Bing SERP HTML into [`ResultItem`]s.
    fn parse_serp(html: &str, page: usize) -> Vec<ResultItem> {
        use scraper::{Html, Selector};

        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // Bing's organic result structure:
        //  <li class="b_algo">
        //    <h2><a href="https://real-url.com/">Title</a></h2>
        //    <div class="b_caption">
        //      <p class="b_algoSlug">Snippet text…</p>
        //    </div>
        //  </li>
        //
        // In headless mode, Bing wraps links via bing.com/ck/a?...&u=a1BASE64URL
        let container_sel = Selector::parse("li.b_algo").unwrap();
        let title_sel = Selector::parse("h2 a").unwrap();
        let snippet_sel = Selector::parse("p.b_algoSlug, .b_caption p, p").unwrap();

        let mut rank = (page * 10 + 1) as u32;

        for container in document.select(&container_sel) {
            let title_elem = match container.select(&title_sel).next() {
                Some(e) => e,
                None => continue,
            };

            let title = title_elem.text().collect::<String>().trim().to_string();
            if title.is_empty() {
                continue;
            }

            let raw_href = title_elem
                .value()
                .attr("href")
                .filter(|h| h.starts_with("http"))
                .unwrap_or_default();

            // Headless Bing wraps destinations in /ck/a?...&u=a1<base64url>
            // Decode the real URL if present, otherwise use the href directly.
            let url = if raw_href.contains("bing.com/ck/a") {
                decode_bing_redirect(raw_href)
            } else if !raw_href.is_empty() && !raw_href.contains("bing.com") {
                raw_href.to_string()
            } else {
                String::new()
            };

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
                backend: BackendId::Bing,
                score: None,
                published_date: None,
            });
            rank += 1;
        }

        debug!(
            "Bing parse_serp: {} results from page {}",
            results.len(),
            page
        );

        results
    }
}

/// Decode Bing's `/ck/a?...&u=a1<base64url>` redirect to the real destination URL.
///
/// Bing headless mode wraps all SERP links through a click-tracker. The `u` query
/// parameter contains `a1` + standard base64 of the real URL.
fn decode_bing_redirect(href: &str) -> String {
    // Extract the `u` query parameter value
    let u_val = href
        .split('&')
        .find(|seg| seg.starts_with("u=a1"))
        .map(|seg| &seg[4..]); // strip "u=a1"

    if let Some(b64) = u_val {
        // URL-safe base64: replace '-' with '+' and '_' with '/'
        let normalized = b64.replace('-', "+").replace('_', "/");
        // Decode and convert to UTF-8
        if let Ok(bytes) = BASE64.decode(normalized.as_bytes()) {
            if let Ok(url) = String::from_utf8(bytes) {
                if url.starts_with("http") {
                    return url;
                }
            }
        }
    }
    String::new()
}

#[async_trait]
impl SearchBackend for BingBackend {
    fn id(&self) -> BackendId {
        BackendId::Bing
    }

    fn requires_headless(&self) -> bool {
        cfg!(feature = "headless")
    }

    async fn search(&self, query: &str, max_results: u32) -> HsxResult<Vec<ResultItem>> {
        // ── Non-headless path: lightweight HTTP scraper ───────────────────────
        #[cfg(not(feature = "headless"))]
        {
            let pages = (max_results as usize).div_ceil(10).min(3);
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
                    .header("Referer", "https://www.bing.com/")
                    .header("Cache-Control", "no-cache")
                    .send()
                    .await
                {
                    Ok(resp) if resp.status().is_success() => match resp.text().await {
                        Ok(html) => {
                            let page_results = Self::parse_serp(&html, page);
                            if page_results.is_empty() && page == 0 {
                                debug!("Bing: 0 results for {:?}", query);
                                break;
                            }
                            all_results.extend(page_results);
                        }
                        Err(e) => {
                            debug!("Bing: body read error: {e}");
                            break;
                        }
                    },
                    Ok(resp) => {
                        debug!("Bing: HTTP {} for {query:?}", resp.status());
                        break;
                    }
                    Err(e) => {
                        debug!("Bing: request error: {e}");
                        break;
                    }
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
                            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                        }
                        // Wait for Bing organic results to render (JS-heavy SERP)
                        match tab.navigate_and_wait(&url, "li.b_algo", 12_000).await {
                            Ok(_) => {
                                if let Ok(html) = tab.content().await {
                                    all_results.extend(Self::parse_serp(&html, page));
                                }
                            }
                            Err(e) => warn!("Bing navigate error page {page}: {e}"),
                        }
                    }
                    Err(e) => {
                        warn!("Bing tab acquire failed: {e}");
                        break;
                    }
                }
            }

            Ok(all_results.into_iter().take(max_results as usize).collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "headless"))]
    fn make_backend() -> BingBackend {
        use crate::config::HsxConfig;
        use crate::http::HttpClient;
        BingBackend::new_http(HttpClient::new(&HsxConfig::default()).expect("http"))
    }

    #[cfg(not(feature = "headless"))]
    #[test]
    fn backend_id() {
        assert_eq!(make_backend().id(), BackendId::Bing);
    }

    #[cfg(not(feature = "headless"))]
    #[test]
    fn build_url_page_0() {
        let url = BingBackend::build_url("rust async", 0);
        assert!(url.contains("bing.com/search"));
        assert!(url.contains("first=1"));
    }

    #[cfg(not(feature = "headless"))]
    #[test]
    fn build_url_page_1() {
        let url = BingBackend::build_url("test", 1);
        assert!(url.contains("first=11"));
    }

    #[cfg(not(feature = "headless"))]
    #[test]
    fn parse_serp_empty_html() {
        let results = BingBackend::parse_serp("<html></html>", 0);
        assert!(results.is_empty());
    }

    #[cfg(not(feature = "headless"))]
    #[test]
    fn parse_serp_real_structure() {
        let html = r#"<html><body>
            <ol id="b_results">
                <li class="b_algo">
                    <h2><a href="https://www.rust-lang.org/">Rust Programming Language</a></h2>
                    <div class="b_caption">
                        <p class="b_algoSlug">A language empowering everyone to build reliable and efficient software.</p>
                    </div>
                </li>
                <li class="b_algo">
                    <h2><a href="https://doc.rust-lang.org/book/">The Rust Book</a></h2>
                    <p class="b_algoSlug">The official Rust programming guide.</p>
                </li>
            </ol>
        </body></html>"#;
        let results = BingBackend::parse_serp(html, 0);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Rust Programming Language");
        assert_eq!(results[0].url, "https://www.rust-lang.org/");
        assert_eq!(results[0].rank, 1);
        assert_eq!(results[0].backend, BackendId::Bing);
        assert_eq!(results[1].rank, 2);
    }

    #[cfg(not(feature = "headless"))]
    #[test]
    fn filters_bing_internal_links() {
        let html = r#"<html><body><ol id="b_results">
            <li class="b_algo">
                <h2><a href="https://www.bing.com/search?q=related">Bing Internal</a></h2>
            </li>
            <li class="b_algo">
                <h2><a href="https://example.com/">External Link</a></h2>
            </li>
        </ol></body></html>"#;
        let results = BingBackend::parse_serp(html, 0);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "https://example.com/");
    }

    #[cfg(feature = "headless")]
    #[test]
    fn build_url_headless() {
        let url = BingBackend::build_url("rust async", 0);
        assert!(url.contains("bing.com/search"));
        assert!(url.contains("first=1"));
    }

    #[cfg(feature = "headless")]
    #[test]
    fn parse_serp_empty_headless() {
        let results = BingBackend::parse_serp("<html></html>", 0);
        assert!(results.is_empty());
    }
}
