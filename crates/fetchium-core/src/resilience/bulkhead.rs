//! Bulkhead isolation pattern — prevents one slow backend from starving others.
//!
//! Each backend gets its own concurrency pool. If backend A is slow and consuming
//! all its permits, backend B still has its own pool and runs unaffected.
//! This is critical for search orchestrator reliability.

use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::warn;

/// Configuration for bulkhead isolation.
#[derive(Debug, Clone)]
pub struct BulkheadConfig {
    /// Maximum concurrent requests per backend.
    pub max_concurrent_per_backend: usize,
    /// How long to wait for a permit before giving up.
    pub acquire_timeout: Duration,
}

impl Default for BulkheadConfig {
    fn default() -> Self {
        Self {
            max_concurrent_per_backend: 5,
            acquire_timeout: Duration::from_secs(10),
        }
    }
}

/// Bulkhead isolation — per-backend concurrency pools.
#[derive(Clone)]
pub struct Bulkhead {
    pools: Arc<DashMap<String, Arc<Semaphore>>>,
    config: BulkheadConfig,
}

impl Bulkhead {
    /// Create a new bulkhead with default config.
    pub fn new() -> Self {
        Self::with_config(BulkheadConfig::default())
    }

    /// Create with custom config.
    pub fn with_config(config: BulkheadConfig) -> Self {
        Self {
            pools: Arc::new(DashMap::new()),
            config,
        }
    }

    /// Try to acquire a concurrency permit for a backend.
    /// Returns None if the backend's pool is exhausted (timeout).
    pub async fn try_acquire(&self, backend_id: &str) -> Option<BulkheadPermit> {
        let sem = self.get_or_create_pool(backend_id);

        match tokio::time::timeout(self.config.acquire_timeout, sem.clone().acquire_owned()).await {
            Ok(Ok(permit)) => Some(BulkheadPermit {
                _permit: permit,
                backend_id: backend_id.to_string(),
            }),
            Ok(Err(_)) => {
                warn!("Bulkhead: semaphore closed for {backend_id}");
                None
            }
            Err(_) => {
                warn!(
                    "Bulkhead: timeout acquiring permit for {backend_id} (all {} slots busy)",
                    self.config.max_concurrent_per_backend
                );
                None
            }
        }
    }

    fn get_or_create_pool(&self, backend_id: &str) -> Arc<Semaphore> {
        self.pools
            .entry(backend_id.to_string())
            .or_insert_with(|| Arc::new(Semaphore::new(self.config.max_concurrent_per_backend)))
            .value()
            .clone()
    }

    /// Available permits for a backend.
    pub fn available(&self, backend_id: &str) -> usize {
        self.pools
            .get(backend_id)
            .map(|s| s.available_permits())
            .unwrap_or(self.config.max_concurrent_per_backend)
    }

    /// Get utilization stats for all backends.
    pub fn utilization(&self) -> Vec<BulkheadStats> {
        let max = self.config.max_concurrent_per_backend;
        self.pools
            .iter()
            .map(|entry| {
                let available = entry.value().available_permits();
                BulkheadStats {
                    backend_id: entry.key().clone(),
                    max_concurrent: max,
                    in_use: max - available,
                    available,
                }
            })
            .collect()
    }
}

impl Default for Bulkhead {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII permit — concurrency slot is released when this is dropped.
pub struct BulkheadPermit {
    _permit: tokio::sync::OwnedSemaphorePermit,
    #[allow(dead_code)]
    backend_id: String,
}

/// Utilization stats for a single backend's bulkhead.
#[derive(Debug, Clone)]
pub struct BulkheadStats {
    pub backend_id: String,
    pub max_concurrent: usize,
    pub in_use: usize,
    pub available: usize,
}

impl BulkheadStats {
    /// Utilization as a fraction (0.0–1.0).
    pub fn utilization(&self) -> f64 {
        if self.max_concurrent == 0 {
            return 0.0;
        }
        self.in_use as f64 / self.max_concurrent as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn bulkhead_allows_requests() {
        let bh = Bulkhead::new();
        let permit = bh.try_acquire("backend-a").await;
        assert!(permit.is_some());
    }

    #[tokio::test]
    async fn bulkhead_isolates_backends() {
        let config = BulkheadConfig {
            max_concurrent_per_backend: 1,
            acquire_timeout: Duration::from_millis(100),
        };
        let bh = Bulkhead::with_config(config);

        // Backend A takes its only slot
        let _p1 = bh.try_acquire("backend-a").await.unwrap();

        // Backend B should still have its own pool available
        let p2 = bh.try_acquire("backend-b").await;
        assert!(p2.is_some());

        // Backend A should be full
        let p3 = bh.try_acquire("backend-a").await;
        assert!(p3.is_none());
    }

    #[tokio::test]
    async fn bulkhead_releases_on_drop() {
        let config = BulkheadConfig {
            max_concurrent_per_backend: 1,
            acquire_timeout: Duration::from_millis(100),
        };
        let bh = Bulkhead::with_config(config);

        {
            let _p = bh.try_acquire("backend-x").await.unwrap();
            assert_eq!(bh.available("backend-x"), 0);
        }
        // Permit dropped — slot should be free
        assert_eq!(bh.available("backend-x"), 1);
    }

    #[test]
    fn utilization_stats() {
        let stats = BulkheadStats {
            backend_id: "test".into(),
            max_concurrent: 5,
            in_use: 3,
            available: 2,
        };
        assert!((stats.utilization() - 0.6).abs() < 0.01);
    }
}
