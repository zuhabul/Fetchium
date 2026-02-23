//! CEP Layer 5 — Screenshot OCR (PRD §8.3).
//!
//! Last-resort extraction for canvas-rendered text, image-based pages,
//! and other content invisible to HTML parsing.
//!
//! Pipeline:
//! 1. Navigate to URL with headless browser
//! 2. Scroll to trigger lazy loading
//! 3. Take full-page PNG screenshot
//! 4. Run `tesseract` OCR subprocess on the PNG
//!
//! Requires `--features headless` AND `tesseract` installed in PATH.

/// Check if `tesseract` is available in PATH.
pub fn tesseract_available() -> bool {
    std::process::Command::new("tesseract")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Run tesseract OCR on PNG bytes, returning extracted text.
///
/// Spawns `tesseract stdin stdout --psm 3` and pipes PNG data.
pub async fn run_tesseract_ocr(png_bytes: &[u8]) -> anyhow::Result<String> {
    use tokio::io::AsyncWriteExt;
    use tokio::process::Command;

    let mut child = Command::new("tesseract")
        .args(["stdin", "stdout", "--psm", "3", "-l", "eng"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn tesseract: {e}. Install with: brew install tesseract"))?;

    // Write PNG to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(png_bytes).await?;
    }

    let output = child.wait_with_output().await?;

    if !output.status.success() {
        anyhow::bail!("tesseract exited with status: {}", output.status);
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(text)
}

/// Layer 5 extractor — screenshot + OCR.
///
/// Only available with `--features headless`.
#[cfg(feature = "headless")]
pub struct Layer5Extractor {
    pool: std::sync::Arc<crate::browser::pool::BrowserPool>,
}

#[cfg(feature = "headless")]
use crate::extract::{ContentMetadata, ExtractedContent};
#[cfg(feature = "headless")]
use crate::prelude::CepLayer;
#[cfg(feature = "headless")]
use crate::token::count_tokens;

#[cfg(feature = "headless")]
impl Layer5Extractor {
    pub fn new(pool: std::sync::Arc<crate::browser::pool::BrowserPool>) -> Self {
        Self { pool }
    }

    /// Extract content via screenshot OCR.
    pub async fn extract(&self, url: &str) -> anyhow::Result<ExtractedContent> {
        if !tesseract_available() {
            anyhow::bail!(
                "Layer 5 OCR requires tesseract. Install: brew install tesseract (macOS) or apt install tesseract-ocr (Debian)"
            );
        }

        let tab = self
            .pool
            .acquire_tab()
            .await
            .map_err(|e| anyhow::anyhow!("L5: tab acquire failed: {e}"))?;

        tab.navigate(url, 20_000)
            .await
            .map_err(|e| anyhow::anyhow!("L5: navigate failed: {e}"))?;

        // Scroll to trigger lazy loading
        tab.evaluate("window.scrollTo(0, document.body.scrollHeight)")
            .await
            .ok(); // Ignore errors (some pages block scrollTo)

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        // Take full-page screenshot
        use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotParams;
        let screenshot_params = CaptureScreenshotParams::default();
        let png_bytes = tab
            .page()
            .screenshot(screenshot_params)
            .await
            .map_err(|e| anyhow::anyhow!("L5: screenshot failed: {e}"))?;

        // Run OCR
        let text = run_tesseract_ocr(&png_bytes)
            .await
            .map_err(|e| anyhow::anyhow!("L5: OCR failed: {e}"))?;

        if text.is_empty() {
            anyhow::bail!("L5: OCR produced no text");
        }

        let token_count = count_tokens(&text);

        Ok(ExtractedContent {
            title: url.rsplit('/').next().unwrap_or("page").to_string(),
            text,
            layer_used: CepLayer::Layer5,
            tokens: token_count,
            metadata: ContentMetadata {
                content_type: "image/png+ocr".to_string(),
                ..Default::default()
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tesseract_check_does_not_panic() {
        // Just verify it doesn't panic (tesseract may or may not be installed)
        let _ = tesseract_available();
    }
}
