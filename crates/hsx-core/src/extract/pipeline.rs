//! CEP extraction pipeline — orchestrates layer selection and escalation.
//!
//! PRD SS16: 5-layer cascade with auto-escalation. Phase 1 implements L1-L2.
//! PRD SS8.3: ML-predicted method selection (stubbed for Phase 1,
//!            implemented in Phase 5).

use crate::extract::layer1;
use crate::extract::layer2;
use crate::extract::ExtractedContent;
use crate::types::CepLayer;
use tracing::{debug, info};

/// Maximum CEP layer available in the current build.
/// Phase 1: layers 1-2 only. Layers 3-5 added in Phase 2.
pub const MAX_AVAILABLE_LAYER: CepLayer = CepLayer::Layer2;

/// Run the CEP extraction pipeline on raw HTML.
///
/// Starts at Layer 1 and escalates to Layer 2 if insufficient content
/// is extracted. Returns the best extraction result.
pub fn extract(html: &str, url: &str) -> ExtractedContent {
    let l1_result = layer1::extract(html, url);

    if layer1::is_sufficient(&l1_result, html) {
        info!(
            "CEP Layer1 sufficient for {} ({} tokens)",
            url, l1_result.tokens
        );
        return l1_result;
    }

    debug!(
        "CEP Layer1 insufficient for {} ({} chars), escalating to Layer2",
        url,
        l1_result.text.len()
    );

    let l2_result = layer2::extract(html, url);

    if layer2::is_sufficient(&l2_result) {
        info!(
            "CEP Layer2 sufficient for {} ({} tokens)",
            url, l2_result.tokens
        );
        return l2_result;
    }

    debug!(
        "CEP Layer2 insufficient for {} ({} chars), returning best result",
        url,
        l2_result.text.len()
    );

    if l2_result.text.len() > l1_result.text.len() {
        l2_result
    } else {
        l1_result
    }
}

/// Predict the best CEP layer for a given URL (stub for Phase 1).
///
/// In Phase 5, this will use an ML classifier trained on URL patterns,
/// domain, content-type, and HTML structural features.
pub fn predict_layer(_url: &str, _content_type: &str) -> CepLayer {
    CepLayer::Layer1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_uses_layer1_for_good_content() {
        let html = r#"
        <html>
        <head><title>Good Page</title></head>
        <body>
        <article>
            <h1>Well Structured Article</h1>
            <p>This article has plenty of well-structured content that
               should be easily extractable by Layer 1. The CSS selectors
               will find the article tag and extract all the paragraphs
               within it. This should pass the minimum content threshold
               without needing to escalate to Layer 2.</p>
            <p>Second paragraph with additional details and information
               that further enriches the article content for readers.</p>
        </article>
        </body>
        </html>
        "#;

        let result = extract(html, "https://example.com/good");
        assert_eq!(result.layer_used, CepLayer::Layer1);
        assert!(result.text.contains("Well Structured"));
    }

    #[test]
    fn pipeline_escalates_for_poor_content() {
        let html = "<html><body><p>x</p></body></html>";
        let result = extract(html, "https://example.com/poor");
        assert!(
            result.layer_used == CepLayer::Layer1
                || result.layer_used == CepLayer::Layer2
        );
    }
}
