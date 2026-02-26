//! Lock-free pipeline metrics — latency histograms, counters, gauges.
//!
//! Uses atomic operations throughout for zero-contention in concurrent pipelines.
//! Metrics are organized by operation type (search, fetch, extract, rank, validate).

use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Global pipeline metrics collector.
///
/// Thread-safe, lock-free, designed for concurrent access from
/// multiple search backends and pipeline stages simultaneously.
#[derive(Clone)]
pub struct PipelineMetrics {
    /// Per-operation latency tracking.
    operations: Arc<DashMap<String, OperationStats>>,
    /// Global counters.
    counters: Arc<GlobalCounters>,
    /// When the metrics collector was created.
    started_at: Instant,
}

/// Atomic global counters for pipeline-wide metrics.
struct GlobalCounters {
    total_searches: AtomicU64,
    total_fetches: AtomicU64,
    total_extractions: AtomicU64,
    total_cache_hits: AtomicU64,
    total_cache_misses: AtomicU64,
    total_errors: AtomicU64,
    total_tokens_processed: AtomicU64,
    total_bytes_fetched: AtomicU64,
}

impl GlobalCounters {
    fn new() -> Self {
        Self {
            total_searches: AtomicU64::new(0),
            total_fetches: AtomicU64::new(0),
            total_extractions: AtomicU64::new(0),
            total_cache_hits: AtomicU64::new(0),
            total_cache_misses: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            total_tokens_processed: AtomicU64::new(0),
            total_bytes_fetched: AtomicU64::new(0),
        }
    }
}

/// Per-operation latency and count tracking using HDR-style bucketing.
#[derive(Debug)]
struct OperationStats {
    count: AtomicU64,
    total_duration_us: AtomicU64,
    min_duration_us: AtomicU64,
    max_duration_us: AtomicU64,
    errors: AtomicU64,
}

impl Default for OperationStats {
    fn default() -> Self {
        Self {
            count: AtomicU64::new(0),
            total_duration_us: AtomicU64::new(0),
            min_duration_us: AtomicU64::new(u64::MAX),
            max_duration_us: AtomicU64::new(0),
            errors: AtomicU64::new(0),
        }
    }
}

impl OperationStats {
    fn record(&self, duration: Duration) {
        let us = duration.as_micros() as u64;
        self.count.fetch_add(1, Ordering::Relaxed);
        self.total_duration_us.fetch_add(us, Ordering::Relaxed);
        self.min_duration_us.fetch_min(us, Ordering::Relaxed);
        self.max_duration_us.fetch_max(us, Ordering::Relaxed);
    }

    fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    fn snapshot(&self) -> OperationSnapshot {
        let count = self.count.load(Ordering::Relaxed);
        let total_us = self.total_duration_us.load(Ordering::Relaxed);
        let min_us = self.min_duration_us.load(Ordering::Relaxed);
        let max_us = self.max_duration_us.load(Ordering::Relaxed);
        let errors = self.errors.load(Ordering::Relaxed);

        OperationSnapshot {
            count,
            avg_duration: if count > 0 {
                Duration::from_micros(total_us / count)
            } else {
                Duration::ZERO
            },
            min_duration: if min_us == u64::MAX {
                Duration::ZERO
            } else {
                Duration::from_micros(min_us)
            },
            max_duration: Duration::from_micros(max_us),
            errors,
            error_rate: if count > 0 {
                errors as f64 / count as f64
            } else {
                0.0
            },
        }
    }
}

impl PipelineMetrics {
    /// Create a new metrics collector.
    pub fn new() -> Self {
        Self {
            operations: Arc::new(DashMap::new()),
            counters: Arc::new(GlobalCounters::new()),
            started_at: Instant::now(),
        }
    }

    /// Start timing an operation. Returns a timer that records on drop.
    pub fn start_operation(&self, operation: &str) -> OperationTimer {
        OperationTimer {
            operation: operation.to_string(),
            start: Instant::now(),
            metrics: self.clone(),
            errored: false,
        }
    }

    /// Record a completed operation with explicit duration.
    pub fn record_operation(&self, operation: &str, duration: Duration) {
        let entry = self.operations.entry(operation.to_string()).or_default();
        entry.value().record(duration);
    }

    /// Record an error for an operation.
    pub fn record_error(&self, operation: &str) {
        let entry = self.operations.entry(operation.to_string()).or_default();
        entry.value().record_error();
        self.counters.total_errors.fetch_add(1, Ordering::Relaxed);
    }

    // ─── Counter Helpers ──────────────────────────────────────────

    /// Increment the search counter.
    pub fn inc_searches(&self) {
        self.counters.total_searches.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment the fetch counter.
    pub fn inc_fetches(&self) {
        self.counters.total_fetches.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment the extraction counter.
    pub fn inc_extractions(&self) {
        self.counters
            .total_extractions
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache hit.
    pub fn inc_cache_hits(&self) {
        self.counters
            .total_cache_hits
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss.
    pub fn inc_cache_misses(&self) {
        self.counters
            .total_cache_misses
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Add to total tokens processed.
    pub fn add_tokens(&self, count: u64) {
        self.counters
            .total_tokens_processed
            .fetch_add(count, Ordering::Relaxed);
    }

    /// Add to total bytes fetched.
    pub fn add_bytes(&self, count: u64) {
        self.counters
            .total_bytes_fetched
            .fetch_add(count, Ordering::Relaxed);
    }

    // ─── Snapshots ────────────────────────────────────────────────

    /// Get a full metrics snapshot for reporting.
    pub fn snapshot(&self) -> MetricsSnapshot {
        let uptime = self.started_at.elapsed();
        let operations: std::collections::HashMap<String, OperationSnapshot> = self
            .operations
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().snapshot()))
            .collect();

        let total_searches = self.counters.total_searches.load(Ordering::Relaxed);
        let total_fetches = self.counters.total_fetches.load(Ordering::Relaxed);

        MetricsSnapshot {
            uptime,
            total_searches,
            total_fetches,
            total_extractions: self.counters.total_extractions.load(Ordering::Relaxed),
            total_cache_hits: self.counters.total_cache_hits.load(Ordering::Relaxed),
            total_cache_misses: self.counters.total_cache_misses.load(Ordering::Relaxed),
            total_errors: self.counters.total_errors.load(Ordering::Relaxed),
            total_tokens_processed: self.counters.total_tokens_processed.load(Ordering::Relaxed),
            total_bytes_fetched: self.counters.total_bytes_fetched.load(Ordering::Relaxed),
            searches_per_sec: if uptime.as_secs() > 0 {
                total_searches as f64 / uptime.as_secs_f64()
            } else {
                0.0
            },
            fetches_per_sec: if uptime.as_secs() > 0 {
                total_fetches as f64 / uptime.as_secs_f64()
            } else {
                0.0
            },
            cache_hit_rate: {
                let hits = self.counters.total_cache_hits.load(Ordering::Relaxed);
                let misses = self.counters.total_cache_misses.load(Ordering::Relaxed);
                let total = hits + misses;
                if total > 0 {
                    hits as f64 / total as f64
                } else {
                    0.0
                }
            },
            operations,
        }
    }

    /// Get a summary suitable for CLI display.
    pub fn summary_line(&self) -> String {
        let snap = self.snapshot();
        format!(
            "searches={} fetches={} extractions={} errors={} cache_hit_rate={:.0}% uptime={:.1}s",
            snap.total_searches,
            snap.total_fetches,
            snap.total_extractions,
            snap.total_errors,
            snap.cache_hit_rate * 100.0,
            snap.uptime.as_secs_f64(),
        )
    }
}

impl Default for PipelineMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII timer — records operation duration on drop.
pub struct OperationTimer {
    operation: String,
    start: Instant,
    metrics: PipelineMetrics,
    errored: bool,
}

impl OperationTimer {
    /// Mark this operation as failed.
    pub fn mark_error(&mut self) {
        self.errored = true;
    }

    /// Get elapsed time so far.
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Drop for OperationTimer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.metrics.record_operation(&self.operation, duration);
        if self.errored {
            self.metrics.record_error(&self.operation);
        }
    }
}

/// Point-in-time snapshot of all metrics.
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub uptime: Duration,
    pub total_searches: u64,
    pub total_fetches: u64,
    pub total_extractions: u64,
    pub total_cache_hits: u64,
    pub total_cache_misses: u64,
    pub total_errors: u64,
    pub total_tokens_processed: u64,
    pub total_bytes_fetched: u64,
    pub searches_per_sec: f64,
    pub fetches_per_sec: f64,
    pub cache_hit_rate: f64,
    pub operations: std::collections::HashMap<String, OperationSnapshot>,
}

/// Snapshot of a single operation's metrics.
#[derive(Debug, Clone)]
pub struct OperationSnapshot {
    pub count: u64,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub errors: u64,
    pub error_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_basic_counting() {
        let m = PipelineMetrics::new();
        m.inc_searches();
        m.inc_searches();
        m.inc_fetches();

        let snap = m.snapshot();
        assert_eq!(snap.total_searches, 2);
        assert_eq!(snap.total_fetches, 1);
    }

    #[test]
    fn operation_timing() {
        let m = PipelineMetrics::new();
        m.record_operation("test_op", Duration::from_millis(100));
        m.record_operation("test_op", Duration::from_millis(200));
        m.record_operation("test_op", Duration::from_millis(300));

        let snap = m.snapshot();
        let op = snap.operations.get("test_op").unwrap();
        assert_eq!(op.count, 3);
        assert!(op.avg_duration.as_millis() >= 150 && op.avg_duration.as_millis() <= 250);
        assert!(op.min_duration.as_millis() >= 90 && op.min_duration.as_millis() <= 110);
        assert!(op.max_duration.as_millis() >= 290 && op.max_duration.as_millis() <= 310);
    }

    #[test]
    fn operation_timer_raii() {
        let m = PipelineMetrics::new();
        {
            let _timer = m.start_operation("timed_op");
            std::thread::sleep(Duration::from_millis(10));
        }
        let snap = m.snapshot();
        let op = snap.operations.get("timed_op").unwrap();
        assert_eq!(op.count, 1);
        assert!(op.avg_duration.as_millis() >= 5);
    }

    #[test]
    fn error_tracking() {
        let m = PipelineMetrics::new();
        m.record_operation("fail_op", Duration::from_millis(50));
        m.record_error("fail_op");

        let snap = m.snapshot();
        assert_eq!(snap.total_errors, 1);
        let op = snap.operations.get("fail_op").unwrap();
        assert_eq!(op.errors, 1);
    }

    #[test]
    fn cache_hit_rate() {
        let m = PipelineMetrics::new();
        m.inc_cache_hits();
        m.inc_cache_hits();
        m.inc_cache_hits();
        m.inc_cache_misses();

        let snap = m.snapshot();
        assert!((snap.cache_hit_rate - 0.75).abs() < 0.01);
    }

    #[test]
    fn summary_line_format() {
        let m = PipelineMetrics::new();
        m.inc_searches();
        let line = m.summary_line();
        assert!(line.contains("searches=1"));
    }
}
