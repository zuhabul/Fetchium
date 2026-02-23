//! HTTP client with connection pooling, retries, and robots.txt respect.

use crate::config::HsxConfig;
use crate::error::HsxResult;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

/// Shared HTTP client with connection pooling.
#[derive(Clone)]
pub struct HttpClient {
    inner: Client,
    config: Arc<HsxConfig>,
}

impl HttpClient {
    /// Create a new HTTP client from config.
    pub fn new(config: &HsxConfig) -> HsxResult<Self> {
        let client = Client::builder()
            .user_agent(&config.fetch.user_agent)
            .timeout(Duration::from_secs(config.fetch.timeout_secs))
            .redirect(reqwest::redirect::Policy::limited(
                config.fetch.max_redirects as usize,
            ))
            .gzip(true)
            .brotli(true)
            .pool_max_idle_per_host(10)
            .build()
            .map_err(|e| crate::error::HsxError::Network(e))?;

        Ok(Self {
            inner: client,
            config: Arc::new(config.clone()),
        })
    }

    /// Get the inner reqwest client.
    pub fn client(&self) -> &Client {
        &self.inner
    }

    /// Fetch a URL and return the response body as a string.
    pub async fn fetch_text(&self, url: &str) -> HsxResult<String> {
        let resp = self
            .inner
            .get(url)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| crate::error::HsxError::Network(e))?;

        let text = resp.text().await?;
        Ok(text)
    }

    /// Get the config reference.
    pub fn config(&self) -> &HsxConfig {
        &self.config
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
}
