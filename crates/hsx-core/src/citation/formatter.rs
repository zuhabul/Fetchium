//! 6-style citation formatter (PRD §24).

use crate::citation::types::{CitationStyle, FormattedCitation, SourceMeta};

/// Formats citations in any of 6 supported styles.
pub struct CitationFormatter {
    style: CitationStyle,
}

impl CitationFormatter {
    pub fn new(style: CitationStyle) -> Self {
        Self { style }
    }

    /// Format a single source as a citation.
    pub fn format(&self, source: &SourceMeta, index: usize) -> FormattedCitation {
        match self.style {
            CitationStyle::Inline => self.format_inline(source, index),
            CitationStyle::Footnote => self.format_footnote(source, index),
            CitationStyle::Apa => self.format_apa(source, index),
            CitationStyle::Mla => self.format_mla(source, index),
            CitationStyle::Chicago => self.format_chicago(source, index),
            CitationStyle::Ieee => self.format_ieee(source, index),
            CitationStyle::Bibtex => self.format_bibtex(source, index),
        }
    }

    /// Format a list of sources into a complete reference section string.
    pub fn format_references(&self, sources: &[SourceMeta]) -> String {
        sources
            .iter()
            .enumerate()
            .map(|(i, s)| self.format(s, i + 1).reference_entry)
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_inline(&self, source: &SourceMeta, index: usize) -> FormattedCitation {
        FormattedCitation {
            inline_marker: format!("[{index}]"),
            reference_entry: format!("[{index}] {} - {}", source.title, source.url),
            url: source.url.clone(),
            index,
        }
    }

    fn format_footnote(&self, source: &SourceMeta, index: usize) -> FormattedCitation {
        FormattedCitation {
            inline_marker: format!("^{index}"),
            reference_entry: format!("^{index}. {} ({})", source.title, source.url),
            url: source.url.clone(),
            index,
        }
    }

    fn format_apa(&self, source: &SourceMeta, index: usize) -> FormattedCitation {
        let author = source.author.as_deref().unwrap_or("Unknown");
        let year = Self::extract_year(&source.published_date).unwrap_or_else(|| "n.d.".into());
        let publisher = source.publisher.as_deref().unwrap_or("");
        let pub_prefix = if publisher.is_empty() {
            String::new()
        } else {
            format!("{publisher}. ")
        };
        FormattedCitation {
            inline_marker: format!("({author}, {year})"),
            reference_entry: format!(
                "{author} ({year}). {}. {pub_prefix}Retrieved {} from {}",
                source.title, source.accessed_date, source.url
            ),
            url: source.url.clone(),
            index,
        }
    }

    fn format_mla(&self, source: &SourceMeta, index: usize) -> FormattedCitation {
        let author = source.author.as_deref().unwrap_or("Unknown");
        let publisher = source.publisher.as_deref().unwrap_or("Web");
        let date = source.published_date.as_deref().unwrap_or("n.d.");
        FormattedCitation {
            inline_marker: format!("({author})"),
            reference_entry: format!(
                "{author}. \"{}\" *{publisher}*, {date}, {}.",
                source.title, source.url
            ),
            url: source.url.clone(),
            index,
        }
    }

    fn format_chicago(&self, source: &SourceMeta, index: usize) -> FormattedCitation {
        let author = source.author.as_deref().unwrap_or("Unknown");
        let publisher = source.publisher.as_deref().unwrap_or("");
        let date = source.published_date.as_deref().unwrap_or("n.d.");
        let pub_prefix = if publisher.is_empty() {
            String::new()
        } else {
            format!("{publisher}. ")
        };
        FormattedCitation {
            inline_marker: format!("({author} {date})"),
            reference_entry: format!(
                "{author}. \"{}\" {pub_prefix}{date}. {}.",
                source.title, source.url
            ),
            url: source.url.clone(),
            index,
        }
    }

    fn format_ieee(&self, source: &SourceMeta, index: usize) -> FormattedCitation {
        let author = source.author.as_deref().unwrap_or("Unknown");
        let publisher = source.publisher.as_deref().unwrap_or("Online");
        let date = source.published_date.as_deref().unwrap_or("n.d.");
        FormattedCitation {
            inline_marker: format!("[{index}]"),
            reference_entry: format!(
                "[{index}] {author}, \"{}\" {publisher}, {date}. [Online]. Available: {}",
                source.title, source.url
            ),
            url: source.url.clone(),
            index,
        }
    }

    fn format_bibtex(&self, source: &SourceMeta, index: usize) -> FormattedCitation {
        let key = Self::bibtex_key(source);
        let author = source.author.as_deref().unwrap_or("Unknown");
        let year = Self::extract_year(&source.published_date).unwrap_or_else(|| "0000".into());
        FormattedCitation {
            inline_marker: format!("\\cite{{{key}}}"),
            reference_entry: format!(
                "@misc{{{key},\n  author = {{{author}}},\n  title = {{{}}},\n  year = {{{year}}},\n  url = {{{}}},\n  note = {{Accessed: {}}}\n}}",
                source.title, source.url, source.accessed_date
            ),
            url: source.url.clone(),
            index,
        }
    }

    fn extract_year(date: &Option<String>) -> Option<String> {
        date.as_ref().and_then(|d| d.get(..4).map(|s| s.to_string()))
    }

    fn bibtex_key(source: &SourceMeta) -> String {
        let author_part = source
            .author
            .as_deref()
            .unwrap_or("unknown")
            .split_whitespace()
            .next()
            .unwrap_or("unknown")
            .to_lowercase()
            .replace(',', "");
        let year = Self::extract_year(&source.published_date).unwrap_or_else(|| "0000".into());
        let title_word = source
            .title
            .split_whitespace()
            .next()
            .unwrap_or("untitled")
            .to_lowercase();
        format!("{author_part}{year}{title_word}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> SourceMeta {
        SourceMeta {
            url: "https://example.com/article".into(),
            title: "Understanding Rust".into(),
            author: Some("Smith, J.".into()),
            publisher: Some("Tech Blog".into()),
            published_date: Some("2025-03-15".into()),
            accessed_date: "2026-02-23".into(),
        }
    }

    #[test]
    fn inline_format() {
        let f = CitationFormatter::new(CitationStyle::Inline);
        let c = f.format(&sample(), 1);
        assert_eq!(c.inline_marker, "[1]");
        assert!(c.reference_entry.contains("Understanding Rust"));
    }

    #[test]
    fn apa_format() {
        let f = CitationFormatter::new(CitationStyle::Apa);
        let c = f.format(&sample(), 1);
        assert_eq!(c.inline_marker, "(Smith, J., 2025)");
        assert!(c.reference_entry.contains("Retrieved"));
    }

    #[test]
    fn bibtex_format() {
        let f = CitationFormatter::new(CitationStyle::Bibtex);
        let c = f.format(&sample(), 1);
        assert!(c.inline_marker.starts_with("\\cite{"));
        assert!(c.reference_entry.contains("@misc{"));
        assert!(c.reference_entry.contains("year = {2025}"));
    }

    #[test]
    fn missing_author_fallback() {
        let f = CitationFormatter::new(CitationStyle::Apa);
        let mut s = sample();
        s.author = None;
        let c = f.format(&s, 1);
        assert!(c.inline_marker.contains("Unknown"));
    }

    #[test]
    fn format_references_produces_list() {
        let f = CitationFormatter::new(CitationStyle::Inline);
        let refs = f.format_references(&[sample(), sample()]);
        assert!(refs.contains("[1]"));
        assert!(refs.contains("[2]"));
    }
}
