//! REST API middleware — rate limiting, CORS, request logging.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use hsx_core::config::HsxConfig;
use hsx_core::http::client::HttpClient;
use hsx_core::cache::MemoryCache;

/// Shared application state injected into all axum handlers.
#[derive(Clone)]
pub struct AppState {
    pub config: HsxConfig,
    pub http: HttpClient,
    pub cache: MemoryCache,
    pub rate_limiter: Arc<RateLimiter>,
}

/// Default rate-limit: 100 requests per 60-second window per IP.
const DEFAULT_RATE_LIMIT_REQUESTS: u32 = 100;
const DEFAULT_RATE_LIMIT_WINDOW_SECS: u64 = 60;

impl AppState {
    pub fn new(config: HsxConfig) -> anyhow::Result<Self> {
        let http = HttpClient::new(&config)?;
        let cache = MemoryCache::new(
            config.cache.memory_max_entries,
            config.cache.ttl_secs,
            config.cache.enabled,
        );
        Ok(Self {
            config,
            http,
            cache,
            rate_limiter: Arc::new(RateLimiter::new(
                DEFAULT_RATE_LIMIT_REQUESTS,
                DEFAULT_RATE_LIMIT_WINDOW_SECS,
            )),
        })
    }
}

/// Simple token-bucket rate limiter keyed by IP.
pub struct RateLimiter {
    /// IP → (request_count, window_start)
    buckets: Mutex<HashMap<String, (u32, Instant)>>,
    max_requests: u32,
    window_secs: u64,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            buckets: Mutex::new(HashMap::new()),
            max_requests,
            window_secs,
        }
    }

    /// Check if `ip` is within the rate limit. Returns `true` if allowed.
    pub async fn check(&self, ip: &str) -> bool {
        let mut buckets = self.buckets.lock().await;
        let now = Instant::now();
        let entry = buckets.entry(ip.to_string()).or_insert((0, now));

        // Reset window if expired
        if now.duration_since(entry.1).as_secs() >= self.window_secs {
            *entry = (0, now);
        }

        if entry.0 >= self.max_requests {
            return false;
        }

        entry.0 += 1;
        true
    }
}
