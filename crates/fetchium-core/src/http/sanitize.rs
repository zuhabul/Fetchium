//! HTML sanitization for safe terminal and file output (PRD §41).
//!
//! All extracted HTML must pass through `sanitize_html()` before display.
//! Uses the `ammonia` crate (the same library used by crates.io itself).

use ammonia::Builder;
use std::collections::HashSet;

/// Sanitize HTML for safe display.
///
/// Strips `<script>`, `<iframe>`, `<object>`, event handlers (`onclick`, etc.),
/// and data URIs. Preserves semantic content tags.
///
/// # Examples
/// ```rust
/// use fetchium_core::http::sanitize::sanitize_html;
/// let safe = sanitize_html(r#"<p>Hello</p><script>alert(1)</script>"#);
/// assert!(safe.contains("Hello"));
/// assert!(!safe.contains("script"));
/// ```
pub fn sanitize_html(html: &str) -> String {
    Builder::default()
        .tags(allowed_tags())
        .clean(html)
        .to_string()
}

/// Strip ALL HTML tags, returning plain text only.
///
/// # Examples
/// ```rust
/// use fetchium_core::http::sanitize::sanitize_to_text;
/// let text = sanitize_to_text(r#"<p>Hello <strong>world</strong></p>"#);
/// assert!(!text.contains('<'));
/// assert!(text.contains("Hello"));
/// ```
pub fn sanitize_to_text(html: &str) -> String {
    ammonia::clean_text(html)
}

fn allowed_tags() -> HashSet<&'static str> {
    [
        "p",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "a",
        "code",
        "pre",
        "table",
        "thead",
        "tbody",
        "tr",
        "th",
        "td",
        "ul",
        "ol",
        "li",
        "blockquote",
        "em",
        "strong",
        "br",
        "span",
        "div",
        "section",
        "article",
        "header",
        "footer",
        "aside",
        "nav",
    ]
    .iter()
    .copied()
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_script_tags() {
        let input = r#"<p>Hello</p><script>alert('xss')</script><p>World</p>"#;
        let output = sanitize_html(input);
        assert!(!output.contains("script"), "script tag should be stripped");
        assert!(
            !output.contains("alert"),
            "script content should be stripped"
        );
        assert!(output.contains("Hello"));
        assert!(output.contains("World"));
    }

    #[test]
    fn strips_event_handlers() {
        let input = r#"<p onclick="evil()">Click me</p>"#;
        let output = sanitize_html(input);
        assert!(
            !output.contains("onclick"),
            "event handler should be stripped"
        );
        assert!(output.contains("Click me"));
    }

    #[test]
    fn strips_iframes() {
        let input = r#"<p>Text</p><iframe src="https://evil.com"></iframe>"#;
        let output = sanitize_html(input);
        assert!(!output.contains("iframe"), "iframe should be stripped");
        assert!(output.contains("Text"));
    }

    #[test]
    fn preserves_code_blocks() {
        let input = r#"<pre><code>fn main() { println!("hello"); }</code></pre>"#;
        let output = sanitize_html(input);
        assert!(output.contains("fn main()"));
        assert!(output.contains("<code>"));
        assert!(output.contains("<pre>"));
    }

    #[test]
    fn preserves_tables() {
        let input = r#"<table><thead><tr><th>A</th></tr></thead><tbody><tr><td>1</td></tr></tbody></table>"#;
        let output = sanitize_html(input);
        assert!(output.contains("<table>"));
        assert!(output.contains("<th>"));
        assert!(output.contains("<td>"));
    }

    #[test]
    fn sanitize_to_text_strips_all_tags() {
        let input = r#"<p>Hello <strong>world</strong></p>"#;
        let output = sanitize_to_text(input);
        assert!(!output.contains('<'), "no HTML tags in plain text");
        assert!(output.contains("Hello"));
        assert!(output.contains("world"));
    }

    #[test]
    fn sanitize_to_text_handles_empty() {
        assert_eq!(sanitize_to_text(""), "");
    }

    #[test]
    fn sanitize_html_handles_empty() {
        assert_eq!(sanitize_html(""), "");
    }
}
