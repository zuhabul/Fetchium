//! ArXiv search backend — Atom XML API (no auth required).
//!
//! Uses the official ArXiv API at `export.arxiv.org/api/query`. Returns
//! academic papers relevant to the query, sorted by relevance.
//!
//! Parsing is done with a lightweight hand-written state machine rather than
//! pulling in an XML crate dependency.

use crate::error::FetchiumResult;
use crate::http::HttpClient;
use crate::search::SearchBackend;
use crate::types::{BackendId, ResultItem};
use async_trait::async_trait;
use tracing::debug;

/// ArXiv API base URL.
const ARXIV_API: &str = "https://export.arxiv.org/api/query";

/// ArXiv search backend using the official Atom XML API.
///
/// Queries all fields (`all:`) by default. Returns at most 20 results
/// (ArXiv API hard limit for free queries).
pub struct ArxivBackend {
    http: HttpClient,
}

impl ArxivBackend {
    /// Create a new ArXiv backend with the given HTTP client.
    pub fn new(http: HttpClient) -> Self {
        Self { http }
    }
}

#[async_trait]
impl SearchBackend for ArxivBackend {
    fn id(&self) -> BackendId {
        BackendId::Arxiv
    }

    async fn search(&self, query: &str, max_results: u32) -> FetchiumResult<Vec<ResultItem>> {
        let max = max_results.min(20);
        let url = format!(
            "{ARXIV_API}?search_query=all:{}&max_results={max}&sortBy=relevance",
            urlencoding_encode(query)
        );

        let body = match self.http.fetch_text(&url).await {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!("ArXiv request failed: {e}");
                return Ok(vec![]);
            }
        };

        let results = parse_atom_xml(&body);
        debug!("ArXiv: {} results for {:?}", results.len(), query);
        Ok(results)
    }
}

/// Parse an ArXiv Atom XML response into a list of `ResultItem`s.
///
/// Uses a simple split-and-scan approach rather than a full XML parser for
/// zero additional dependencies. Handles `<entry>` blocks and extracts
/// `<title>`, `<summary>`, `<published>`, and `<link>` fields.
fn parse_atom_xml(xml: &str) -> Vec<ResultItem> {
    let mut results = Vec::new();
    let mut rank = 1u32;

    // Each result is wrapped in <entry>...</entry>
    for entry_block in xml.split("<entry>").skip(1) {
        let end = entry_block.find("</entry>").unwrap_or(entry_block.len());
        let entry = &entry_block[..end];

        let title = match extract_xml_text(entry, "title") {
            Some(t) => t.replace('\n', " ").trim().to_string(),
            None => continue, // Skip malformed entries
        };

        if title.is_empty() {
            continue;
        }

        // Prefer the HTML abstract page link over the PDF link
        let url = extract_arxiv_link(entry)
            .or_else(|| extract_xml_text(entry, "id"))
            .unwrap_or_default();

        let summary = extract_xml_text(entry, "summary")
            .unwrap_or_default()
            .replace('\n', " ");
        let summary = summary.trim();

        // Truncate summary to 300 chars for the snippet
        let snippet = if summary.chars().count() > 300 {
            let truncated: String = summary.chars().take(300).collect();
            format!("{truncated}...")
        } else {
            summary.to_string()
        };

        let published = extract_xml_text(entry, "published");

        results.push(ResultItem {
            title,
            url,
            snippet,
            rank,
            backend: BackendId::Arxiv,
            score: None,
            published_date: published,
        });
        rank += 1;
    }

    results
}

/// Extract the text content of the first occurrence of `<tag>...</tag>`.
///
/// Handles attributes on the opening tag (e.g. `<title type="text">`).
fn extract_xml_text(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let start = xml.find(&open)?;
    // Skip past the closing `>` of the opening tag (handles attributes)
    let after_open_angle = xml[start..].find('>')?;
    let content_start = start + after_open_angle + 1;
    let content_end = xml[content_start..].find(&close)?;
    let text = &xml[content_start..content_start + content_end];
    Some(decode_html_entities(text.trim()))
}

/// Extract the `href` from the HTML-type link element in an `<entry>` block.
///
/// ArXiv entries contain multiple `<link>` elements (HTML page and PDF).
/// We prefer `type="text/html"` or `title="abs"`.
fn extract_arxiv_link(entry: &str) -> Option<String> {
    for link_fragment in entry.split("<link ").skip(1) {
        let is_html = link_fragment.contains(r#"type="text/html""#)
            || link_fragment.contains(r#"title="abs""#);
        if !is_html {
            continue;
        }
        if let Some(href_start) = link_fragment.find(r#"href=""#) {
            let after_href = &link_fragment[href_start + 6..];
            if let Some(end) = after_href.find('"') {
                let href = &after_href[..end];
                if !href.is_empty() {
                    return Some(href.to_string());
                }
            }
        }
    }
    None
}

/// Decode the five predefined XML/HTML entities.
fn decode_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
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

    const SAMPLE_ATOM: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <entry>
    <id>http://arxiv.org/abs/2301.00001v1</id>
    <title>Attention Is All You Need</title>
    <summary>We propose a novel simple network architecture called the Transformer.</summary>
    <published>2017-06-12T00:00:00Z</published>
    <link href="https://arxiv.org/abs/2301.00001" rel="alternate" type="text/html"/>
    <link href="https://arxiv.org/pdf/2301.00001" rel="related" type="application/pdf"/>
  </entry>
</feed>"#;

    const MULTI_ENTRY_ATOM: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <entry>
    <id>http://arxiv.org/abs/1706.03762v1</id>
    <title>Attention Is All You Need</title>
    <summary>Transformer model architecture.</summary>
    <published>2017-06-12T00:00:00Z</published>
    <link href="https://arxiv.org/abs/1706.03762" type="text/html"/>
  </entry>
  <entry>
    <id>http://arxiv.org/abs/1810.04805v1</id>
    <title>BERT: Pre-training of Deep Bidirectional Transformers</title>
    <summary>BERT model for NLP.</summary>
    <published>2018-10-11T00:00:00Z</published>
    <link href="https://arxiv.org/abs/1810.04805" type="text/html"/>
  </entry>
</feed>"#;

    #[test]
    fn parse_sample_arxiv_atom() {
        let results = parse_atom_xml(SAMPLE_ATOM);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Attention Is All You Need");
        assert!(
            results[0].snippet.contains("Transformer"),
            "Snippet: {:?}",
            results[0].snippet
        );
        assert_eq!(results[0].rank, 1);
        assert_eq!(results[0].backend, BackendId::Arxiv);
    }

    #[test]
    fn parse_extracts_html_link() {
        let results = parse_atom_xml(SAMPLE_ATOM);
        assert_eq!(results[0].url, "https://arxiv.org/abs/2301.00001");
    }

    #[test]
    fn parse_multiple_entries() {
        let results = parse_atom_xml(MULTI_ENTRY_ATOM);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].rank, 1);
        assert_eq!(results[1].rank, 2);
        assert_eq!(
            results[1].title,
            "BERT: Pre-training of Deep Bidirectional Transformers"
        );
    }

    #[test]
    fn parse_published_date() {
        let results = parse_atom_xml(SAMPLE_ATOM);
        assert_eq!(
            results[0].published_date.as_deref(),
            Some("2017-06-12T00:00:00Z")
        );
    }

    #[test]
    fn extract_xml_text_works() {
        let xml = "<title>Hello World</title>";
        assert_eq!(
            extract_xml_text(xml, "title").as_deref(),
            Some("Hello World")
        );
    }

    #[test]
    fn extract_xml_text_with_attributes() {
        let xml = r#"<title type="text">Hello World</title>"#;
        assert_eq!(
            extract_xml_text(xml, "title").as_deref(),
            Some("Hello World")
        );
    }

    #[test]
    fn extract_xml_text_missing_tag() {
        let xml = "<summary>Some text</summary>";
        assert!(extract_xml_text(xml, "title").is_none());
    }

    #[test]
    fn decode_entities_works() {
        assert_eq!(decode_html_entities("a &amp; b &lt;c&gt;"), "a & b <c>");
        assert_eq!(decode_html_entities("&quot;quoted&quot;"), "\"quoted\"");
        assert_eq!(decode_html_entities("it&#39;s"), "it's");
    }

    #[test]
    fn empty_xml_returns_empty_vec() {
        let results = parse_atom_xml("");
        assert!(results.is_empty());
    }

    #[test]
    fn empty_entry_skipped() {
        // Entry with empty title should be skipped
        let xml = "<feed><entry><title></title><summary>text</summary></entry></feed>";
        let results = parse_atom_xml(xml);
        assert!(results.is_empty());
    }
}
