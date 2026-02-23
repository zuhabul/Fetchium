//! Cache system — Memory LRU (moka) + SQLite disk (PRD §28).
//!
//! Phase 1: In-memory LRU cache using the `moka` crate.
//! Phase 2+: Disk-backed SQLite cache for cross-session persistence.
//!
//! Cache key convention:
//!   - Fetch:  "fetch:{url}"
//!   - Search: "search:{query}:{backend}:{max_results}"
//!   - QATBE:  "qatbe:{url}:{query}:{budget}"

use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;

/// Default TTL for cached entries (1 hour).
const _DEFAULT_TTL_SECS: u64 = 3600;
/// Default maximum number of entries in the memory cache.
const _DEFAULT_MAX_ENTRIES: u64 = 1000;
/// Maximum size of a single cached value (in bytes of JSON).
const MAX_ENTRY_BYTES: usize = 10 * 1024 * 1024; // 10 MB

/// A cached value with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedEntry {
    /// The serialized JSON value.
    pub value: serde_json::Value,
    /// When this entry was created (Unix timestamp seconds).
    pub created_at: u64,
    /// Cache key for reference.
    pub key: String,
}

/// In-memory LRU cache for fetch and search results.
///
/// Thread-safe and async-native via `moka`. Each entry has a TTL
/// and the total number of entries is bounded.
#[derive(Clone)]
pub struct MemoryCache {
    inner: Arc<Cache<String, CachedEntry>>,
    enabled: bool,
    ttl: Duration,
}

impl MemoryCache {
    /// Create a new memory cache with the given configuration.
    pub fn new(max_entries: u64, ttl_secs: u64, enabled: bool) -> Self {
        let ttl = Duration::from_secs(ttl_secs);
        let inner = Cache::builder()
            .max_capacity(max_entries)
            .time_to_live(ttl)
            .time_to_idle(Duration::from_secs(ttl_secs / 2))
            .build();

        Self {
            inner: Arc::new(inner),
            enabled,
            ttl,
        }
    }

    /// Create a cache with default settings from config.
    pub fn from_config(config: &crate::config::CacheConfig) -> Self {
        Self::new(config.memory_max_entries, config.ttl_secs, config.enabled)
    }

    /// Get a cached value by key, if it exists and has not expired.
    pub async fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        if !self.enabled {
            return None;
        }
        let entry = self.inner.get(key).await?;
        debug!("Cache HIT: {key}");
        serde_json::from_value(entry.value).ok()
    }

    /// Store a value in the cache under the given key.
    ///
    /// If the serialized value exceeds MAX_ENTRY_BYTES, the entry is silently
    /// skipped to prevent memory bloat from large pages.
    pub async fn set<T: Serialize>(&self, key: &str, value: &T) {
        if !self.enabled {
            return;
        }
        let json = match serde_json::to_value(value) {
            Ok(v) => v,
            Err(e) => {
                debug!("Cache: failed to serialize for key {key}: {e}");
                return;
            }
        };

        // Check size before caching
        let size = json.to_string().len();
        if size > MAX_ENTRY_BYTES {
            debug!("Cache: skipping oversized entry for {key} ({size} bytes)");
            return;
        }

        let entry = CachedEntry {
            value: json,
            created_at: unix_now(),
            key: key.to_string(),
        };

        self.inner.insert(key.to_string(), entry).await;
        debug!("Cache SET: {key} ({size} bytes)");
    }

    /// Remove a specific entry from the cache.
    pub async fn invalidate(&self, key: &str) {
        self.inner.invalidate(key).await;
        debug!("Cache INVALIDATE: {key}");
    }

    /// Clear all entries from the cache.
    pub async fn clear(&self) {
        self.inner.invalidate_all();
        debug!("Cache CLEAR: all entries removed");
    }

    /// Return cache statistics.
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entry_count: self.inner.entry_count(),
            weighted_size: self.inner.weighted_size(),
            ttl_secs: self.ttl.as_secs(),
            enabled: self.enabled,
        }
    }

    /// Whether the cache is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl std::fmt::Debug for MemoryCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryCache")
            .field("enabled", &self.enabled)
            .field("entry_count", &self.inner.entry_count())
            .finish()
    }
}

/// Statistics about the current cache state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub entry_count: u64,
    pub weighted_size: u64,
    pub ttl_secs: u64,
    pub enabled: bool,
}

/// Build a cache key for a fetch operation.
pub fn fetch_key(url: &str) -> String {
    format!("fetch:{url}")
}

/// Build a cache key for a search operation.
pub fn search_key(query: &str, backend: &str, max_results: u32) -> String {
    format!("search:{query}:{backend}:{max_results}")
}

/// Build a cache key for a QATBE operation.
pub fn qatbe_key(url: &str, query: &str, budget: u32) -> String {
    format!("qatbe:{url}:{query}:{budget}")
}

/// Get current Unix timestamp in seconds.
fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cache_set_and_get() {
        let cache = MemoryCache::new(100, 3600, true);
        let key = "test:key";
        let value = serde_json::json!({"hello": "world", "count": 42});

        cache.set(key, &value).await;
        let retrieved: Option<serde_json::Value> = cache.get(key).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap()["count"], 42);
    }

    #[tokio::test]
    async fn disabled_cache_returns_none() {
        let cache = MemoryCache::new(100, 3600, false);
        cache.set("key", &"value").await;
        let result: Option<String> = cache.get("key").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn cache_miss_returns_none() {
        let cache = MemoryCache::new(100, 3600, true);
        let result: Option<String> = cache.get("nonexistent:key").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn cache_invalidate() {
        let cache = MemoryCache::new(100, 3600, true);
        cache.set("mykey", &42u32).await;
        cache.invalidate("mykey").await;
        let result: Option<u32> = cache.get("mykey").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn cache_stats() {
        let cache = MemoryCache::new(100, 3600, true);
        cache.set("k1", &"v1").await;
        cache.set("k2", &"v2").await;
        let stats = cache.stats();
        assert!(stats.enabled);
        assert_eq!(stats.ttl_secs, 3600);
    }

    #[test]
    fn cache_key_builders() {
        assert_eq!(fetch_key("https://example.com"), "fetch:https://example.com");
        assert_eq!(search_key("rust", "ddg", 10), "search:rust:ddg:10");
        assert_eq!(qatbe_key("https://example.com", "rust", 4000), "qatbe:https://example.com:rust:4000");
    }
}
