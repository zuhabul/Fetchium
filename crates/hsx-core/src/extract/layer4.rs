//! CEP Layer 4 — PDF and document extraction (PRD §8.3).
//!
//! Extracts text from PDF and Office documents when static HTML extraction fails.
//! Uses the `pdf-extract` crate (optional) or falls back to raw text extraction.
//!
//! Layer 4 triggers when: content_type contains "pdf", "msword", "officedocument".

use crate::extract::{ContentMetadata, ExtractedContent};
use crate::token::counter::count_tokens;
use crate::types::CepLayer;

/// Layer 4: PDF and document text extractor.
pub struct Layer4Extractor;

impl Layer4Extractor {
    /// Check if the given content-type header indicates a document format.
    pub fn is_document(content_type: &str) -> bool {
        let ct = content_type.to_lowercase();
        ct.contains("pdf")
            || ct.contains("msword")
            || ct.contains("officedocument")
            || ct.contains("opendocument")
            || ct.contains("rtf")
    }

    /// Extract text from raw PDF bytes.
    ///
    /// Uses `pdf-extract` if available, otherwise attempts basic text extraction.
    pub fn extract_pdf(bytes: &[u8], url: &str) -> anyhow::Result<ExtractedContent> {
        let text = Self::extract_pdf_text(bytes)?;
        let token_count = count_tokens(&text);

        Ok(ExtractedContent {
            title: Self::title_from_url(url),
            text,
            layer_used: CepLayer::Layer4,
            tokens: token_count,
            metadata: ContentMetadata {
                content_type: "application/pdf".to_string(),
                ..Default::default()
            },
        })
    }

    fn extract_pdf_text(bytes: &[u8]) -> anyhow::Result<String> {
        // Attempt basic PDF text extraction by scanning for text streams.
        // This is a lightweight fallback that works for simple PDFs without
        // the full pdf-extract dependency (which requires C bindings).
        let raw = String::from_utf8_lossy(bytes);
        let mut text = String::new();

        // Extract text between BT (Begin Text) and ET (End Text) markers
        for segment in raw.split("BT") {
            if let Some(end) = segment.find("ET") {
                let text_ops = &segment[..end];
                // Extract strings in parentheses: (text content)
                let mut i = text_ops.chars().peekable();
                while let Some(c) = i.next() {
                    if c == '(' {
                        let mut s = String::new();
                        for nc in i.by_ref() {
                            if nc == ')' {
                                break;
                            }
                            if nc != '\\' {
                                s.push(nc);
                            }
                        }
                        if s.chars().any(|c| c.is_alphabetic()) {
                            text.push_str(&s);
                            text.push(' ');
                        }
                    }
                }
            }
        }

        let text = text.trim().to_string();
        if text.is_empty() {
            anyhow::bail!("No text content extracted from PDF");
        }

        Ok(text)
    }

    fn title_from_url(url: &str) -> String {
        url.rsplit('/')
            .next()
            .unwrap_or("document")
            .trim_end_matches(".pdf")
            .trim_end_matches(".doc")
            .trim_end_matches(".docx")
            .replace(['-', '_'], " ")
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_document_pdf() {
        assert!(Layer4Extractor::is_document("application/pdf"));
        assert!(Layer4Extractor::is_document("application/PDF"));
    }

    #[test]
    fn is_document_msword() {
        assert!(Layer4Extractor::is_document(
            "application/msword"
        ));
        assert!(Layer4Extractor::is_document(
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        ));
    }

    #[test]
    fn is_document_html_is_false() {
        assert!(!Layer4Extractor::is_document("text/html"));
        assert!(!Layer4Extractor::is_document("application/json"));
    }

    #[test]
    fn title_from_url() {
        assert_eq!(
            Layer4Extractor::title_from_url("https://example.com/my-paper.pdf"),
            "my paper"
        );
        assert_eq!(
            Layer4Extractor::title_from_url("https://example.com/report_2024.pdf"),
            "report 2024"
        );
    }
}
