//! CEP Layer 1: CSS selector extraction using `scraper` crate.
//!
//! PRD SS16: "HTTP + Cheerio" — ~2ms, ~5MB RAM, handles 85% of web pages.
//! This is the fastest extraction method, using CSS selectors to pull
//! title, main content, and metadata from static HTML.

use crate::extract::boilerplate::{self, clean_text, BOILERPLATE_SELECTORS};
use crate::extract::{ContentMetadata, ExtractedContent};
use crate::types::CepLayer;
use scraper::{Html, Selector};
use tracing::debug;

/// Extract content from HTML using CSS selectors (Layer 1).
pub fn extract(html: &str, url: &str) -> ExtractedContent {
    let document = Html::parse_document(html);

    let title = extract_title(&document);
    let metadata = extract_metadata(&document, url);

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
    let title_sel = Selector::parse("title").expect("valid selector");
    if let Some(el) = doc.select(&title_sel).next() {
        let t = el.text().collect::<String>().trim().to_string();
        if !t.is_empty() {
            return t;
        }
    }
    let h1_sel = Selector::parse("h1").expect("valid selector");
    if let Some(el) = doc.select(&h1_sel).next() {
        return el.text().collect::<String>().trim().to_string();
    }
    let og_sel = Selector::parse("meta[property='og:title']").expect("valid selector");
    if let Some(el) = doc.select(&og_sel).next() {
        if let Some(content) = el.value().attr("content") {
            return content.trim().to_string();
        }
    }
    String::new()
}

fn extract_metadata(doc: &Html, _url: &str) -> ContentMetadata {
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
    if let Some(val) = el.value().attr("content") {
        let trimmed = val.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if let Some(val) = el.value().attr("datetime") {
        return Some(val.trim().to_string());
    }
    if let Some(val) = el.value().attr("lang") {
        return Some(val.trim().to_string());
    }
    None
}

fn extract_main_content(doc: &Html) -> String {
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
                    collect_text_excluding_boilerplate(el, &mut text);
                }
                let cleaned = text.trim().to_string();
                if cleaned.len() >= boilerplate::MIN_CONTENT_LENGTH {
                    return cleaned;
                }
            }
        }
    }

    if let Ok(body_sel) = Selector::parse("body") {
        if let Some(body) = doc.select(&body_sel).next() {
            let mut text = String::new();
            collect_text_excluding_boilerplate(body, &mut text);
            return text.trim().to_string();
        }
    }

    doc.root_element().text().collect::<String>()
}

fn collect_text_excluding_boilerplate(
    element: scraper::ElementRef<'_>,
    output: &mut String,
) {
    let tag = element.value().name();

    if matches!(
        tag,
        "script" | "style" | "nav" | "footer" | "aside" | "noscript" | "svg" | "iframe"
    ) {
        return;
    }

    for bp in BOILERPLATE_SELECTORS {
        if *bp == tag {
            return;
        }
    }

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
        assert_eq!(estimate_tokens("test"), 1);
        assert_eq!(estimate_tokens("hello world twelve"), 5);
    }
}
