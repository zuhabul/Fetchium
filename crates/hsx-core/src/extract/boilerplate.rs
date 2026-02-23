//! Boilerplate removal — strip nav, footer, ads, scripts, styles.
//!
//! PRD SS20: Boilerplate stripping yields ~30% token savings.

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
