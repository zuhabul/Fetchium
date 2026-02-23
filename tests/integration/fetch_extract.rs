// Integration: fetch + extract pipeline tests using wiremock.

use hsx_core::config::HsxConfig;
use hsx_core::extract::layer1;
use hsx_core::http::HttpClient;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn load_fixture(name: &str) -> String {
    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name);
    std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("fixture '{name}' not found: {e}"))
}

#[tokio::test]
async fn fetch_returns_html_body() {
    let server = MockServer::start().await;
    let html = load_fixture("simple-article.html");

    Mock::given(method("GET"))
        .and(path("/article"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html))
        .mount(&server)
        .await;

    let config = HsxConfig::default();
    let client = HttpClient::new(&config).unwrap();
    let url = format!("{}/article", server.uri());
    let result = client.fetch(&url).await.unwrap();

    assert_eq!(result.status, 200);
    assert!(result.body.contains("Rust Ownership"));
}

#[tokio::test]
async fn fetch_404_returns_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/missing"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let config = HsxConfig::default();
    let client = HttpClient::new(&config).unwrap();
    let url = format!("{}/missing", server.uri());
    let result = client.fetch(&url).await;

    assert!(result.is_err(), "404 should return an error");
}

#[test]
fn layer1_extracts_article_content() {
    let html = load_fixture("simple-article.html");
    let content = layer1::extract(&html, "https://example.com/article");

    assert!(
        content.text.contains("ownership system"),
        "should extract main article text"
    );
    assert!(
        !content.text.contains("Navigation links"),
        "should strip nav boilerplate"
    );
    assert_eq!(content.title, "Understanding Rust Ownership");
}

#[test]
fn layer1_extracts_table_content() {
    let html = load_fixture("table-heavy.html");
    let content = layer1::extract(&html, "https://benchmarks.example.com");

    assert!(
        content.text.contains("Actix") || content.text.contains("Axum"),
        "should extract table content"
    );
    assert!(
        content.title.contains("Benchmark") || content.title.contains("Framework"),
        "title should reflect page content"
    );
}

#[test]
fn layer1_handles_spa_shell_gracefully() {
    let html = load_fixture("spa-shell.html");
    let content = layer1::extract(&html, "https://myapp.example.com");

    // SPA shell has minimal text — should not panic, just return minimal content
    assert!(
        content.text.len() < 200,
        "SPA shell should produce minimal text: got {} chars",
        content.text.len()
    );
}

#[test]
fn layer1_does_not_panic_on_empty_html() {
    let content = layer1::extract("", "https://example.com");
    // Should return empty/minimal content, not panic
    assert!(content.tokens == 0 || content.text.is_empty());
}

#[test]
fn layer1_does_not_panic_on_malformed_html() {
    let malformed = "<html><head><title>Test</title><body><p>Unclosed paragraph<div>Nested";
    let content = layer1::extract(malformed, "https://example.com");
    // Must not panic; may or may not extract text
    let _ = content;
}
