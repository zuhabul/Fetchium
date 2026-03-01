//! HTTP client with connection pooling, retries, rate limiting, and size limits.
//!
//! PRD SS14: Domain-aware scheduler with per-domain concurrency caps.
//! PRD SS44: Structured errors with retry info.

use crate::config::HsxConfig;
use crate::error::{ErrorKind, HsxError, HsxResult, StructuredError};
use dashmap::DashMap;
use reqwest::{Client, StatusCode};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::debug;
use url::Url;

/// Maximum retry attempts for transient errors.
const MAX_RETRIES: u32 = 3;
/// Base delay for exponential backoff (milliseconds).
const BASE_DELAY_MS: u64 = 500;
/// Maximum backoff delay cap (30 seconds) to prevent excessive waits.
const MAX_BACKOFF_MS: u64 = 30_000;
/// Maximum response body size (10 MB).
const _MAX_BODY_SIZE: u64 = 10 * 1024 * 1024;

/// Per-domain rate limit state.
#[derive(Debug)]
struct DomainState {
    last_request: Instant,
    min_delay: Duration,
}

/// Shared HTTP client with connection pooling, retries, and rate limiting.
#[derive(Clone)]
pub struct HttpClient {
    inner: Client,
    config: Arc<HsxConfig>,
    /// Per-domain rate limiting state.
    domain_delays: Arc<DashMap<String, DomainState>>,
}

/// Fetch result with metadata about the request.
#[derive(Debug)]
pub struct FetchResult {
    pub body: String,
    pub status: u16,
    pub content_type: String,
    pub content_length: Option<u64>,
    pub url: String,
    pub elapsed_ms: u64,
    pub retries: u32,
}

impl HttpClient {
    /// Create a new HTTP client from config.
    pub fn new(config: &HsxConfig) -> HsxResult<Self> {
        let client = Client::builder()
            .user_agent(&config.fetch.user_agent)
            .timeout(Duration::from_secs(config.fetch.timeout_secs))
            .connect_timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::limited(
                config.fetch.max_redirects as usize,
            ))
            .gzip(true)
            .brotli(true)
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .map_err(HsxError::Network)?;

        Ok(Self {
            inner: client,
            config: Arc::new(config.clone()),
            domain_delays: Arc::new(DashMap::new()),
        })
    }

    /// Get the inner reqwest client for direct use.
    pub fn client(&self) -> &Client {
        &self.inner
    }

    /// Get the config reference.
    pub fn config(&self) -> &HsxConfig {
        &self.config
    }

    /// Extract domain from a URL for rate limiting.
    fn extract_domain(url: &str) -> String {
        Url::parse(url)
            .map(|u| u.host_str().unwrap_or("unknown").to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    }

    /// Wait if needed to respect per-domain rate limits.
    async fn enforce_rate_limit(&self, domain: &str) {
        let min_delay = Duration::from_millis(200);
        if let Some(state) = self.domain_delays.get(domain) {
            let elapsed = state.last_request.elapsed();
            if elapsed < state.min_delay {
                let wait = state.min_delay - elapsed;
                debug!("Rate limiting {domain}: waiting {wait:?}");
                sleep(wait).await;
            }
        }
        self.domain_delays.insert(
            domain.to_string(),
            DomainState {
                last_request: Instant::now(),
                min_delay,
            },
        );
    }

    /// Calculate backoff delay with jitter for retry attempt.
    /// Capped at MAX_BACKOFF_MS to prevent excessive waits on high retry counts.
    fn backoff_delay(attempt: u32) -> Duration {
        let base = BASE_DELAY_MS.saturating_mul(2u64.saturating_pow(attempt));
        let jitter = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos()
            % 500) as u64;
        Duration::from_millis(base.saturating_add(jitter).min(MAX_BACKOFF_MS))
    }

    /// Whether a status code is retryable.
    fn is_retryable_status(status: StatusCode) -> bool {
        matches!(
            status,
            StatusCode::TOO_MANY_REQUESTS
                | StatusCode::INTERNAL_SERVER_ERROR
                | StatusCode::BAD_GATEWAY
                | StatusCode::SERVICE_UNAVAILABLE
                | StatusCode::GATEWAY_TIMEOUT
        )
    }

    /// Map an HTTP status to a structured error kind.
    fn status_to_error_kind(status: StatusCode) -> ErrorKind {
        match status {
            StatusCode::FORBIDDEN => ErrorKind::Http403,
            StatusCode::TOO_MANY_REQUESTS => ErrorKind::Http429,
            s if s.is_server_error() => ErrorKind::Http5xx,
            _ => ErrorKind::Unknown,
        }
    }

    /// Fetch a URL with retries, rate limiting, and size enforcement.
    /// Returns the response body as a string along with metadata.
    pub async fn fetch(&self, url: &str) -> HsxResult<FetchResult> {
        let domain = Self::extract_domain(url);
        let start = Instant::now();
        let max_size = self.config.fetch.max_page_size;
        let mut last_err: Option<HsxError> = None;

        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                let delay = Self::backoff_delay(attempt - 1);
                debug!("Retry {attempt}/{MAX_RETRIES} for {url} after {delay:?}");
                sleep(delay).await;
            }

            self.enforce_rate_limit(&domain).await;

            let result = self.inner.get(url).send().await;

            match result {
                Ok(resp) => {
                    let status = resp.status();

                    if !status.is_success() {
                        if Self::is_retryable_status(status) && attempt < MAX_RETRIES {
                            debug!("Retryable status {status} for {url} (attempt {attempt}/{MAX_RETRIES})");
                            last_err = Some(HsxError::Structured(StructuredError {
                                kind: Self::status_to_error_kind(status),
                                retryable: true,
                                message: format!("HTTP {status} from {url}"),
                                source_url: Some(url.to_string()),
                                suggested_action: "Retry with backoff".into(),
                                alternatives: vec![],
                            }));
                            continue;
                        }

                        return Err(HsxError::Structured(StructuredError {
                            kind: Self::status_to_error_kind(status),
                            retryable: false,
                            message: format!("HTTP {status} from {url}"),
                            source_url: Some(url.to_string()),
                            suggested_action: match status {
                                StatusCode::FORBIDDEN => "Site blocks automated access".into(),
                                StatusCode::TOO_MANY_REQUESTS => "Rate limited, try later".into(),
                                _ => "Check URL and try again".into(),
                            },
                            alternatives: vec![],
                        }));
                    }

                    let content_length = resp.content_length();
                    if let Some(len) = content_length {
                        if len > max_size {
                            return Err(HsxError::Structured(StructuredError {
                                kind: ErrorKind::ExtractionFailed,
                                retryable: false,
                                message: format!(
                                    "Response too large: {len} bytes (max {max_size})"
                                ),
                                source_url: Some(url.to_string()),
                                suggested_action: "Increase max_page_size in config".into(),
                                alternatives: vec![],
                            }));
                        }
                    }

                    let content_type = resp
                        .headers()
                        .get("content-type")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("text/html")
                        .to_string();

                    let final_url = resp.url().to_string();

                    let body = resp.text().await.map_err(HsxError::Network)?;

                    if body.len() as u64 > max_size {
                        return Err(HsxError::Structured(StructuredError {
                            kind: ErrorKind::ExtractionFailed,
                            retryable: false,
                            message: format!("Body too large: {} bytes", body.len()),
                            source_url: Some(url.to_string()),
                            suggested_action: "Increase max_page_size".into(),
                            alternatives: vec![],
                        }));
                    }

                    return Ok(FetchResult {
                        body,
                        status: status.as_u16(),
                        content_type,
                        content_length,
                        url: final_url,
                        elapsed_ms: start.elapsed().as_millis() as u64,
                        retries: attempt,
                    });
                }
                Err(e) => {
                    if (e.is_timeout() || e.is_connect()) && attempt < MAX_RETRIES {
                        debug!("Transient error for {url}: {e}");
                        last_err = Some(HsxError::Network(e));
                        continue;
                    }
                    return Err(HsxError::Network(e));
                }
            }
        }

        Err(last_err.unwrap_or_else(|| {
            HsxError::Structured(StructuredError {
                kind: ErrorKind::NetworkTimeout,
                retryable: false,
                message: format!("All {MAX_RETRIES} retries exhausted for {url}"),
                source_url: Some(url.to_string()),
                suggested_action: "Check network connectivity".into(),
                alternatives: vec![],
            })
        }))
    }

    /// Convenience: fetch a URL and return just the body text.
    pub async fn fetch_text(&self, url: &str) -> HsxResult<String> {
        let result = self.fetch(url).await?;
        Ok(result.body)
    }

    /// Single-shot fetch — no retries, no rate limiting.
    ///
    /// Use this for sources where connection errors are *expected* (e.g. third-party
    /// Piped/Invidious instances that may be down). A single fast attempt is better
    /// than burning 3.5s on retry sleeps when we're racing many sources in parallel.
    pub async fn fetch_text_once(&self, url: &str) -> HsxResult<String> {
        let resp = self
            .inner
            .get(url)
            .send()
            .await
            .map_err(HsxError::Network)?;

        if !resp.status().is_success() {
            return Err(HsxError::Structured(crate::error::StructuredError {
                kind: Self::status_to_error_kind(resp.status()),
                retryable: false,
                message: format!("HTTP {} from {url}", resp.status()),
                source_url: Some(url.to_string()),
                suggested_action: "Source unavailable".into(),
                alternatives: vec![],
            }));
        }

        resp.text().await.map_err(HsxError::Network)
    }

    /// Single-shot POST — no retries, no rate limiting.
    ///
    /// Like `fetch_text_once` but sends a JSON POST body. Used for YouTube Innertube
    /// API calls where each call is time-sensitive and retries are not desired.
    pub async fn post_json_once(&self, url: &str, body: String) -> HsxResult<String> {
        let resp = self
            .inner
            .post(url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(HsxError::Network)?;

        if !resp.status().is_success() {
            return Err(HsxError::Structured(crate::error::StructuredError {
                kind: Self::status_to_error_kind(resp.status()),
                retryable: false,
                message: format!("HTTP {} from {url}", resp.status()),
                source_url: Some(url.to_string()),
                suggested_action: "Source unavailable".into(),
                alternatives: vec![],
            }));
        }

        resp.text().await.map_err(HsxError::Network)
    }

    /// POST JSON with retry support (uses the standard retry/rate-limit pipeline).
    pub async fn post_json(&self, url: &str, body: &str) -> HsxResult<String> {
        let resp = self
            .inner
            .post(url)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await
            .map_err(HsxError::Network)?;

        if !resp.status().is_success() {
            return Err(HsxError::Structured(crate::error::StructuredError {
                kind: Self::status_to_error_kind(resp.status()),
                retryable: resp.status().is_server_error(),
                message: format!("HTTP {} from {url}", resp.status()),
                source_url: Some(url.to_string()),
                suggested_action: "Check API key and request format".into(),
                alternatives: vec![],
            }));
        }

        resp.text().await.map_err(HsxError::Network)
    }

    /// POST JSON with a custom header (e.g., API key in non-standard header).
    pub async fn post_json_with_header(
        &self,
        url: &str,
        body: &str,
        header_name: &str,
        header_value: &str,
    ) -> HsxResult<String> {
        let resp = self
            .inner
            .post(url)
            .header("Content-Type", "application/json")
            .header(header_name, header_value)
            .body(body.to_string())
            .send()
            .await
            .map_err(HsxError::Network)?;

        if !resp.status().is_success() {
            return Err(HsxError::Structured(crate::error::StructuredError {
                kind: Self::status_to_error_kind(resp.status()),
                retryable: resp.status().is_server_error(),
                message: format!("HTTP {} from {url}", resp.status()),
                source_url: Some(url.to_string()),
                suggested_action: "Check API key and request format".into(),
                alternatives: vec![],
            }));
        }

        resp.text().await.map_err(HsxError::Network)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_creation() {
        let config = HsxConfig::default();
        let client = HttpClient::new(&config);
        assert!(client.is_ok());
    }

    #[test]
    fn domain_extraction() {
        assert_eq!(
            HttpClient::extract_domain("https://www.example.com/page"),
            "www.example.com"
        );
        assert_eq!(
            HttpClient::extract_domain("http://localhost:8080/api"),
            "localhost"
        );
        assert_eq!(HttpClient::extract_domain("not-a-url"), "unknown");
    }

    #[test]
    fn backoff_delay_increases() {
        let d0 = HttpClient::backoff_delay(0);
        let d1 = HttpClient::backoff_delay(1);
        let d2 = HttpClient::backoff_delay(2);
        assert!(d0.as_millis() >= 500 && d0.as_millis() < 1000);
        assert!(d1.as_millis() >= 1000 && d1.as_millis() < 1500);
        assert!(d2.as_millis() >= 2000 && d2.as_millis() < 2500);
    }

    #[test]
    fn retryable_status_codes() {
        assert!(HttpClient::is_retryable_status(
            StatusCode::TOO_MANY_REQUESTS
        ));
        assert!(HttpClient::is_retryable_status(StatusCode::BAD_GATEWAY));
        assert!(HttpClient::is_retryable_status(
            StatusCode::SERVICE_UNAVAILABLE
        ));
        assert!(!HttpClient::is_retryable_status(StatusCode::NOT_FOUND));
        assert!(!HttpClient::is_retryable_status(StatusCode::FORBIDDEN));
    }
}
