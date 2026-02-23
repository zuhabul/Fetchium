//! BibTeX citation export (Phase 5, PRD §26).
//!
//! Generates BibTeX entries from source lists. Useful for academic workflows
//! where the research output needs to cite web sources in LaTeX documents.

use crate::types::Source;

/// Generate a BibTeX bibliography from a list of sources.
///
/// Each source is rendered as either `@article` (arxiv) or `@misc`.
pub fn generate_bibtex(sources: &[Source]) -> String {
    let mut bib = String::new();
    for (i, source) in sources.iter().enumerate() {
        let key = bibtex_key(&source.title, i + 1);
        let entry_type = if source.url.contains("arxiv.org") {
            "article"
        } else {
            "misc"
        };

        bib.push_str(&format!("@{entry_type}{{{key},\n"));
        bib.push_str(&format!(
            "  title  = {{{}}},\n",
            escape_bibtex(&source.title)
        ));
        bib.push_str(&format!("  url    = {{{}}},\n", source.url));

        if let Some(ref date) = source.published_date {
            // Extract 4-digit year from any date format
            let year: String = date.chars().filter(|c| c.is_ascii_digit()).take(4).collect();
            if year.len() == 4 {
                bib.push_str(&format!("  year   = {{{year}}},\n"));
            }
        }

        bib.push_str(&format!(
            "  note   = {{Accessed: {}}},\n",
            chrono::Utc::now().format("%Y-%m-%d")
        ));
        bib.push_str("}\n\n");
    }
    bib
}

/// Generate a BibTeX key from a title and index.
///
/// Key format: first word of title (lowercase, alphanumeric) + index.
fn bibtex_key(title: &str, index: usize) -> String {
    let first_word: String = title
        .split_whitespace()
        .next()
        .unwrap_or("source")
        .chars()
        .filter(|c| c.is_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect();

    if first_word.is_empty() {
        format!("source{index}")
    } else {
        format!("{first_word}{index}")
    }
}

/// Escape special BibTeX characters in a string.
fn escape_bibtex(s: &str) -> String {
    s.replace('{', "\\{").replace('}', "\\}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FetchMethod, Source};

    fn make_source(title: &str, url: &str, date: Option<&str>) -> Source {
        Source {
            id: 1,
            url: url.to_string(),
            title: title.to_string(),
            domain: url.to_string(),
            fetch_method: FetchMethod::Http,
            content_type: "text/html".into(),
            tokens: 100,
            published_date: date.map(|s| s.to_string()),
            trust_score: 0.8,
            citation: None,
        }
    }

    #[test]
    fn bibtex_contains_title_and_url() {
        let sources = vec![make_source("Rust Programming", "https://rust-lang.org", None)];
        let bib = generate_bibtex(&sources);
        assert!(bib.contains("Rust Programming"));
        assert!(bib.contains("https://rust-lang.org"));
    }

    #[test]
    fn arxiv_uses_article_type() {
        let sources = vec![make_source(
            "Attention Is All You Need",
            "https://arxiv.org/abs/1706.03762",
            Some("2017-06-12"),
        )];
        let bib = generate_bibtex(&sources);
        assert!(bib.contains("@article{"));
        assert!(bib.contains("year   = {2017}"));
    }

    #[test]
    fn non_arxiv_uses_misc_type() {
        let sources = vec![make_source("Blog Post", "https://blog.example.com/post", None)];
        let bib = generate_bibtex(&sources);
        assert!(bib.contains("@misc{"));
    }

    #[test]
    fn bibtex_key_generation() {
        assert_eq!(bibtex_key("Rust Programming", 1), "rust1");
        assert_eq!(bibtex_key("", 2), "source2");
        assert_eq!(bibtex_key("123 Numeric", 3), "1233");
    }

    #[test]
    fn escape_braces() {
        assert_eq!(escape_bibtex("{braces}"), "\\{braces\\}");
    }
}
