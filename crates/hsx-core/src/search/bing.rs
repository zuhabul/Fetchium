//! Bing Search backend — headless Chromium SERP scraper (PRD §8.3).
//!
//! Requires `--features headless`. Bing is less aggressive with bot detection
//! than Google, making it a reliable headless fallback.

use crate::error::HsxResult;
use crate::search::SearchBackend;
use crate::types::{BackendId, ResultItem};
use async_trait::async_trait;
use tracing::warn;
#[cfg(feature = "headless")]
use tracing::debug;

/// Bing SERP scraper via headless Chromium.
pub struct BingBackend {
    #[cfg(feature = "headless")]
    pool: std::sync::Arc<crate::browser::pool::BrowserPool>,
}

impl BingBackend {
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
        let first = page * 10 + 1;
        let encoded = url::form_urlencoded::Serializer::new(String::new())
            .append_key_only(query)
            .finish();
        format!("https://www.bing.com/search?q={encoded}&first={first}&setlang=en")
    }

    #[cfg(feature = "headless")]
    fn parse_serp(html: &str, page: usize) -> Vec<ResultItem> {
        use scraper::{Html, Selector};

        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // Bing result container: li.b_algo
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

            let url = title_elem
                .value()
                .attr("href")
                .filter(|h| h.starts_with("http"))
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
                backend: BackendId::Bing,
                score: None,
                published_date: None,
            });
            rank += 1;
        }

        debug!("Bing parse_serp: {} results from page {}", results.len(), page);
        results
    }
}

#[async_trait]
impl SearchBackend for BingBackend {
    fn id(&self) -> BackendId {
        BackendId::Bing
    }

    fn requires_headless(&self) -> bool {
        true
    }

    async fn search(&self, query: &str, max_results: u32) -> HsxResult<Vec<ResultItem>> {
        #[cfg(not(feature = "headless"))]
        {
            let _ = (query, max_results);
            warn!("Bing backend requires --features headless");
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
                        if page > 0 {
                            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                        }
                        match tab.navigate(&url, 12_000).await {
                            Ok(_) => {
                                tokio::time::sleep(std::time::Duration::from_millis(400)).await;
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

    #[cfg(feature = "headless")]
    #[test]
    fn build_url_page_0() {
        let url = BingBackend::build_url("rust async", 0);
        assert!(url.contains("bing.com/search"));
        assert!(url.contains("first=1"));
    }

    #[cfg(feature = "headless")]
    #[test]
    fn build_url_page_1() {
        let url = BingBackend::build_url("test", 1);
        assert!(url.contains("first=11"));
    }

    #[cfg(feature = "headless")]
    #[test]
    fn parse_serp_empty_html() {
        let results = BingBackend::parse_serp("<html></html>", 0);
        assert!(results.is_empty());
    }

    #[cfg(not(feature = "headless"))]
    #[test]
    fn stub_compiles() {
        // Verifies the non-headless stub builds and satisfies SearchBackend::id()
        let backend = BingBackend::new_stub();
        assert_eq!(backend.id(), crate::types::BackendId::Bing);
    }
}
