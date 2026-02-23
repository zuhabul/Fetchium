//! PDF text extraction via pdftotext subprocess (poppler-utils).

use crate::error::{HsxError, HsxResult};
use super::{ContentType, MultimodalContent, MultimodalSegment};
use std::process::Command;

/// Extract text from a local PDF file using `pdftotext`.
///
/// Falls back gracefully if pdftotext is not installed.
pub fn extract_pdf_file(path: &std::path::Path) -> HsxResult<MultimodalContent> {
    // Run: pdftotext -layout <path> -
    let output = Command::new("pdftotext")
        .args(["-layout", &path.to_string_lossy(), "-"])
        .output()
        .map_err(|e| {
            HsxError::Extraction(format!(
                "pdftotext not available (install poppler-utils): {e}"
            ))
        })?;

    if !output.status.success() {
        return Err(HsxError::Extraction(
            "pdftotext returned an error".into(),
        ));
    }

    let full_text = String::from_utf8_lossy(&output.stdout).to_string();
    // Split on form-feed (^L) to separate pages
    let page_texts: Vec<&str> = full_text.split('\x0C').collect();
    let page_count = page_texts.len() as u32;

    let segments: Vec<MultimodalSegment> = page_texts
        .iter()
        .enumerate()
        .filter(|(_, t)| !t.trim().is_empty())
        .map(|(i, t)| MultimodalSegment {
            offset_ms: None,
            page: Some(i as u32 + 1),
            text: t.trim().to_string(),
        })
        .collect();

    Ok(MultimodalContent {
        source_url: path.to_string_lossy().to_string(),
        content_type: ContentType::Pdf { page_count },
        text: full_text.trim().to_string(),
        segments,
        extracted_at: chrono::Utc::now(),
    })
}

/// Download a PDF from a URL and extract its text.
pub async fn extract_pdf_url(
    url: &str,
    http: &crate::http::client::HttpClient,
) -> HsxResult<MultimodalContent> {
    // Fetch the URL using the inner reqwest client for binary content
    let resp = http
        .client()
        .get(url)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(crate::error::HsxError::Network)?;

    let bytes = resp
        .bytes()
        .await
        .map_err(crate::error::HsxError::Network)?;

    // Write to a temp file and run pdftotext
    let mut tmp = tempfile::Builder::new().suffix(".pdf").tempfile()?;
    use std::io::Write;
    tmp.write_all(&bytes)?;

    let mut result = extract_pdf_file(tmp.path())?;
    result.source_url = url.to_string();
    Ok(result)
}

/// Check if pdftotext is available on this system.
pub fn is_available() -> bool {
    Command::new("pdftotext")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_available_does_not_panic() {
        // Just ensure the check runs without panicking
        let _ = is_available();
    }
}
