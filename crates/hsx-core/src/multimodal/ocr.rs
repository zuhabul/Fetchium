//! OCR text extraction via Tesseract CLI (PRD §34).

use super::{ContentType, MultimodalContent, MultimodalSegment};
use crate::error::{HsxError, HsxResult};
use std::process::Command;

/// Run Tesseract OCR on an image file.
///
/// Requires `tesseract` to be installed and in PATH.
pub fn ocr_image_file(path: &std::path::Path, lang: &str) -> HsxResult<MultimodalContent> {
    let lang = if lang.is_empty() { "eng" } else { lang };

    let output = Command::new("tesseract")
        .args([path.to_str().unwrap_or(""), "stdout", "-l", lang])
        .output()
        .map_err(|e| {
            HsxError::Extraction(format!(
                "Tesseract not available (install tesseract-ocr): {e}"
            ))
        })?;

    if !output.status.success() {
        return Err(HsxError::Extraction(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }

    let full_text = String::from_utf8_lossy(&output.stdout).into_owned();

    Ok(MultimodalContent {
        source_url: path.to_string_lossy().to_string(),
        content_type: ContentType::Image {
            width: 0,
            height: 0,
        },
        text: full_text.clone(),
        segments: vec![MultimodalSegment {
            offset_ms: None,
            page: None,
            text: full_text,
        }],
        extracted_at: chrono::Utc::now(),
    })
}

/// Check whether Tesseract is available.
pub fn is_available() -> bool {
    Command::new("tesseract")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
