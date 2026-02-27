//! Boilerplate removal — strip nav, footer, ads, scripts, styles.
//!
//! PRD SS20: Boilerplate stripping yields ~30% token savings.
//! QADD (Query-Aware DOM Distillation): pre-filter heavy tags from raw HTML
//! strings before DOM parsing to reduce parse cost by 60-80%.

use once_cell::sync::Lazy;
use regex::Regex;

/// CSS selectors for elements that are almost always boilerplate.
pub const BOILERPLATE_SELECTORS: &[&str] = &[
    "nav",
    "footer",
    "header",
    "aside",
    "script",
    "style",
    "noscript",
    "iframe",
    "svg",
    ".sidebar",
    ".navigation",
    ".menu",
    ".footer",
    ".header",
    ".nav",
    ".ads",
    ".ad",
    ".advertisement",
    ".social-share",
    ".cookie-banner",
    ".popup",
    ".modal",
    "#cookie-consent",
    "#gdpr",
    "[role='navigation']",
    "[role='banner']",
    "[role='contentinfo']",
    "[aria-hidden='true']",
];

/// Tags to strip inline (keep text content).
pub const INLINE_STRIP_TAGS: &[&str] = &["span", "em", "strong", "b", "i", "u", "a"];

static WHITESPACE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s{3,}").expect("valid regex"));
static EMPTY_LINES_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\n{3,}").expect("valid regex"));

/// Regex patterns for QADD heavy-tag stripping.
/// The `(?si)` flags enable case-insensitive matching (`i`) and dotall mode
/// (`s`) so `.` matches newlines, allowing multi-line tag bodies to be removed
/// in a single pass.
static SCRIPT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?si)<script[^>]*>.*?</script>").expect("valid regex"));
static STYLE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?si)<style[^>]*>.*?</style>").expect("valid regex"));
static NOSCRIPT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?si)<noscript[^>]*>.*?</noscript>").expect("valid regex"));
static SVG_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?si)<svg[^>]*>.*?</svg>").expect("valid regex"));
static IFRAME_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?si)<iframe[^>]*>.*?</iframe>").expect("valid regex"));

/// QADD pre-filter: strip heavy tag blocks from a raw HTML string before DOM
/// parsing.
///
/// Removes `<script>`, `<style>`, `<noscript>`, `<svg>`, and `<iframe>`
/// elements — including all their content — using fast regex passes over the
/// raw string. This shrinks the DOM tree by 60–80% before it is handed to the
/// scraper or lol_html parser, reducing:
///
/// - Parse time (fewer nodes to allocate and link)
/// - RAM usage (smaller arena)
/// - CSS-selector query time (fewer nodes to traverse)
///
/// The function is intentionally conservative: it only removes tags whose
/// entire subtree is never useful for text extraction. Block-level content
/// tags such as `<nav>` or `<footer>` are left for the downstream DOM-aware
/// boilerplate logic which can inspect class/role attributes.
pub fn strip_heavy_tags(html: &str) -> String {
    let html = SCRIPT_RE.replace_all(html, "");
    let html = STYLE_RE.replace_all(&html, "");
    let html = NOSCRIPT_RE.replace_all(&html, "");
    let html = SVG_RE.replace_all(&html, "");
    let html = IFRAME_RE.replace_all(&html, "");
    html.into_owned()
}

/// Clean extracted text: collapse whitespace, remove empty lines.
pub fn clean_text(text: &str) -> String {
    let text = WHITESPACE_RE.replace_all(text, "  ");
    let text = EMPTY_LINES_RE.replace_all(&text, "\n\n");
    text.trim().to_string()
}

/// Estimate the text-to-HTML ratio for quality assessment.
pub fn text_ratio(html: &str, text: &str) -> f64 {
    if html.is_empty() {
        return 0.0;
    }
    text.len() as f64 / html.len() as f64
}

/// Minimum text length (chars) for Layer 1 to be considered sufficient.
pub const MIN_CONTENT_LENGTH: usize = 200;

/// Minimum text ratio for Layer 1 to be considered sufficient.
pub const MIN_TEXT_RATIO: f64 = 0.05;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_heavy_tags_removes_script() {
        let html = r#"<html><head><script>alert('xss');</script></head><body><p>Content</p></body></html>"#;
        let result = strip_heavy_tags(html);
        assert!(!result.contains("alert"));
        assert!(!result.contains("<script"));
        assert!(result.contains("Content"));
    }

    #[test]
    fn strip_heavy_tags_removes_style() {
        let html = r#"<html><head><style>.foo { color: red; }</style></head><body><p>Text</p></body></html>"#;
        let result = strip_heavy_tags(html);
        assert!(!result.contains("color: red"));
        assert!(!result.contains("<style"));
        assert!(result.contains("Text"));
    }

    #[test]
    fn strip_heavy_tags_removes_noscript() {
        let html =
            r#"<html><body><noscript>Please enable JS</noscript><p>Article text</p></body></html>"#;
        let result = strip_heavy_tags(html);
        assert!(!result.contains("Please enable JS"));
        assert!(!result.contains("<noscript"));
        assert!(result.contains("Article text"));
    }

    #[test]
    fn strip_heavy_tags_removes_svg() {
        let html = r#"<html><body><svg viewBox="0 0 100 100"><circle cx="50" cy="50" r="40"/></svg><p>Main content</p></body></html>"#;
        let result = strip_heavy_tags(html);
        assert!(!result.contains("<svg"));
        assert!(!result.contains("<circle"));
        assert!(result.contains("Main content"));
    }

    #[test]
    fn strip_heavy_tags_removes_iframe() {
        let html = r#"<html><body><iframe src="https://ads.example.com">Ad fallback</iframe><p>Real content</p></body></html>"#;
        let result = strip_heavy_tags(html);
        assert!(!result.contains("<iframe"));
        assert!(!result.contains("Ad fallback"));
        assert!(result.contains("Real content"));
    }

    #[test]
    fn strip_heavy_tags_preserves_regular_content() {
        let html = r#"<html><body><article><h1>Title</h1><p>Paragraph one.</p><p>Paragraph two.</p></article></body></html>"#;
        let result = strip_heavy_tags(html);
        assert!(result.contains("Title"));
        assert!(result.contains("Paragraph one"));
        assert!(result.contains("Paragraph two"));
        assert!(result.contains("<article>"));
    }

    #[test]
    fn strip_heavy_tags_handles_empty_input() {
        let result = strip_heavy_tags("");
        assert_eq!(result, "");
    }

    #[test]
    fn strip_heavy_tags_multiline_script() {
        let html = "<html><body>\n<script type=\"text/javascript\">\n  var x = 1;\n  var y = 2;\n</script>\n<p>Visible</p></body></html>";
        let result = strip_heavy_tags(html);
        assert!(!result.contains("var x"));
        assert!(result.contains("Visible"));
    }
}
