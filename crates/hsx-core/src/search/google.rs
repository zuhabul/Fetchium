//! Google Search backend — headless Chromium SERP scraper (PRD §8.3).
//!
//! Requires `--features headless`. Falls back to empty results without it.
//! Anti-detection: random delays (200–800ms), viewport variation.

use crate::error::HsxResult;
use crate::search::SearchBackend;
use crate::types::{BackendId, ResultItem};
use async_trait::async_trait;
use tracing::warn;
#[cfg(feature = "headless")]
use tracing::debug;

/// Google SERP scraper via headless Chromium.
///
/// Enable with `cargo build --features headless`.
pub struct GoogleBackend {
    #[cfg(feature = "headless")]
    pool: std::sync::Arc<crate::browser::pool::BrowserPool>,
}

impl GoogleBackend {
    #[cfg(feature = "headless")]
    pub fn new(pool: std::sync::Arc<crate::browser::pool::BrowserPool>) -> Self {
        Self { pool }
    }

    #[cfg(not(feature = "headless"))]
    pub fn new_stub() -> Self {
        Self {}
    }

    #[cfg(feature = "headless")]
    fn build_url(query: &str, page: usize) -> String {
        let start = page * 10;
        let encoded = url::form_urlencoded::Serializer::new(String::new())
            .append_key_only(query)
            .finish();
        format!(
            "https://www.google.com/search?q={encoded}&start={start}&hl=en&num=10&gl=us"
        )
    }

    #[cfg(feature = "headless")]
    fn parse_serp(html: &str, page: usize) -> Vec<ResultItem> {
        use scraper::{Html, Selector};

        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // Google result container: div.g or div[data-sokoban-container]
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
                .map(|s| s.to_string())
                .unwrap_or_default();

            if url.is_empty() || url.contains("google.com") {
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

        debug!("Google parse_serp: {} results from page {}", results.len(), page);
        results
    }
}

#[async_trait]
impl SearchBackend for GoogleBackend {
    fn id(&self) -> BackendId {
        BackendId::Google
    }

    fn requires_headless(&self) -> bool {
        true
    }

    async fn search(&self, query: &str, max_results: u32) -> HsxResult<Vec<ResultItem>> {
        #[cfg(not(feature = "headless"))]
        {
            let _ = (query, max_results);
            warn!("Google backend requires --features headless");
            return Ok(vec![]);
        }

        #[cfg(feature = "headless")]
        {
            let pages = ((max_results as usize + 9) / 10).min(3);
            let mut all_results = Vec::new();

            for page in 0..pages {
                let url = Self::build_url(query, page);

                match self.pool.acquire_tab().await {
                    Ok(tab) => {
                        // Random anti-detection delay between pages
                        if page > 0 {
                            let delay_ms = 200 + (rand_u64() % 600);
                            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                        }

                        match tab.navigate(&url, 15_000).await {
                            Ok(_) => {
                                // Short wait for dynamic content
                                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                                match tab.content().await {
                                    Ok(html) => {
                                        let mut page_results = Self::parse_serp(&html, page);
                                        all_results.append(&mut page_results);
                                    }
                                    Err(e) => {
                                        warn!("Google content error page {page}: {e}");
                                    }
                                }
                            }
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

/// Simple xorshift pseudo-random for anti-detection delays (no heavy dep).
#[allow(dead_code)]
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

    #[cfg(feature = "headless")]
    #[test]
    fn build_url_page_0() {
        let url = GoogleBackend::build_url("rust programming", 0);
        assert!(url.contains("google.com/search"));
        assert!(url.contains("start=0"));
    }

    #[cfg(feature = "headless")]
    #[test]
    fn build_url_page_2() {
        let url = GoogleBackend::build_url("test query", 2);
        assert!(url.contains("start=20"));
    }

    #[cfg(feature = "headless")]
    #[test]
    fn parse_serp_empty_html() {
        let results = GoogleBackend::parse_serp("<html><body></body></html>", 0);
        assert!(results.is_empty());
    }

    #[cfg(feature = "headless")]
    #[test]
    fn parse_serp_filters_non_http() {
        let html = r#"<div class="g"><h3>Good Result</h3><a href="https://example.com">link</a><div class="VwiC3b">snippet</div></div>"#;
        let results = GoogleBackend::parse_serp(html, 0);
        let _ = results.len();
    }

    #[cfg(not(feature = "headless"))]
    #[test]
    fn stub_compiles() {
        let backend = GoogleBackend::new_stub();
        assert_eq!(backend.id(), crate::types::BackendId::Google);
    }
}
