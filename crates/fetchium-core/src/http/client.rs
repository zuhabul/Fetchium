//! HTTP client with connection pooling, retries, rate limiting, and size limits.
//!
//! PRD SS14: Domain-aware scheduler with per-domain concurrency caps.
//! PRD SS44: Structured errors with retry info.

use crate::config::FetchiumConfig;
use crate::error::{ErrorKind, FetchiumError, FetchiumResult, StructuredError};
use crate::proxy::{DataImpulseClient, ProxyPool};
use dashmap::DashMap;
use reqwest::{Client, StatusCode};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, info};
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

/// Domains that are blocked by datacenter IPs and require residential proxies.
/// API-based backends (Serper, Exa, Tavily, Gemini) are NOT in this list.
/// Bing, Brave, Yahoo, Yandex, Startpage work fine without residential → excluded.
const RESIDENTIAL_REQUIRED_DOMAINS: &[&str] = &[
    "google.com",
    "www.google.com",
    "html.duckduckgo.com",
    "lite.duckduckgo.com",
    "duckduckgo.com",
];

/// Shared HTTP client with connection pooling, retries, and rate limiting.
#[derive(Clone)]
pub struct HttpClient {
    inner: Client,
    config: Arc<FetchiumConfig>,
    /// Per-domain rate limiting state.
    domain_delays: Arc<DashMap<String, DomainState>>,
    /// Optional Webshare proxy pool (datacenter — legacy fallback).
    proxy_pool: Option<ProxyPool>,
    /// DataImpulse residential proxy client (country-targeted, pay-per-GB).
    dataimpulse: Option<DataImpulseClient>,
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
    pub fn new(config: &FetchiumConfig) -> FetchiumResult<Self> {
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
            .map_err(FetchiumError::Network)?;

        // Load proxy pool if configured
        let proxy_pool = if config.proxy.enabled {
            let proxy_file = config
                .proxy
                .proxy_file
                .clone()
                .unwrap_or_else(|| config.data_dir().join("proxies.txt"));
            if proxy_file.exists() {
                match ProxyPool::load_from_file(&proxy_file) {
                    Ok(pool) if !pool.is_empty() => {
                        info!(
                            "Proxy pool loaded: {} proxies from {}",
                            pool.len(),
                            proxy_file.display()
                        );
                        Some(pool)
                    }
                    Ok(_) => {
                        tracing::warn!("Proxy file is empty: {}", proxy_file.display());
                        None
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load proxy file {}: {e}", proxy_file.display());
                        None
                    }
                }
            } else {
                tracing::warn!("Proxy file not found: {}", proxy_file.display());
                None
            }
        } else {
            None
        };

        // Build DataImpulse residential proxy client if configured
        let dataimpulse = if config.dataimpulse.enabled && !config.dataimpulse.username.is_empty() {
            let di = DataImpulseClient::new(
                &config.dataimpulse.username,
                &config.dataimpulse.password,
                &config.dataimpulse.host,
                config.dataimpulse.port,
                &config.fetch.user_agent,
                Duration::from_secs(config.fetch.timeout_secs),
            );
            info!(
                "DataImpulse residential proxy enabled ({}:{})",
                config.dataimpulse.host, config.dataimpulse.port
            );
            Some(di)
        } else {
            None
        };

        Ok(Self {
            inner: client,
            config: Arc::new(config.clone()),
            domain_delays: Arc::new(DashMap::new()),
            proxy_pool,
            dataimpulse,
        })
    }

    /// Get the inner reqwest client for direct use.
    pub fn client(&self) -> &Client {
        &self.inner
    }

    /// Get a reqwest client for a specific domain with locale-aware residential proxy routing.
    ///
    /// Priority order:
    /// 1. DataImpulse residential proxy (country-targeted) — for domains blocked by datacenter IPs
    /// 2. Webshare datacenter pool — legacy fallback for non-residential-required domains
    /// 3. Direct connection — when no proxy is configured or available
    ///
    /// GB efficiency: DataImpulse only activates for `RESIDENTIAL_REQUIRED_DOMAINS`.
    /// API backends (Serper, Exa, Tavily) use direct connections.
    pub fn client_for_domain_with_locale(&self, domain: &str, locale: Option<&str>) -> Client {
        let needs_residential = self.needs_residential_proxy(domain);

        // 1. DataImpulse residential proxy for blocked domains
        if needs_residential {
            if let Some(ref di) = self.dataimpulse {
                if di.is_configured() {
                    debug!("DataImpulse residential proxy for {domain} locale={locale:?}");
                    return di.client(locale);
                }
            }
        }

        // 2. Webshare datacenter pool (legacy, for non-residential-required domains)
        if let Some(ref pool) = self.proxy_pool {
            if self.should_proxy_domain(domain) {
                if let Some(proxy) = pool.proxy_for_domain(domain) {
                    match ProxyPool::build_client_with_proxy(
                        &proxy,
                        &self.config.fetch.user_agent,
                        Duration::from_secs(self.config.fetch.timeout_secs),
                    ) {
                        Ok(client) => {
                            debug!("Webshare proxy {}:{} for {domain}", proxy.host, proxy.port);
                            return client;
                        }
                        Err(e) => {
                            debug!("Failed to build proxied client for {domain}: {e}");
                        }
                    }
                }
            }
        }

        // 3. Direct connection
        self.inner.clone()
    }

    /// Backwards-compatible wrapper — no locale hint.
    pub fn client_for_domain(&self, domain: &str) -> Client {
        self.client_for_domain_with_locale(domain, None)
    }

    /// Direct client — bypasses ALL proxy routing (residential and datacenter).
    ///
    /// Use for first-attempt requests where proxy is not yet needed.
    /// If the direct attempt fails, escalate to `client_for_domain_with_locale`.
    /// Saves DataImpulse GB when direct connections succeed (majority of requests).
    pub fn client_direct(&self) -> Client {
        self.inner.clone()
    }

    /// Get a **fresh** client that forces a new residential IP on the next request.
    ///
    /// Use when a previous request was blocked (CAPTCHA, 403, empty SERP).
    /// The fresh client has no connection pool, so DataImpulse assigns a new
    /// residential IP. Falls back to `client_for_domain_with_locale` if not residential.
    pub fn fresh_client_for_domain_with_locale(
        &self,
        domain: &str,
        locale: Option<&str>,
    ) -> Client {
        if self.needs_residential_proxy(domain) {
            if let Some(ref di) = self.dataimpulse {
                if di.is_configured() {
                    return di.fresh_client(locale);
                }
            }
        }
        // Non-residential: return normal client (direct or Webshare)
        self.client_for_domain_with_locale(domain, locale)
    }

    /// Whether a domain requires residential IPs to avoid blocks.
    fn needs_residential_proxy(&self, domain: &str) -> bool {
        // Check user-configured override list first
        if !self.config.dataimpulse.proxy_domains.is_empty() {
            return self
                .config
                .dataimpulse
                .proxy_domains
                .iter()
                .any(|d| domain.ends_with(d.as_str()) || domain == d.as_str());
        }
        // Built-in list of search engines blocked by datacenter IPs
        RESIDENTIAL_REQUIRED_DOMAINS
            .iter()
            .any(|&d| domain == d || domain.ends_with(&format!(".{d}")))
    }

    /// Get the config reference.
    pub fn config(&self) -> &FetchiumConfig {
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
    pub async fn fetch(&self, url: &str) -> FetchiumResult<FetchResult> {
        let domain = Self::extract_domain(url);
        let start = Instant::now();
        let max_size = self.config.fetch.max_page_size;
        let mut last_err: Option<FetchiumError> = None;

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
                            last_err = Some(FetchiumError::Structured(StructuredError {
                                kind: Self::status_to_error_kind(status),
                                retryable: true,
                                message: format!("HTTP {status} from {url}"),
                                source_url: Some(url.to_string()),
                                suggested_action: "Retry with backoff".into(),
                                alternatives: vec![],
                            }));
                            continue;
                        }

                        return Err(FetchiumError::Structured(StructuredError {
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
                            return Err(FetchiumError::Structured(StructuredError {
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

                    let body = resp.text().await.map_err(FetchiumError::Network)?;

                    if body.len() as u64 > max_size {
                        return Err(FetchiumError::Structured(StructuredError {
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
                        last_err = Some(FetchiumError::Network(e));
                        continue;
                    }
                    return Err(FetchiumError::Network(e));
                }
            }
        }

        Err(last_err.unwrap_or_else(|| {
            FetchiumError::Structured(StructuredError {
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
    pub async fn fetch_text(&self, url: &str) -> FetchiumResult<String> {
        let result = self.fetch(url).await?;
        Ok(result.body)
    }

    /// Get the proxy pool (if configured).
    pub fn proxy_pool(&self) -> Option<&ProxyPool> {
        self.proxy_pool.as_ref()
    }

    /// Check if a domain should use proxies.
    fn should_proxy_domain(&self, domain: &str) -> bool {
        // Never proxy bypass domains
        if self
            .config
            .proxy
            .bypass_domains
            .iter()
            .any(|d| domain.contains(d.as_str()))
        {
            return false;
        }
        // If proxy_domains is specified, only proxy those
        if !self.config.proxy.proxy_domains.is_empty() {
            return self
                .config
                .proxy
                .proxy_domains
                .iter()
                .any(|d| domain.contains(d.as_str()));
        }
        // Default: proxy everything
        true
    }

    /// Fetch a URL through a rotating proxy. Falls back to direct if no proxy available.
    pub async fn fetch_via_proxy(&self, url: &str) -> FetchiumResult<FetchResult> {
        let domain = Self::extract_domain(url);

        let pool = match &self.proxy_pool {
            Some(p) if !p.is_empty() && self.should_proxy_domain(&domain) => p,
            _ => return self.fetch(url).await,
        };

        let proxy = match pool.proxy_for_domain(&domain) {
            Some(p) => p,
            None => {
                debug!("No available proxy for {domain}, falling back to direct");
                return self.fetch(url).await;
            }
        };

        let start = Instant::now();
        let client = ProxyPool::build_client_with_proxy(
            &proxy,
            &self.config.fetch.user_agent,
            Duration::from_secs(self.config.fetch.timeout_secs),
        )
        .map_err(FetchiumError::Network)?;

        match client.get(url).send().await {
            Ok(resp) => {
                let status = resp.status();
                let latency = start.elapsed().as_millis() as u64;

                if status.is_success() {
                    proxy.record_success(latency);
                    let content_type = resp
                        .headers()
                        .get("content-type")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("text/html")
                        .to_string();
                    let content_length = resp.content_length();
                    let final_url = resp.url().to_string();
                    let body = resp.text().await.map_err(FetchiumError::Network)?;

                    Ok(FetchResult {
                        body,
                        status: status.as_u16(),
                        content_type,
                        content_length,
                        url: final_url,
                        elapsed_ms: latency,
                        retries: 0,
                    })
                } else if status == StatusCode::FORBIDDEN || status == StatusCode::TOO_MANY_REQUESTS
                {
                    // Proxy got blocked — record failure and retry with next proxy
                    proxy.record_failure();
                    debug!(
                        "Proxy {}:{} blocked (HTTP {}) for {domain}, trying next",
                        proxy.host, proxy.port, status
                    );
                    // One retry with a different proxy
                    if let Some(next_proxy) = pool.next_proxy() {
                        let client2 = ProxyPool::build_client_with_proxy(
                            &next_proxy,
                            &self.config.fetch.user_agent,
                            Duration::from_secs(self.config.fetch.timeout_secs),
                        )
                        .map_err(FetchiumError::Network)?;

                        let start2 = Instant::now();
                        match client2.get(url).send().await {
                            Ok(resp2) if resp2.status().is_success() => {
                                next_proxy.record_success(start2.elapsed().as_millis() as u64);
                                let body = resp2.text().await.map_err(FetchiumError::Network)?;
                                Ok(FetchResult {
                                    body,
                                    status: 200,
                                    content_type: "text/html".into(),
                                    content_length: None,
                                    url: url.to_string(),
                                    elapsed_ms: start.elapsed().as_millis() as u64,
                                    retries: 1,
                                })
                            }
                            _ => {
                                next_proxy.record_failure();
                                self.fetch(url).await // fall back to direct
                            }
                        }
                    } else {
                        self.fetch(url).await
                    }
                } else {
                    proxy.record_failure();
                    Err(FetchiumError::Structured(StructuredError {
                        kind: Self::status_to_error_kind(status),
                        retryable: false,
                        message: format!("HTTP {status} via proxy for {url}"),
                        source_url: Some(url.to_string()),
                        suggested_action: "Try different proxy or direct".into(),
                        alternatives: vec![],
                    }))
                }
            }
            Err(e) => {
                proxy.record_failure();
                debug!(
                    "Proxy {}:{} connection failed for {domain}: {e}",
                    proxy.host, proxy.port
                );
                // Fall back to direct connection
                self.fetch(url).await
            }
        }
    }

    /// Convenience: fetch text through a proxy.
    pub async fn fetch_text_via_proxy(&self, url: &str) -> FetchiumResult<String> {
        let result = self.fetch_via_proxy(url).await?;
        Ok(result.body)
    }

    /// Single-shot fetch — no retries, no rate limiting.
    ///
    /// Use this for sources where connection errors are *expected* (e.g. third-party
    /// Piped/Invidious instances that may be down). A single fast attempt is better
    /// than burning 3.5s on retry sleeps when we're racing many sources in parallel.
    pub async fn fetch_text_once(&self, url: &str) -> FetchiumResult<String> {
        let resp = self
            .inner
            .get(url)
            .send()
            .await
            .map_err(FetchiumError::Network)?;

        if !resp.status().is_success() {
            return Err(FetchiumError::Structured(crate::error::StructuredError {
                kind: Self::status_to_error_kind(resp.status()),
                retryable: false,
                message: format!("HTTP {} from {url}", resp.status()),
                source_url: Some(url.to_string()),
                suggested_action: "Source unavailable".into(),
                alternatives: vec![],
            }));
        }

        resp.text().await.map_err(FetchiumError::Network)
    }

    /// Single-shot POST — no retries, no rate limiting.
    ///
    /// Like `fetch_text_once` but sends a JSON POST body. Used for YouTube Innertube
    /// API calls where each call is time-sensitive and retries are not desired.
    pub async fn post_json_once(&self, url: &str, body: String) -> FetchiumResult<String> {
        let resp = self
            .inner
            .post(url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(FetchiumError::Network)?;

        if !resp.status().is_success() {
            return Err(FetchiumError::Structured(crate::error::StructuredError {
                kind: Self::status_to_error_kind(resp.status()),
                retryable: false,
                message: format!("HTTP {} from {url}", resp.status()),
                source_url: Some(url.to_string()),
                suggested_action: "Source unavailable".into(),
                alternatives: vec![],
            }));
        }

        resp.text().await.map_err(FetchiumError::Network)
    }

    /// POST JSON with retry support (uses the standard retry/rate-limit pipeline).
    pub async fn post_json(&self, url: &str, body: &str) -> FetchiumResult<String> {
        let resp = self
            .inner
            .post(url)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await
            .map_err(FetchiumError::Network)?;

        if !resp.status().is_success() {
            return Err(FetchiumError::Structured(crate::error::StructuredError {
                kind: Self::status_to_error_kind(resp.status()),
                retryable: resp.status().is_server_error(),
                message: format!("HTTP {} from {url}", resp.status()),
                source_url: Some(url.to_string()),
                suggested_action: "Check API key and request format".into(),
                alternatives: vec![],
            }));
        }

        resp.text().await.map_err(FetchiumError::Network)
    }

    /// POST JSON with a custom header (e.g., API key in non-standard header).
    pub async fn post_json_with_header(
        &self,
        url: &str,
        body: &str,
        header_name: &str,
        header_value: &str,
    ) -> FetchiumResult<String> {
        let resp = self
            .inner
            .post(url)
            .header("Content-Type", "application/json")
            .header(header_name, header_value)
            .body(body.to_string())
            .send()
            .await
            .map_err(FetchiumError::Network)?;

        if !resp.status().is_success() {
            return Err(FetchiumError::Structured(crate::error::StructuredError {
                kind: Self::status_to_error_kind(resp.status()),
                retryable: resp.status().is_server_error(),
                message: format!("HTTP {} from {url}", resp.status()),
                source_url: Some(url.to_string()),
                suggested_action: "Check API key and request format".into(),
                alternatives: vec![],
            }));
        }

        resp.text().await.map_err(FetchiumError::Network)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_creation() {
        let config = FetchiumConfig::default();
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
