//! DuckDuckGo search backend — scrapes both the full and lite HTML endpoints.
//!
//! ## Endpoints (tried in order on each search call)
//! 1. `https://html.duckduckgo.com/html/` — full HTML, richer snippets
//! 2. `https://lite.duckduckgo.com/lite/` — lightweight fallback, more
//!    scraper-tolerant; used when the full endpoint returns no results
//!
//! Both require browser-like User-Agent + Referer headers or DDG's CDN
//! returns a bot-detection page (~14KB, no results).
//!
//! ## Selectors
//! Full HTML: `div.result`, `a.result__a`, `a.result__snippet`, `span.result__url`
//! Lite HTML:  `b a[href]` (titles), `td[valign=top]` (snippets)

use crate::error::HsxResult;
use crate::http::HttpClient;
use crate::search::{SearchBackend, SearchContext, TimeRange};
use crate::types::{BackendId, ResultItem};
use async_trait::async_trait;
use scraper::{Html, Selector};
use tracing::{debug, info};

/// Full DDG HTML endpoint.
const DDG_HTML_URL: &str = "https://html.duckduckgo.com/html/";
/// Lite DDG endpoint — simpler HTML, more bot-tolerant.
const DDG_LITE_URL: &str = "https://lite.duckduckgo.com/lite/";
/// Browser User-Agent (required — DDG blocks generic/empty UAs).
const BROWSER_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
    AppleWebKit/537.36 (KHTML, like Gecko) \
    Chrome/121.0.0.0 Safari/537.36";

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

    /// Parse DDG lite HTML response into ResultItems.
    ///
    /// Lite DDG HTML structure:
    /// ```html
    /// <table>
    ///   <tr><td colspan="2"><b><a href="DDG-REDIRECT">Title</a></b></td></tr>
    ///   <tr>
    ///     <td valign="top">&nbsp;</td>
    ///     <td valign="top">Snippet text. <span class="link-text">display.url</span></td>
    ///   </tr>
    ///   <tr><td colspan="2"><hr/></td></tr>
    /// </table>
    /// ```
    fn parse_lite_results(&self, html: &str, max_results: u32) -> Vec<ResultItem> {
        let document = Html::parse_document(html);

        // Title links are inside <b><a href="...">Title</a></b>
        let title_sel = Selector::parse("b a[href]").expect("valid");
        // Snippets: <td valign="top"> with substantial content (filter out &nbsp; padding cells)
        let snippet_sel = Selector::parse("td[valign='top']").expect("valid");

        let title_links: Vec<_> = document.select(&title_sel).collect();
        let snippets: Vec<_> = document
            .select(&snippet_sel)
            .filter(|td| {
                // Filter out the blank padding cells (contain only non-breaking space)
                let text = td.text().collect::<String>();
                let t = text.trim();
                t.len() > 2 && t != "\u{00a0}" // exclude &nbsp; cells
            })
            .collect();

        title_links
            .iter()
            .zip(snippets.iter())
            .take(max_results as usize)
            .enumerate()
            .filter_map(|(i, (link, snippet_td))| {
                let title = link.text().collect::<String>().trim().to_string();
                let href = link.value().attr("href").unwrap_or("").to_string();
                let url = Self::resolve_url(&href);

                if title.is_empty() || url.is_empty() {
                    return None;
                }

                let snippet = snippet_td
                    .text()
                    .collect::<String>()
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");

                Some(ResultItem {
                    title,
                    url,
                    snippet,
                    rank: (i + 1) as u32,
                    backend: BackendId::DuckDuckGo,
                    score: None,
                    published_date: None,
                })
            })
            .collect()
    }

    /// Detect DDG CAPTCHA / bot-detection pages.
    ///
    /// DDG serves a "Select all squares containing a duck" challenge when it
    /// detects bot traffic. The page contains `anomaly-modal` class names and
    /// the text "bots use DuckDuckGo too". HTTP status is typically 202.
    fn is_captcha_page(html: &str) -> bool {
        html.contains("anomaly-modal")
            || html.contains("bots use DuckDuckGo")
            || html.contains("cc=botnet")
            || (html.contains("challenge-form") && html.contains("anomaly"))
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

    /// Fetch results from the full DDG HTML endpoint.
    async fn try_full(&self, query: &str, max_results: u32, df: &str) -> Vec<ResultItem> {
        let form: &[(&str, &str)] = &[("q", query), ("b", ""), ("s", "0"), ("kl", ""), ("df", df)];

        match self
            .client
            .client_for_domain("duckduckgo.com")
            .post(DDG_HTML_URL)
            .form(form)
            .header("User-Agent", BROWSER_UA)
            .header(
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            )
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Referer", "https://duckduckgo.com/")
            .header("Cache-Control", "no-cache")
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 202 => {
                match resp.text().await {
                    Ok(html) if !html.trim().is_empty() => {
                        if Self::is_captcha_page(&html) {
                            debug!("DDG: CAPTCHA detected on full endpoint for {query:?}");
                            vec![]
                        } else {
                            let r = self.parse_results(&html, max_results);
                            debug!("DDG full HTML: {} results for {query:?}", r.len());
                            r
                        }
                    }
                    _ => vec![],
                }
            }
            Ok(resp) => {
                debug!("DDG full HTML: HTTP {} for {query:?}", resp.status());
                vec![]
            }
            Err(e) => {
                debug!("DDG full HTML request failed: {e}");
                vec![]
            }
        }
    }

    /// Fetch results from the DDG lite endpoint.
    async fn try_lite(&self, query: &str, max_results: u32, df: &str) -> Vec<ResultItem> {
        let lite_form: Vec<(&str, &str)> = if df.is_empty() {
            vec![("q", query), ("s", "0"), ("o", "json"), ("dc", "1")]
        } else {
            vec![
                ("q", query),
                ("s", "0"),
                ("o", "json"),
                ("dc", "1"),
                ("df", df),
            ]
        };

        match self
            .client
            .client_for_domain("duckduckgo.com")
            .post(DDG_LITE_URL)
            .form(&lite_form)
            .header("User-Agent", BROWSER_UA)
            .header(
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            )
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Referer", "https://duckduckgo.com/")
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 202 => {
                match resp.text().await {
                    Ok(html) if !html.trim().is_empty() => {
                        if Self::is_captcha_page(&html) {
                            debug!("DDG: CAPTCHA detected on lite endpoint for {query:?}");
                            vec![]
                        } else {
                            let r = self.parse_lite_results(&html, max_results);
                            debug!("DDG lite: {} results for {query:?}", r.len());
                            r
                        }
                    }
                    _ => vec![],
                }
            }
            Ok(resp) => {
                debug!("DDG lite: HTTP {} for {query:?}", resp.status());
                vec![]
            }
            Err(e) => {
                debug!("DDG lite request failed: {e}");
                vec![]
            }
        }
    }
}

impl DuckDuckGoBackend {
    fn time_range_to_df(tr: Option<TimeRange>) -> &'static str {
        match tr {
            Some(TimeRange::Day) => "d",
            Some(TimeRange::Week) => "w",
            Some(TimeRange::Month) => "m",
            Some(TimeRange::Year) => "y",
            None => "",
        }
    }

    async fn search_inner(
        &self,
        query: &str,
        max_results: u32,
        df: &str,
    ) -> HsxResult<Vec<ResultItem>> {
        info!("DDG search: query={query:?}, max={max_results}, df={df:?}");

        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        let (full, lite) = tokio::join!(
            self.try_full(query, max_results, df),
            self.try_lite(query, max_results, df),
        );

        if !full.is_empty() {
            info!(
                "DDG: {} results via full endpoint for {query:?}",
                full.len()
            );
            Ok(full)
        } else if !lite.is_empty() {
            info!(
                "DDG: {} results via lite endpoint for {query:?}",
                lite.len()
            );
            Ok(lite)
        } else {
            debug!("DDG: no results for {query:?} (CAPTCHA or rate-limited)");
            Ok(Vec::new())
        }
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
        self.search_inner(query, max_results, "").await
    }

    async fn search_with_context(
        &self,
        query: &str,
        max_results: u32,
        ctx: &SearchContext,
    ) -> HsxResult<Vec<ResultItem>> {
        let df = Self::time_range_to_df(ctx.time_range);
        self.search_inner(query, max_results, df).await
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
        assert_eq!(
            DuckDuckGoBackend::resolve_url(href),
            "https://example.com/page"
        );
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
            .map(|i| {
                format!(
                    r#"<div class="result results_links_deep web-result">
                    <a class="result__a" href="https://result{i}.com/">Result {i}</a>
                    <a class="result__snippet">Snippet {i}</a>
                </div>"#
                )
            })
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
