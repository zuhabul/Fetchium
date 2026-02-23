//! DuckDuckGo search backend — scrapes html.duckduckgo.com (PRD §15).
//!
//! Uses the lite HTML interface at html.duckduckgo.com which:
//! - Returns clean, parseable HTML
//! - Requires no API keys
//! - Has no aggressive bot detection
//! - Uses POST form submission for queries
//!
//! CSS selectors (verified against current DDG HTML output):
//! - Results container: div.result, div.results_links_deep
//! - Title + link: a.result__a
//! - Display URL: span.result__url, a.result__url
//! - Snippet: a.result__snippet

use crate::error::{HsxError, HsxResult};
use crate::http::HttpClient;
use crate::search::SearchBackend;
use crate::types::{BackendId, ResultItem};
use async_trait::async_trait;
use scraper::{Html, Selector};
use tracing::{debug, info, warn};

/// DuckDuckGo HTML endpoint (lite version, no JS required).
const DDG_HTML_URL: &str = "https://html.duckduckgo.com/html/";

/// DuckDuckGo search backend.
pub struct DuckDuckGoBackend {
    client: HttpClient,
}

impl DuckDuckGoBackend {
    /// Create a new DDG backend with the given HTTP client.
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }

    /// Parse DDG HTML response into ResultItems.
    ///
    /// DDG HTML structure (current as of 2025):
    /// ```html
    /// <div class="result results_links results_links_deep web-result">
    ///   <div class="result__body">
    ///     <h2 class="result__title">
    ///       <a class="result__a" href="REDIRECT_URL">Title Text</a>
    ///     </h2>
    ///     <div class="result__extras">
    ///       <span class="result__url">display.url.com</span>
    ///     </div>
    ///     <a class="result__snippet">Snippet text...</a>
    ///   </div>
    /// </div>
    /// ```
    fn parse_results(&self, html: &str, max_results: u32) -> Vec<ResultItem> {
        let document = Html::parse_document(html);

        // Primary selectors for result containers
        let result_sel = Selector::parse("div.result").expect("valid");
        // Title link selector (contains the title text and redirect href)
        let title_sel = Selector::parse("a.result__a, h2.result__title a").expect("valid");
        // Snippet selector
        let snippet_sel = Selector::parse("a.result__snippet, .result__snippet").expect("valid");
        // Display URL (used as fallback if href is a redirect)
        let url_sel = Selector::parse("span.result__url, a.result__url").expect("valid");

        let mut results = Vec::new();
        let mut rank = 0u32;

        for element in document.select(&result_sel) {
            if results.len() >= max_results as usize {
                break;
            }

            // Skip ads and special result types
            let classes = element.value().attr("class").unwrap_or("");
            if classes.contains("result--ad")
                || classes.contains("result--more")
                || classes.contains("result--no-results")
            {
                continue;
            }

            // Extract title and href
            let Some(link_el) = element.select(&title_sel).next() else {
                continue;
            };
            let title = link_el.text().collect::<String>().trim().to_string();
            if title.is_empty() {
                continue;
            }

            let href = link_el.value().attr("href").unwrap_or("").to_string();

            // Resolve the real URL (DDG wraps links in a redirect)
            let url = if !href.is_empty() {
                Self::resolve_url(&href)
            } else {
                // Try display URL as fallback
                element
                    .select(&url_sel)
                    .next()
                    .map(|el| {
                        let text = el.text().collect::<String>().trim().to_string();
                        if text.starts_with("http") {
                            text
                        } else {
                            format!("https://{text}")
                        }
                    })
                    .unwrap_or_default()
            };

            if url.is_empty() {
                debug!("DDG: skipping result with empty URL (title: {title})");
                continue;
            }

            // Extract snippet
            let snippet = element
                .select(&snippet_sel)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            rank += 1;
            results.push(ResultItem {
                title,
                url,
                snippet,
                rank,
                backend: BackendId::DuckDuckGo,
                score: None,
                published_date: None,
            });
        }

        results
    }

    /// Resolve a DDG redirect URL to the actual destination URL.
    ///
    /// DDG wraps outbound links in one of these forms:
    /// 1. Direct:  `https://www.example.com/`
    /// 2. Redirect: `//duckduckgo.com/l/?uddg=ENCODED_URL&rut=hash`
    /// 3. Protocol-relative: `//example.com/page`
    fn resolve_url(href: &str) -> String {
        if href.is_empty() {
            return String::new();
        }

        // Form 2: DDG redirect with uddg= parameter
        if let Some(pos) = href.find("uddg=") {
            let encoded = &href[pos + 5..];
            let end = encoded.find('&').unwrap_or(encoded.len());
            let encoded_url = &encoded[..end];
            if let Ok(decoded) = percent_decode(encoded_url) {
                if decoded.starts_with("http") {
                    return decoded;
                }
            }
        }

        // Form 1: already a full https/http URL
        if href.starts_with("https://") || href.starts_with("http://") {
            // Skip DDG-internal links
            if href.contains("duckduckgo.com/l/") {
                return String::new();
            }
            return href.to_string();
        }

        // Form 3: protocol-relative
        if href.starts_with("//") {
            // Skip DDG-internal protocol-relative links
            if href.starts_with("//duckduckgo.com") {
                return String::new();
            }
            return format!("https:{href}");
        }

        String::new()
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

    async fn search(&self, query: &str, max_results: u32) -> HsxResult<Vec<ResultItem>> {
        info!("DDG search: query={query:?}, max={max_results}");

        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Use POST form submission (more stable than GET for DDG)
        let form: &[(&str, &str)] = &[
            ("q", query),
            ("b", ""),      // page marker (empty = page 1)
            ("kl", ""),     // locale (empty = default)
            ("df", ""),     // date filter (empty = any)
        ];

        let response = self
            .client
            .client()
            .post(DDG_HTML_URL)
            .form(form)
            .header("Accept", "text/html,application/xhtml+xml")
            .header("Accept-Language", "en-US,en;q=0.9")
            .send()
            .await
            .map_err(HsxError::Network)?;

        let status = response.status();
        if !status.is_success() {
            return Err(HsxError::Search(format!(
                "DuckDuckGo returned HTTP {status} for query {query:?}"
            )));
        }

        let html = response.text().await.map_err(HsxError::Network)?;

        if html.trim().is_empty() {
            warn!("DDG: empty response body for query {query:?}");
            return Ok(Vec::new());
        }

        let results = self.parse_results(&html, max_results);

        if results.is_empty() {
            warn!("DDG: no results parsed for query {query:?} (HTML len={})", html.len());
        } else {
            info!("DDG: parsed {} results for {query:?}", results.len());
            debug!(
                "DDG top results: {:?}",
                results.iter().take(3).map(|r| &r.title).collect::<Vec<_>>()
            );
        }

        Ok(results)
    }
}

/// Percent-decode a URL-encoded string.
fn percent_decode(input: &str) -> Result<String, ()> {
    let mut bytes = Vec::with_capacity(input.len());
    let mut chars = input.bytes().peekable();
    while let Some(b) = chars.next() {
        match b {
            b'%' => {
                let hi = chars.next().ok_or(())?;
                let lo = chars.next().ok_or(())?;
                let hex = [hi, lo];
                let hex_str = std::str::from_utf8(&hex).map_err(|_| ())?;
                let val = u8::from_str_radix(hex_str, 16).map_err(|_| ())?;
                bytes.push(val);
            }
            b'+' => bytes.push(b' '),
            _ => bytes.push(b),
        }
    }
    String::from_utf8(bytes).map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::HsxConfig;

    fn make_backend() -> DuckDuckGoBackend {
        let config = HsxConfig::default();
        let client = HttpClient::new(&config).expect("client");
        DuckDuckGoBackend::new(client)
    }

    #[test]
    fn parse_standard_results() {
        let html = r#"
        <html><body>
        <div class="result results_links results_links_deep web-result">
            <div class="result__body">
                <h2 class="result__title">
                    <a class="result__a" href="https://www.rust-lang.org/">Rust Programming Language</a>
                </h2>
                <div class="result__extras">
                    <span class="result__url">www.rust-lang.org</span>
                </div>
                <a class="result__snippet">A language empowering everyone to build reliable and efficient software.</a>
            </div>
        </div>
        <div class="result results_links results_links_deep web-result">
            <div class="result__body">
                <h2 class="result__title">
                    <a class="result__a" href="https://doc.rust-lang.org/book/">The Rust Book</a>
                </h2>
                <a class="result__snippet">The official Rust programming language book.</a>
            </div>
        </div>
        </body></html>
        "#;
        let backend = make_backend();
        let results = backend.parse_results(html, 10);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Rust Programming Language");
        assert_eq!(results[0].url, "https://www.rust-lang.org/");
        assert_eq!(results[0].rank, 1);
        assert_eq!(results[0].backend, BackendId::DuckDuckGo);
        assert!(results[0].snippet.contains("reliable"));
        assert_eq!(results[1].rank, 2);
        assert_eq!(results[1].title, "The Rust Book");
    }

    #[test]
    fn parse_redirect_url() {
        let href = "//duckduckgo.com/l/?uddg=https%3A%2F%2Fwww.rust-lang.org%2F&rut=abc123";
        let url = DuckDuckGoBackend::resolve_url(href);
        assert_eq!(url, "https://www.rust-lang.org/");
    }

    #[test]
    fn resolve_direct_url() {
        let href = "https://example.com/page";
        assert_eq!(DuckDuckGoBackend::resolve_url(href), href);
    }

    #[test]
    fn resolve_protocol_relative_url() {
        let href = "//example.com/page";
        assert_eq!(DuckDuckGoBackend::resolve_url(href), "https://example.com/page");
    }

    #[test]
    fn skip_ddg_internal_links() {
        let href = "//duckduckgo.com/privacy";
        assert!(DuckDuckGoBackend::resolve_url(href).is_empty());
    }

    #[test]
    fn skip_ad_results() {
        let html = r#"
        <html><body>
        <div class="result result--ad">
            <a class="result__a" href="https://ad.example.com/">Ad Result</a>
        </div>
        <div class="result results_links_deep web-result">
            <h2 class="result__title">
                <a class="result__a" href="https://real.example.com/">Real Result</a>
            </h2>
            <a class="result__snippet">A real search result.</a>
        </div>
        </body></html>
        "#;
        let backend = make_backend();
        let results = backend.parse_results(html, 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Real Result");
    }

    #[test]
    fn max_results_respected() {
        let html = (0..20)
            .map(|i| format!(
                r#"<div class="result results_links_deep web-result">
                    <a class="result__a" href="https://result{i}.com/">Result {i}</a>
                    <a class="result__snippet">Snippet {i}</a>
                </div>"#
            ))
            .collect::<String>();
        let html = format!("<html><body>{html}</body></html>");
        let backend = make_backend();
        let results = backend.parse_results(&html, 5);
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn percent_decode_works() {
        assert_eq!(
            percent_decode("https%3A%2F%2Fwww.example.com%2F"),
            Ok("https://www.example.com/".to_string())
        );
        assert_eq!(percent_decode("hello+world"), Ok("hello world".to_string()));
    }
}
