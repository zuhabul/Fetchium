//! CEP Layer 2: Readability-style extraction using `lol_html` streaming rewriter.
//!
//! PRD SS16: "HTTP + Readability" — ~8ms, ~10MB RAM, for article pages.
//! Uses lol_html to stream through HTML, stripping boilerplate in a single
//! pass without building a full DOM tree.

use crate::extract::boilerplate::{self, clean_text, MIN_CONTENT_LENGTH};
use crate::extract::layer1::estimate_tokens;
use crate::extract::{ContentMetadata, ExtractedContent};
use crate::types::CepLayer;
use lol_html::{element, rewrite_str, RewriteStrSettings};
use once_cell::sync::Lazy;
use regex::Regex;
use tracing::debug;

/// Tags to remove entirely (element and all children).
const REMOVE_TAGS: &[&str] = &[
    "script",
    "style",
    "nav",
    "footer",
    "aside",
    "noscript",
    "svg",
    "iframe",
    "form",
    "button",
    "input",
    "select",
    "textarea",
];

/// Extract content using lol_html streaming rewriter (Layer 2).
///
/// This method is more aggressive at stripping boilerplate than Layer 1.
/// It processes HTML in a single streaming pass, making it memory-efficient
/// for large pages.
///
/// QADD pre-filtering with `boilerplate::strip_heavy_tags` is applied before
/// the lol_html pass to further reduce the amount of data the streamer must
/// process.
pub fn extract(html: &str, url: &str) -> ExtractedContent {
    // QADD: strip heavy tags before DOM parsing to reduce parse cost
    let stripped_html = boilerplate::strip_heavy_tags(html);

    let title = extract_title_simple(&stripped_html);
    let metadata = extract_metadata_simple(&stripped_html);

    let cleaned_html = strip_boilerplate(&stripped_html);
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
    selectors.push(element!(".sidebar, .nav, .menu, .footer, .header, .ads, .ad, .advertisement", |el| {
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
    static TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<[^>]+>").expect("valid regex"));
    static ENTITY_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"&(amp|lt|gt|quot|apos|nbsp|#\d+|#x[0-9a-fA-F]+);").expect("valid regex")
    });

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

    let text = TAG_RE.replace_all(&text, "");

    let text = ENTITY_RE.replace_all(&text, |caps: &regex::Captures| {
        match &caps[1] {
            "amp" => "&".to_string(),
            "lt" => "<".to_string(),
            "gt" => ">".to_string(),
            "quot" => "\"".to_string(),
            "apos" => "'".to_string(),
            "nbsp" => " ".to_string(),
            s if s.starts_with('#') => {
                let num = if let Some(hex) = s.strip_prefix("#x") {
                    u32::from_str_radix(hex, 16).ok()
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
    static TITLE_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?i)<title[^>]*>(.*?)</title>").expect("valid regex"));

    TITLE_RE
        .captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_default()
}

/// Simple metadata extraction using regex (no DOM).
fn extract_metadata_simple(html: &str) -> ContentMetadata {
    static DESC_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"(?i)<meta[^>]*name=["']description["'][^>]*content=["']([^"']*)["']"#)
            .expect("valid regex")
    });
    static AUTHOR_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"(?i)<meta[^>]*name=["']author["'][^>]*content=["']([^"']*)["']"#)
            .expect("valid regex")
    });
    static OG_DESC_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"(?i)<meta[^>]*property=["']og:description["'][^>]*content=["']([^"']*)["']"#)
            .expect("valid regex")
    });

    ContentMetadata {
        description: DESC_RE
            .captures(html)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .or_else(|| {
                OG_DESC_RE
                    .captures(html)
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().to_string())
            }),
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
        assert!(text.contains("Tom & Jerry"));
        assert!(text.contains("<3"));
        assert!(text.contains("\"cheese\""));
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
