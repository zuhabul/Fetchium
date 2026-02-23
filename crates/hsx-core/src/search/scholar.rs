//! Google Scholar backend — headless Chromium SERP scraper (PRD §8.3).
//!
//! Extracts academic metadata: authors, year, citation count, venue, PDF link.
//! Citation counts feed into the `authority_score` of HyperFusion.
//!
//! Requires `--features headless`.

use crate::error::HsxResult;
use crate::search::SearchBackend;
use crate::types::{BackendId, ResultItem};
use async_trait::async_trait;
use tracing::warn;
#[cfg(feature = "headless")]
use tracing::debug;

/// Academic metadata from Google Scholar.
#[derive(Debug, Clone, Default)]
pub struct ScholarMetadata {
    pub authors: Vec<String>,
    pub year: Option<u16>,
    pub citation_count: Option<u32>,
    pub venue: Option<String>,
    pub pdf_url: Option<String>,
}

/// Google Scholar SERP scraper via headless Chromium.
pub struct ScholarBackend {
    #[cfg(feature = "headless")]
    pool: std::sync::Arc<crate::browser::pool::BrowserPool>,
}

impl ScholarBackend {
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
            "https://scholar.google.com/scholar?q={encoded}&start={start}&hl=en"
        )
    }

    #[cfg(feature = "headless")]
    fn parse_serp(html: &str, page: usize) -> Vec<ResultItem> {
        use scraper::{Html, Selector};

        let document = Html::parse_document(html);
        let mut results = Vec::new();

        let container_sel = Selector::parse("div.gs_r.gs_or.gs_scl").unwrap();
        let title_sel = Selector::parse("h3.gs_rt a").unwrap();
        let snippet_sel = Selector::parse("div.gs_rs").unwrap();
        let meta_sel = Selector::parse("div.gs_a").unwrap();
        let cite_sel = Selector::parse("div.gs_fl a").unwrap();

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

            // Parse metadata line: "A Smith, B Jones - Nature, 2024"
            let meta = container
                .select(&meta_sel)
                .next()
                .map(|e| e.text().collect::<String>())
                .unwrap_or_default();
            let scholar_meta = parse_scholar_meta(&meta);

            // Extract citation count from "Cited by N"
            let citation_count = container
                .select(&cite_sel)
                .find(|e| e.text().collect::<String>().starts_with("Cited by"))
                .and_then(|e| {
                    let text = e.text().collect::<String>();
                    text.trim_start_matches("Cited by ").parse::<u32>().ok()
                });

            // Build rich snippet with academic metadata
            let rich_snippet = build_rich_snippet(&snippet, &scholar_meta, citation_count);

            // Encode citation count in snippet for authority_score to pick up
            let final_snippet = if let Some(n) = citation_count {
                format!("{rich_snippet} [Cited by {n}]")
            } else {
                rich_snippet
            };

            let year_str = scholar_meta.year.map(|y| format!("{y}-01-01"));

            results.push(ResultItem {
                title,
                url,
                snippet: final_snippet,
                rank,
                backend: BackendId::GoogleScholar,
                score: None,
                published_date: year_str,
            });
            rank += 1;
        }

        debug!("Scholar parse_serp: {} results from page {}", results.len(), page);
        results
    }
}

#[cfg(feature = "headless")]
fn parse_scholar_meta(meta: &str) -> ScholarMetadata {
    let mut result = ScholarMetadata::default();
    // Format: "Author1, Author2 - Venue, Year - publisher"
    let parts: Vec<&str> = meta.splitn(3, " - ").collect();
    if parts.len() >= 2 {
        result.authors = parts[0]
            .split(',')
            .map(|a| a.trim().to_string())
            .filter(|a| !a.is_empty())
            .collect();

        let venue_year = parts[1];
        // Extract year (4 consecutive digits)
        if let Some(year_str) = venue_year
            .split(|c: char| !c.is_ascii_digit())
            .find(|s| s.len() == 4)
        {
            result.year = year_str.parse().ok();
        }
        result.venue = Some(venue_year.trim().to_string());
    }
    result
}

#[cfg(feature = "headless")]
fn build_rich_snippet(
    snippet: &str,
    meta: &ScholarMetadata,
    citation_count: Option<u32>,
) -> String {
    let mut parts = vec![snippet.to_string()];

    if !meta.authors.is_empty() {
        let authors = meta.authors.join(", ");
        let year_str = meta
            .year
            .map(|y| format!(" ({y})"))
            .unwrap_or_default();
        parts.push(format!("Authors: {authors}{year_str}"));
    }

    if let Some(venue) = &meta.venue {
        parts.push(format!("Venue: {venue}"));
    }

    if let Some(n) = citation_count {
        parts.push(format!("{n} citations"));
    }

    parts.join(" | ")
}

#[async_trait]
impl SearchBackend for ScholarBackend {
    fn id(&self) -> BackendId {
        BackendId::GoogleScholar
    }

    fn requires_headless(&self) -> bool {
        true
    }

    async fn search(&self, query: &str, max_results: u32) -> HsxResult<Vec<ResultItem>> {
        #[cfg(not(feature = "headless"))]
        {
            let _ = (query, max_results);
            warn!("Scholar backend requires --features headless");
            return Ok(vec![]);
        }

        #[cfg(feature = "headless")]
        {
            // Scholar: max 2 pages (20 results) to avoid rate limits
            let pages = ((max_results as usize + 9) / 10).min(2);
            let mut all_results = Vec::new();

            for page in 0..pages {
                let url = Self::build_url(query, page);

                if page > 0 {
                    // Scholar rate-limits aggressively; be conservative
                    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                }

                match self.pool.acquire_tab().await {
                    Ok(tab) => match tab.navigate(&url, 20_000).await {
                        Ok(_) => {
                            tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                            if let Ok(html) = tab.content().await {
                                all_results.extend(Self::parse_serp(&html, page));
                            }
                        }
                        Err(e) => warn!("Scholar navigate error page {page}: {e}"),
                    },
                    Err(e) => {
                        warn!("Scholar tab acquire failed: {e}");
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
        let url = ScholarBackend::build_url("transformer attention", 0);
        assert!(url.contains("scholar.google.com"));
        assert!(url.contains("start=0"));
    }

    #[cfg(feature = "headless")]
    #[test]
    fn parse_scholar_meta_full() {
        let meta = "A Smith, B Jones - Nature, 2023 - nature.com";
        let parsed = parse_scholar_meta(meta);
        assert!(parsed.authors.contains(&"A Smith".to_string()));
        assert_eq!(parsed.year, Some(2023));
        assert!(parsed.venue.is_some());
    }

    #[cfg(feature = "headless")]
    #[test]
    fn parse_scholar_meta_no_year() {
        let meta = "John Doe - Unknown Venue";
        let parsed = parse_scholar_meta(meta);
        assert_eq!(parsed.year, None);
    }

    #[cfg(feature = "headless")]
    #[test]
    fn build_rich_snippet_with_citations() {
        let meta = ScholarMetadata {
            authors: vec!["Smith".to_string()],
            year: Some(2022),
            citation_count: None,
            venue: Some("Nature".to_string()),
            pdf_url: None,
        };
        let s = build_rich_snippet("Abstract text here.", &meta, Some(500));
        assert!(s.contains("Smith"));
        assert!(s.contains("500 citations"));
    }

    #[cfg(not(feature = "headless"))]
    #[test]
    fn stub_compiles() {
        let backend = ScholarBackend::new_stub();
        assert_eq!(backend.id(), crate::types::BackendId::GoogleScholar);
    }
}
