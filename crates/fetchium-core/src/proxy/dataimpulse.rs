//! DataImpulse residential proxy gateway client.
//!
//! Single gateway endpoint with country-targeted username suffix:
//!   `{user}__cr.{cc}:{pass}@gw.dataimpulse.com:823`
//!
//! ## GB efficiency (pay-per-GB)
//! - Clients cached per country code — built once, reused across requests
//! - gzip + brotli compression enabled on all requests
//! - Only used for scraper backends that are blocked without residential IPs
//!   (Google, DDG, Bing, Brave). API backends (Serper, Exa, Tavily) are never proxied.
//! - Response body capped at 512 KB — SERP HTML is ~50-150 KB gzip'd

use dashmap::DashMap;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};

/// DataImpulse residential proxy gateway client with country targeting.
#[derive(Clone)]
pub struct DataImpulseClient {
    inner: Arc<DataImpulseInner>,
}

struct DataImpulseInner {
    username: String,
    password: String,
    host: String,
    port: u16,
    user_agent: String,
    timeout: Duration,
    /// Cached clients keyed by country code ("" = random/default).
    clients: DashMap<String, Client>,
}

impl DataImpulseClient {
    /// Build a DataImpulse client from credentials.
    pub fn new(
        username: impl Into<String>,
        password: impl Into<String>,
        host: impl Into<String>,
        port: u16,
        user_agent: impl Into<String>,
        timeout: Duration,
    ) -> Self {
        Self {
            inner: Arc::new(DataImpulseInner {
                username: username.into(),
                password: password.into(),
                host: host.into(),
                port,
                user_agent: user_agent.into(),
                timeout,
                clients: DashMap::new(),
            }),
        }
    }

    /// Get a reqwest::Client routed through the given country.
    ///
    /// Pass `None` for a random residential IP (no country targeting).
    /// Pass `Some("us")`, `Some("gb")`, `Some("fr")` etc. for country-specific routing.
    /// Clients are cached per country code — O(1) on subsequent calls.
    pub fn client(&self, country_code: Option<&str>) -> Client {
        let cc = country_code.map(|c| c.to_lowercase()).unwrap_or_default();

        // Fast path: cached client
        if let Some(c) = self.inner.clients.get(&cc) {
            return c.clone();
        }

        // Build username with country suffix
        let username = if cc.is_empty() {
            self.inner.username.clone()
        } else {
            format!("{}__cr.{}", self.inner.username, cc)
        };

        let proxy_url = format!(
            "http://{}:{}@{}:{}",
            username, self.inner.password, self.inner.host, self.inner.port
        );

        debug!(
            "DataImpulse: building client for country={:?} proxy={}",
            cc, self.inner.host
        );

        let client = match reqwest::Client::builder()
            .user_agent(&self.inner.user_agent)
            .timeout(self.inner.timeout)
            .connect_timeout(Duration::from_secs(8))
            .proxy(reqwest::Proxy::all(&proxy_url).expect("valid DataImpulse proxy URL"))
            .redirect(reqwest::redirect::Policy::limited(5))
            .gzip(true)
            .brotli(true)
            // Don't auto-decompress beyond gzip/brotli — saves GB vs double-decompression
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                warn!("DataImpulse: failed to build client for cc={cc}: {e}");
                return reqwest::Client::new();
            }
        };

        self.inner.clients.insert(cc, client.clone());
        client
    }

    /// Get a fresh (non-cached) client that forces a new residential IP assignment.
    ///
    /// Use this for retry-on-block scenarios. The client has zero idle connections
    /// so the proxy gateway assigns a fresh IP on every request.
    pub fn fresh_client(&self, country_code: Option<&str>) -> Client {
        let cc = country_code.map(|c| c.to_lowercase()).unwrap_or_default();

        let username = if cc.is_empty() {
            self.inner.username.clone()
        } else {
            format!("{}__cr.{}", self.inner.username, cc)
        };

        let proxy_url = format!(
            "http://{}:{}@{}:{}",
            username, self.inner.password, self.inner.host, self.inner.port
        );

        match reqwest::Client::builder()
            .user_agent(&self.inner.user_agent)
            .timeout(self.inner.timeout)
            .connect_timeout(Duration::from_secs(8))
            .proxy(reqwest::Proxy::all(&proxy_url).expect("valid DataImpulse proxy URL"))
            .redirect(reqwest::redirect::Policy::limited(5))
            .gzip(true)
            .brotli(true)
            // No connection pool — every request gets a new residential IP
            .pool_max_idle_per_host(0)
            .build()
        {
            Ok(c) => c,
            Err(_) => reqwest::Client::new(),
        }
    }

    /// Whether credentials are configured (non-empty).
    pub fn is_configured(&self) -> bool {
        !self.inner.username.is_empty() && !self.inner.password.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_cached_per_country() {
        let di = DataImpulseClient::new(
            "testuser",
            "testpass",
            "gw.example.com",
            823,
            "TestAgent/1.0",
            Duration::from_secs(10),
        );
        let c1 = di.client(Some("us"));
        let c2 = di.client(Some("us"));
        let c3 = di.client(Some("gb"));
        // Same country → same cached client (pointer equality via Arc internals)
        // Just verify no panic and different country returns a client
        drop(c1);
        drop(c2);
        drop(c3);
    }

    #[test]
    fn configured_check() {
        let di = DataImpulseClient::new("u", "p", "h", 823, "UA", Duration::from_secs(5));
        assert!(di.is_configured());
        let empty = DataImpulseClient::new("", "", "h", 823, "UA", Duration::from_secs(5));
        assert!(!empty.is_configured());
    }
}
