//! CEP layer predictor — pure Rust decision tree (Phase 5, PRD §8.3).
//!
//! Predicts the required CEP extraction layer (1–5) for a URL *before* attempting
//! extraction, saving latency by skipping layers that are unlikely to succeed.
//!
//! When historical data is available (from PIE), it takes priority.
//! Otherwise, the decision tree uses URL features (script density, SPA frameworks, etc.).

use tracing::debug;

/// Features extracted from a URL for layer prediction.
#[derive(Debug, Clone, Default)]
pub struct CepFeatures {
    /// Domain is in the known SPA list (React, Vue, Angular, Next.js, etc.).
    pub domain_is_known_spa: bool,
    /// URL path contains hash routing (`/#/` or `/#!`).
    pub path_has_hash_routing: bool,
    /// Number of `<script>` tags in the first 10 KB of HTML.
    pub script_tag_count: u32,
    /// Text-to-HTML character ratio of the first 10 KB (0.0–1.0).
    pub text_to_html_ratio: f32,
    /// True if the initial HTML contains framework markers
    /// (`__next`, `__nuxt`, `ng-app`, `data-reactroot`, `id="__svelte"`).
    pub has_framework_markers: bool,
    /// True if a `<noscript>` tag is present (strong SPA signal).
    pub has_noscript_tag: bool,
    /// Layer that worked last time for this domain, if known.
    pub historical_layer: Option<u8>,
    /// Historical success rate for this domain+layer pair (0.0–1.0).
    pub historical_success_rate: f32,
}

/// Known SPA/JavaScript-heavy domains that require Layer 3+.
const SPA_DOMAINS: &[&str] = &[
    "twitter.com", "x.com", "facebook.com", "instagram.com",
    "tiktok.com", "linkedin.com", "discord.com", "slack.com",
    "notion.so", "figma.com", "airtable.com", "trello.com",
    "app.github.com", "gitlab.com", "jira.atlassian.net",
    "app.netlify.com", "vercel.app", "app.render.com",
    "grafana.com", "kibana.io",
];

/// Framework markers that indicate a JavaScript SPA.
const FRAMEWORK_MARKERS: &[&str] = &[
    "__next", "__nuxt", "ng-app", "data-reactroot", "__svelte",
    "v-app", "ember-application", "__quasar",
];

/// Extract `CepFeatures` from a domain + initial HTML peek.
///
/// The HTML peek should be the first ~10 KB of the page.
pub fn extract_features(domain: &str, url: &str, html_peek: &str) -> CepFeatures {
    CepFeatures {
        domain_is_known_spa: is_known_spa(domain),
        path_has_hash_routing: url.contains("/#/") || url.contains("/#!"),
        script_tag_count: count_script_tags(html_peek),
        text_to_html_ratio: compute_text_ratio(html_peek),
        has_framework_markers: has_framework_markers(html_peek),
        has_noscript_tag: html_peek.to_lowercase().contains("<noscript"),
        historical_layer: None,
        historical_success_rate: 1.0,
    }
}

/// Predict the optimal starting CEP layer for the given features.
///
/// Returns a layer number 1–5:
/// - 1 = Static HTML (CSS selectors) — fastest
/// - 2 = HTML + readability — for article pages
/// - 3 = Headless Chrome — SPAs and heavy JS
/// - 4 = PDF/document extraction
/// - 5 = Screenshot OCR — last resort
pub fn predict_layer(features: &CepFeatures) -> u8 {
    // Priority 1: Trust historical data when success rate is high
    if let Some(layer) = features.historical_layer {
        if features.historical_success_rate > 0.8 {
            debug!("CEP predictor: using historical layer {layer}");
            return layer;
        }
    }

    // Priority 2: Known SPA domain or framework markers
    if features.domain_is_known_spa || features.has_framework_markers {
        if features.path_has_hash_routing || features.has_noscript_tag {
            debug!("CEP predictor: hash-routed or noscript SPA → Layer 3");
            return 3;
        }
        debug!("CEP predictor: SPA/framework detected → Layer 3");
        return 3;
    }

    // Priority 3: Script density heuristic
    if features.script_tag_count > 20 {
        debug!(
            "CEP predictor: heavy JS ({} scripts) → Layer 3",
            features.script_tag_count
        );
        return 3;
    }
    if features.script_tag_count > 10 {
        debug!(
            "CEP predictor: moderate JS ({} scripts) → Layer 2",
            features.script_tag_count
        );
        return 2;
    }

    // Priority 4: Text ratio heuristic
    if features.text_to_html_ratio > 0.3 {
        debug!("CEP predictor: high text ratio → Layer 1");
        return 1;
    }
    if features.text_to_html_ratio > 0.1 {
        debug!("CEP predictor: medium text ratio → Layer 2");
        return 2;
    }

    // Low text ratio + not SPA = probably a portal/dynamic page
    debug!("CEP predictor: low text ratio → Layer 2 (default)");
    2
}

fn is_known_spa(domain: &str) -> bool {
    SPA_DOMAINS
        .iter()
        .any(|spa| domain == *spa || domain.ends_with(&format!(".{spa}")))
}

fn count_script_tags(html: &str) -> u32 {
    let lower = html.to_lowercase();
    lower.matches("<script").count() as u32
}

fn compute_text_ratio(html: &str) -> f32 {
    if html.is_empty() {
        return 0.0;
    }
    // Strip all HTML tags (naive approximation for feature extraction only)
    let mut in_tag = false;
    let mut text_len = 0usize;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text_len += 1,
            _ => {}
        }
    }
    text_len as f32 / html.len() as f32
}

fn has_framework_markers(html: &str) -> bool {
    let lower = html.to_lowercase();
    FRAMEWORK_MARKERS.iter().any(|marker| lower.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predict_layer_static_html() {
        let features = CepFeatures {
            text_to_html_ratio: 0.4,
            script_tag_count: 2,
            ..Default::default()
        };
        assert_eq!(predict_layer(&features), 1);
    }

    #[test]
    fn predict_layer_heavy_js() {
        let features = CepFeatures {
            script_tag_count: 25,
            text_to_html_ratio: 0.05,
            ..Default::default()
        };
        assert_eq!(predict_layer(&features), 3);
    }

    #[test]
    fn predict_layer_known_spa() {
        let features = CepFeatures {
            domain_is_known_spa: true,
            ..Default::default()
        };
        assert_eq!(predict_layer(&features), 3);
    }

    #[test]
    fn predict_layer_historical_wins() {
        let features = CepFeatures {
            historical_layer: Some(1),
            historical_success_rate: 0.95,
            domain_is_known_spa: true, // would normally → Layer 3
            ..Default::default()
        };
        assert_eq!(predict_layer(&features), 1, "historical should override");
    }

    #[test]
    fn predict_layer_low_history_success_falls_through() {
        let features = CepFeatures {
            historical_layer: Some(1),
            historical_success_rate: 0.3, // below threshold
            script_tag_count: 25,
            ..Default::default()
        };
        // Historical below threshold → falls through to script density → Layer 3
        assert_eq!(predict_layer(&features), 3);
    }

    #[test]
    fn is_known_spa_twitter() {
        assert!(is_known_spa("twitter.com"));
        assert!(is_known_spa("x.com"));
        assert!(!is_known_spa("example.com"));
    }

    #[test]
    fn count_script_tags_correct() {
        let html = "<script src='a.js'></script><script>x</script><p>text</p>";
        assert_eq!(count_script_tags(html), 2);
    }

    #[test]
    fn compute_text_ratio_reasonable() {
        let html = "<p>Hello world</p>";
        let ratio = compute_text_ratio(html);
        assert!(ratio > 0.3 && ratio < 1.0, "ratio={ratio}");
    }

    #[test]
    fn has_framework_markers_detects_next() {
        let html = "<div id='__next'></div>";
        assert!(has_framework_markers(html));
        assert!(!has_framework_markers("<div>plain html</div>"));
    }

    #[test]
    fn extract_features_from_html_peek() {
        let html = "<html><script src='app.js'></script><div id='__next'><p>content here</p></div></html>";
        let features = extract_features("example.com", "https://example.com/page", html);
        assert!(features.has_framework_markers);
        assert_eq!(features.script_tag_count, 1);
    }
}
