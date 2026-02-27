//! QADD structural pruning — Step 1 of the 5-step pipeline.
//!
//! Removes navigation, footers, scripts, ads, and other boilerplate
//! BEFORE content scoring, maximizing signal-to-noise ratio.

use tracing::trace;

/// Tags to remove entirely in structural pruning.
const STRUCTURAL_PRUNE_TAGS: &[&str] = &[
    "nav", "footer", "aside", "script", "style", "noscript", "iframe", "svg", "form", "header",
    "button", "input", "select", "textarea", "picture", "canvas",
];

/// Class/id substrings that indicate boilerplate content.
const BOILERPLATE_PATTERNS: &[&str] = &[
    "sidebar",
    "advertisement",
    " ad ",
    "advert",
    "cookie",
    "popup",
    "modal",
    "banner",
    "newsletter",
    "subscribe",
    "social-share",
    "share-buttons",
    "comments",
    "comment-section",
    "related-posts",
    "recommended",
    "footer",
    "breadcrumb",
    "pagination",
    "navigation",
    "skip-to",
    "print-only",
];

/// A text node extracted from the DOM after structural pruning.
#[derive(Debug, Clone)]
pub struct TextNode {
    /// The text content.
    pub text: String,
    /// The containing HTML tag (p, h1, h2, li, td, etc.).
    pub tag_context: String,
    /// DOM nesting depth (0 = top level).
    pub depth: usize,
    /// Estimated token count (~1.33 tokens/word, or chars/4).
    pub estimated_tokens: usize,
    /// BM25 relevance score (set in pipeline Step 2).
    pub relevance_score: f64,
}

impl TextNode {
    pub fn new(text: String, tag_context: String, depth: usize) -> Self {
        let estimated_tokens = estimate_tokens(&text);
        Self {
            text,
            tag_context,
            depth,
            estimated_tokens,
            relevance_score: 0.0,
        }
    }

    /// Returns true if this node is a heading (h1-h6).
    pub fn is_heading(&self) -> bool {
        matches!(
            self.tag_context.as_str(),
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
        )
    }

    /// Heading level (1-6 for h1-h6, 0 for non-headings).
    pub fn heading_level(&self) -> u8 {
        match self.tag_context.as_str() {
            "h1" => 1,
            "h2" => 2,
            "h3" => 3,
            "h4" => 4,
            "h5" => 5,
            "h6" => 6,
            _ => 0,
        }
    }
}

/// Estimate token count from text. ~4 chars per token (GPT-style approximation).
pub fn estimate_tokens(text: &str) -> usize {
    let chars = text.chars().count();
    let words = text.split_whitespace().count();
    // Average of char-based and word-based estimates
    ((chars / 4) + (words * 4 / 3)) / 2
}

/// Step 1: Structural pruning — walk the HTML, skip pruned elements,
/// collect text nodes with context.
///
/// This is a fast, regex/state-machine based approach that avoids full DOM
/// parsing for maximum throughput.
pub fn structural_prune(html: &str) -> Vec<TextNode> {
    let mut nodes = Vec::new();
    let mut depth = 0usize;
    let mut current_tag = String::new();
    let mut skip_depth: Option<usize> = None;
    let mut in_tag = false;
    let mut tag_buf = String::new();
    let mut text_buf = String::new();
    let mut is_closing = false;
    let mut char_iter = html.char_indices().peekable();

    while let Some((_, c)) = char_iter.next() {
        if c == '<' {
            // Flush accumulated text
            if !text_buf.trim().is_empty() && skip_depth.is_none() {
                let trimmed = text_buf.trim().to_string();
                if trimmed.len() >= 10 {
                    // Skip very short nodes
                    nodes.push(TextNode::new(trimmed, current_tag.clone(), depth));
                }
            }
            text_buf.clear();
            in_tag = true;
            tag_buf.clear();
            is_closing = false;
        } else if c == '>' && in_tag {
            in_tag = false;
            let tag_name = tag_name_from_buf(&tag_buf);
            // HTML declarations (<!DOCTYPE>, <!--...-->) and void elements are self-closing.
            // Treating them as opening tags would shift depth for the entire document.
            let is_self_closing =
                tag_buf.ends_with('/') || is_void_element(&tag_name) || tag_name.starts_with('!');

            if is_closing {
                // Exiting a skip zone: depth was incremented when the pruned element opened,
                // so the closing tag arrives when depth == skip_depth + 1.
                if let Some(sd) = skip_depth {
                    if depth > 0 && depth - 1 == sd {
                        skip_depth = None;
                        trace!("QADD prune: exited skip zone at depth {}", sd);
                    }
                }
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    current_tag.clear();
                }
            } else if !is_self_closing {
                // Check if we should skip this element
                if skip_depth.is_none() && should_prune_tag(&tag_name, &tag_buf) {
                    skip_depth = Some(depth);
                    trace!("QADD prune: skipping <{tag_name}> at depth {depth}");
                }
                current_tag = tag_name.clone();
                depth += 1;
            }
            tag_buf.clear();
        } else if in_tag {
            if c == '/' && tag_buf.is_empty() {
                is_closing = true;
            } else {
                tag_buf.push(c);
            }
        } else if skip_depth.is_none() {
            // Decode basic HTML entities on the fly
            if c == '&' {
                // Collect entity
                let mut entity = String::from("&");
                for (_, ec) in char_iter.by_ref() {
                    entity.push(ec);
                    if ec == ';' || entity.len() > 8 {
                        break;
                    }
                }
                text_buf.push_str(&decode_entity(&entity));
            } else {
                text_buf.push(c);
            }
        }
    }

    // Flush any remaining text
    if !text_buf.trim().is_empty() && skip_depth.is_none() {
        let trimmed = text_buf.trim().to_string();
        if trimmed.len() >= 10 {
            nodes.push(TextNode::new(trimmed, current_tag, depth));
        }
    }

    trace!(
        "QADD structural_prune: {} text nodes extracted from {} chars",
        nodes.len(),
        html.len()
    );
    nodes
}

fn tag_name_from_buf(buf: &str) -> String {
    buf.split_whitespace()
        .next()
        .unwrap_or("")
        .to_lowercase()
        .trim_start_matches('/')
        .to_string()
}

fn should_prune_tag(tag: &str, attrs: &str) -> bool {
    if STRUCTURAL_PRUNE_TAGS.contains(&tag) {
        return true;
    }
    // Check role/class/id attributes for boilerplate signals
    let attrs_lower = attrs.to_lowercase();
    if attrs_lower.contains("role=\"navigation\"") || attrs_lower.contains("role='navigation'") {
        return true;
    }
    BOILERPLATE_PATTERNS.iter().any(|p| attrs_lower.contains(p))
}

fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn decode_entity(entity: &str) -> String {
    match entity {
        "&amp;" => "&".to_string(),
        "&lt;" => "<".to_string(),
        "&gt;" => ">".to_string(),
        "&quot;" => "\"".to_string(),
        "&#39;" | "&apos;" => "'".to_string(),
        "&nbsp;" => " ".to_string(),
        _ => entity.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prune_removes_nav() {
        let html = r#"<div><nav>Skip navigation</nav><p>Real content here for testing purposes.</p></div>"#;
        let nodes = structural_prune(html);
        assert!(
            !nodes.iter().any(|n| n.text.contains("Skip navigation")),
            "nav should be pruned"
        );
        assert!(
            nodes.iter().any(|n| n.text.contains("Real content")),
            "p content should survive"
        );
    }

    #[test]
    fn prune_removes_script() {
        let html = r#"<body><script>var x = 1;</script><p>This is real paragraph content for users to read.</p></body>"#;
        let nodes = structural_prune(html);
        assert!(!nodes.iter().any(|n| n.text.contains("var x")));
    }

    #[test]
    fn prune_keeps_article_content() {
        let html = r#"<article><h1>Main Title Here</h1><p>This is the main article content with substantial information for testing.</p></article>"#;
        let nodes = structural_prune(html);
        assert!(!nodes.is_empty());
    }

    #[test]
    fn estimate_tokens_reasonable() {
        // 100 words ≈ 133 tokens
        let text = "word ".repeat(100);
        let tokens = estimate_tokens(&text);
        assert!(tokens > 50 && tokens < 300, "tokens={tokens}");
    }

    #[test]
    fn text_node_heading_detection() {
        let node = TextNode::new("Title".to_string(), "h2".to_string(), 1);
        assert!(node.is_heading());
        assert_eq!(node.heading_level(), 2);

        let node2 = TextNode::new("Para".to_string(), "p".to_string(), 2);
        assert!(!node2.is_heading());
    }
}
