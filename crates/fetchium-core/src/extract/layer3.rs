//! CEP Layer 3 — Headless Chromium JavaScript rendering (PRD §8.3).
//!
//! Triggered when Layers 1-2 detect insufficient content (SPAs, lazy-loaded content).
//! Waits for network idle + JS execution before extracting.
//!
//! Requires `--features headless`.

/// Minimum relative content gain to consider Layer 3 beneficial.
/// If rendered HTML is <30% larger than static HTML, escalation wasn't needed.
const CONTENT_DIFF_THRESHOLD: f64 = 0.30;

/// Check whether headless rendering produced meaningfully more content.
pub fn was_beneficial(static_len: usize, rendered_len: usize) -> bool {
    if static_len == 0 {
        return true;
    }
    (rendered_len as f64 - static_len as f64) / static_len as f64 > CONTENT_DIFF_THRESHOLD
}

/// Layer 3 extractor — render page via headless Chromium, then apply Layer 2 extraction.
///
/// Only available with `--features headless`.
#[cfg(feature = "headless")]
pub struct Layer3Extractor {
    pool: std::sync::Arc<crate::browser::pool::BrowserPool>,
}

#[cfg(feature = "headless")]
use crate::extract::ExtractedContent;
#[cfg(feature = "headless")]
use crate::prelude::CepLayer;

#[cfg(feature = "headless")]
impl Layer3Extractor {
    pub fn new(pool: std::sync::Arc<crate::browser::pool::BrowserPool>) -> Self {
        Self { pool }
    }

    /// Extract content from a URL via headless rendering.
    ///
    /// Navigates to the URL, waits 500ms for JS to settle, then runs
    /// Layer 2 extraction on the fully rendered HTML.
    pub async fn extract(&self, url: &str) -> anyhow::Result<ExtractedContent> {
        let tab = self
            .pool
            .acquire_tab()
            .await
            .map_err(|e| anyhow::anyhow!("L3: tab acquire failed: {e}"))?;

        tab.navigate(url, 15_000)
            .await
            .map_err(|e| anyhow::anyhow!("L3: navigate failed: {e}"))?;

        // Wait for JS execution and network idle
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let rendered_html = tab
            .content()
            .await
            .map_err(|e| anyhow::anyhow!("L3: content fetch failed: {e}"))?;

        // Reuse Layer 2 extraction on rendered HTML
        let mut result = crate::extract::layer2::extract(&rendered_html, url);
        result.layer_used = CepLayer::Layer3;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn was_beneficial_empty_static() {
        assert!(was_beneficial(0, 1000));
    }

    #[test]
    fn was_beneficial_large_gain() {
        assert!(was_beneficial(1000, 2000)); // 100% gain > 30% threshold
    }

    #[test]
    fn was_beneficial_small_gain() {
        assert!(!was_beneficial(1000, 1100)); // 10% gain < 30% threshold
    }

    #[test]
    fn was_beneficial_exact_threshold() {
        // 30% gain is exactly at the boundary (not beneficial — must exceed)
        assert!(!was_beneficial(1000, 1300));
    }

    #[test]
    fn was_beneficial_large_improvement() {
        assert!(was_beneficial(500, 5000)); // 10x gain = SPA with lots of JS content
    }
}
