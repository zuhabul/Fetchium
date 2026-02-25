//! Output and export system (Phase 5, PRD §26).
//!
//! Supported formats:
//! - Markdown / JSON / Plain text — built-in (from `output` module)
//! - PDF — via Pandoc subprocess (requires Pandoc installation)
//! - DOCX — via Pandoc subprocess (requires Pandoc installation)
//! - BibTeX — pure Rust, no external dependencies

pub mod bibtex;
pub mod pandoc;

pub use bibtex::generate_bibtex;
pub use pandoc::{check_pandoc, export_docx, export_pdf};

use crate::error::HsxError;
use std::path::Path;

/// Supported export formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Pdf,
    Docx,
    Bibtex,
    Markdown,
}

impl ExportFormat {
    /// Parse a format string (case-insensitive).
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pdf" => Some(Self::Pdf),
            "docx" | "word" => Some(Self::Docx),
            "bibtex" | "bib" => Some(Self::Bibtex),
            "md" | "markdown" => Some(Self::Markdown),
            _ => None,
        }
    }
}

/// Export markdown content to a file in the specified format.
pub fn export(
    content: &str,
    format: ExportFormat,
    output_path: &Path,
    title: Option<&str>,
) -> Result<(), HsxError> {
    match format {
        ExportFormat::Pdf => export_pdf(content, output_path, title),
        ExportFormat::Docx => export_docx(content, output_path),
        ExportFormat::Markdown => {
            std::fs::write(output_path, content)?;
            Ok(())
        }
        ExportFormat::Bibtex => {
            // BibTeX export requires sources — caller must use generate_bibtex() directly.
            Err(HsxError::Internal(
                "BibTeX export requires sources list — use generate_bibtex() directly".into(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_from_str() {
        assert_eq!(ExportFormat::parse("pdf"), Some(ExportFormat::Pdf));
        assert_eq!(ExportFormat::parse("PDF"), Some(ExportFormat::Pdf));
        assert_eq!(ExportFormat::parse("docx"), Some(ExportFormat::Docx));
        assert_eq!(ExportFormat::parse("word"), Some(ExportFormat::Docx));
        assert_eq!(ExportFormat::parse("bibtex"), Some(ExportFormat::Bibtex));
        assert_eq!(ExportFormat::parse("bib"), Some(ExportFormat::Bibtex));
        assert_eq!(
            ExportFormat::parse("markdown"),
            Some(ExportFormat::Markdown)
        );
        assert_eq!(ExportFormat::parse("unknown"), None);
    }

    #[test]
    fn export_markdown_writes_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("output.md");
        export("# Hello\n\nContent.", ExportFormat::Markdown, &path, None).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("Hello"));
    }
}
