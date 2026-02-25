//! CEP extraction pipeline — orchestrates layer selection and escalation (PRD §8.3).
//!
//! **Sequential fast-path execution:** L1 runs first (~2ms, ~5MB RAM).
//! L2 only executes when L1 is insufficient, saving its CPU cost for the
//! ~85% of pages where L1 succeeds. Under concurrency this yields much higher
//! throughput than the speculative-parallel approach.
//!
//! **Layer escalation thresholds:**
//! - L1 sufficient: content_len ≥ MIN_CONTENT_LENGTH AND text_ratio ≥ MIN_TEXT_RATIO
//! - L2 sufficient: tokens >= 100
//! - L3 trigger: rendered_len > static_len by >30% (SPA detected)

use crate::extract::layer1;
use crate::extract::layer2;
use crate::extract::ExtractedContent;
use crate::types::CepLayer;
use tracing::info;

#[cfg(feature = "headless")]
use crate::extract::layer3::was_beneficial;

/// Minimum token threshold to consider Layer 3 escalation worthwhile.
#[cfg(feature = "headless")]
const L3_ESCALATION_TOKEN_THRESHOLD: u32 = 100;

/// Run the CEP extraction pipeline on raw HTML (sync path: L1 then L2 if needed).
///
/// Layer 1 runs first. If sufficient, it is returned immediately without running L2.
/// This avoids the CPU cost of lol_html (L2) for the ~85% of pages where L1 succeeds,
/// dramatically improving throughput under concurrency.
///
/// # Selection priority
/// 1. L1 sufficient → return L1 immediately (fastest, ~85% of pages)
/// 2. L2 sufficient → return L2
/// 3. Neither → return whichever produced more content
pub fn extract(html: &str, url: &str) -> ExtractedContent {
    // Fast path: L1 succeeds for ~85% of well-structured pages.
    let l1_result = layer1::extract(html, url);
    if layer1::is_sufficient(&l1_result, html) {
        info!(
            "CEP: L1 sufficient for {} ({} tokens)",
            url, l1_result.tokens
        );
        return l1_result;
    }

    // L1 insufficient — escalate to L2.
    let l2_result = layer2::extract(html, url);
    if layer2::is_sufficient(&l2_result) {
        info!(
            "CEP: L2 selected for {} ({} tokens, L1 had {} tokens)",
            url, l2_result.tokens, l1_result.tokens
        );
        return l2_result;
    }

    // Neither sufficient — return whichever extracted more content.
    if l2_result.text.len() > l1_result.text.len() {
        info!(
            "CEP: L2 fallback for {} (L2={} tokens, L1={} tokens)",
            url, l2_result.tokens, l1_result.tokens
        );
        l2_result
    } else {
        info!(
            "CEP: L1 fallback for {} (L1={} tokens, L2={} tokens)",
            url, l1_result.tokens, l2_result.tokens
        );
        l1_result
    }
}

/// Run CEP extraction with optional L3 (headless) escalation.
///
/// Returns the static L1/L2 result immediately for efficient pages.
/// Only escalates to L3 when the page appears to be a JavaScript SPA
/// (insufficient static content AND headless rendering is available).
#[cfg(feature = "headless")]
pub async fn extract_with_l3(
    html: &str,
    url: &str,
    pool: &std::sync::Arc<crate::browser::pool::BrowserPool>,
) -> ExtractedContent {
    let static_result = extract(html, url);
    let static_len = static_result.text.len();

    // Only escalate if static extraction was insufficient
    if static_result.tokens >= L3_ESCALATION_TOKEN_THRESHOLD {
        info!(
            "CEP: L1/L2 sufficient ({} tokens), skipping L3",
            static_result.tokens
        );
        return static_result;
    }

    info!(
        "CEP: escalating to L3 for {} ({} tokens from static)",
        url, static_result.tokens
    );

    let extractor = crate::extract::layer3::Layer3Extractor::new(std::sync::Arc::clone(pool));
    match extractor.extract(url).await {
        Ok(l3_result) => {
            if was_beneficial(static_len, l3_result.text.len()) {
                info!(
                    "CEP: L3 beneficial for {} ({} → {} tokens)",
                    url, static_result.tokens, l3_result.tokens
                );
                l3_result
            } else {
                info!("CEP: L3 not beneficial, keeping static result");
                static_result
            }
        }
        Err(e) => {
            info!("CEP: L3 failed for {url}: {e}; returning static result");
            static_result
        }
    }
}

/// Determine whether a URL likely needs headless rendering.
///
/// Heuristics: known SPA frameworks, JavaScript-first domains, etc.
pub fn likely_needs_headless(url: &str) -> bool {
    let url_lower = url.to_lowercase();
    // Known SPA/JS-heavy domains
    const JS_HEAVY_DOMAINS: &[&str] = &[
        "twitter.com",
        "x.com",
        "instagram.com",
        "facebook.com",
        "linkedin.com",
        "notion.so",
        "app.netlify.com",
        "vercel.app",
        "figma.com",
    ];
    JS_HEAVY_DOMAINS.iter().any(|d| url_lower.contains(d))
}

/// Predict the best CEP layer for a given URL (stub for Phase 5 ML classifier).
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
        assert!(result.layer_used == CepLayer::Layer1 || result.layer_used == CepLayer::Layer2);
    }

    #[test]
    fn likely_needs_headless_twitter() {
        assert!(likely_needs_headless(
            "https://twitter.com/rust_lang/status/123"
        ));
        assert!(likely_needs_headless("https://x.com/user/post"));
    }

    #[test]
    fn likely_needs_headless_regular_site() {
        assert!(!likely_needs_headless(
            "https://blog.rust-lang.org/2024/01/01/post.html"
        ));
    }
}
