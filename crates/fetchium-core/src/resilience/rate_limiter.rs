//! Adaptive per-domain rate limiter using AIMD (Additive Increase, Multiplicative Decrease).
//!
//! Instead of fixed delays, this system learns the optimal request rate per domain:
//! - On success: additively increase allowed rate (reduce delay)
//! - On 429/5xx: multiplicatively decrease rate (increase delay)
//!
//! This converges to the maximum sustainable throughput per domain automatically.
//! Inspired by TCP congestion control (RFC 5681).

use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tracing::debug;

/// Minimum delay between requests to any domain (politeness floor).
const MIN_DELAY_MS: u64 = 100;
/// Maximum delay between requests (prevents starvation).
const MAX_DELAY_MS: u64 = 30_000;
/// Default starting delay for unknown domains.
const DEFAULT_DELAY_MS: u64 = 200;
/// Additive increase step on success (milliseconds).
const ADDITIVE_INCREASE_MS: u64 = 20;
/// Multiplicative decrease factor on rate limit/error (delay × this).
const MULTIPLICATIVE_DECREASE: f64 = 2.0;

/// Per-domain rate state.
#[derive(Debug, Clone)]
struct DomainRate {
    /// Current delay between requests (adaptive).
    current_delay_ms: u64,
    /// When the last request was sent.
    last_request: Instant,
    /// Total successful requests.
    successes: u64,
    /// Total rate-limited responses (429s).
    rate_limits: u64,
    /// Total errors (5xx, timeouts).
    errors: u64,
}

impl DomainRate {
    fn new() -> Self {
        Self {
            current_delay_ms: DEFAULT_DELAY_MS,
            last_request: Instant::now() - Duration::from_secs(60),
            successes: 0,
            rate_limits: 0,
            errors: 0,
        }
    }

    fn effective_delay(&self) -> Duration {
        Duration::from_millis(self.current_delay_ms)
    }

    fn time_until_allowed(&self) -> Duration {
        let elapsed = self.last_request.elapsed();
        let delay = self.effective_delay();
        if elapsed >= delay {
            Duration::ZERO
        } else {
            delay - elapsed
        }
    }

    fn mark_request_sent(&mut self) {
        self.last_request = Instant::now();
    }

    fn record_success(&mut self) {
        self.successes += 1;
        // Additive increase — slowly reduce delay toward MIN
        self.current_delay_ms = self
            .current_delay_ms
            .saturating_sub(ADDITIVE_INCREASE_MS)
            .max(MIN_DELAY_MS);
    }

    fn record_rate_limit(&mut self) {
        self.rate_limits += 1;
        // Multiplicative decrease — aggressively back off
        self.current_delay_ms =
            ((self.current_delay_ms as f64 * MULTIPLICATIVE_DECREASE) as u64).min(MAX_DELAY_MS);
    }

    fn record_error(&mut self) {
        self.errors += 1;
        // Lighter backoff for server errors (not as aggressive as rate limits)
        self.current_delay_ms = ((self.current_delay_ms as f64 * 1.5) as u64).min(MAX_DELAY_MS);
    }
}

/// Thread-safe adaptive rate limiter for all domains.
///
/// Automatically learns the maximum sustainable request rate per domain
/// using AIMD (Additive Increase, Multiplicative Decrease) control.
#[derive(Clone)]
pub struct AdaptiveRateLimiter {
    domains: Arc<DashMap<String, DomainRate>>,
    /// Global concurrency limit to prevent resource exhaustion.
    global_semaphore: Arc<Semaphore>,
}

impl AdaptiveRateLimiter {
    /// Create a new rate limiter with the given global concurrency limit.
    pub fn new(max_global_concurrent: usize) -> Self {
        Self {
            domains: Arc::new(DashMap::new()),
            global_semaphore: Arc::new(Semaphore::new(max_global_concurrent)),
        }
    }

    /// Wait until a request to the given domain is allowed, respecting rate limits.
    /// Acquires a global concurrency permit + per-domain delay.
    pub async fn acquire(&self, domain: &str) -> RateLimitPermit {
        // Acquire global concurrency slot
        let permit = self
            .global_semaphore
            .clone()
            .acquire_owned()
            .await
            .expect("semaphore closed");

        // Check per-domain delay
        let wait_time = {
            let entry = self
                .domains
                .entry(domain.to_string())
                .or_insert_with(DomainRate::new);
            let rate = entry.value();
            rate.time_until_allowed()
        };

        if !wait_time.is_zero() {
            debug!("Rate limiter: waiting {:?} for domain {domain}", wait_time);
            tokio::time::sleep(wait_time).await;
        }

        // Mark request as sent
        if let Some(mut entry) = self.domains.get_mut(domain) {
            entry.mark_request_sent();
        }

        RateLimitPermit {
            _permit: permit,
            domain: domain.to_string(),
            limiter: self.clone(),
        }
    }

    /// Record a successful response from a domain.
    pub fn record_success(&self, domain: &str) {
        if let Some(mut entry) = self.domains.get_mut(domain) {
            entry.record_success();
            debug!(
                "Rate limiter {domain}: success (delay now {}ms)",
                entry.current_delay_ms
            );
        }
    }

    /// Record a 429 Too Many Requests response from a domain.
    pub fn record_rate_limit(&self, domain: &str) {
        let entry = self
            .domains
            .entry(domain.to_string())
            .or_insert_with(DomainRate::new);
        let mut rate = entry;
        rate.record_rate_limit();
        debug!(
            "Rate limiter {domain}: rate limited! (delay now {}ms)",
            rate.current_delay_ms
        );
    }

    /// Record a server error (5xx, timeout) from a domain.
    pub fn record_error(&self, domain: &str) {
        let entry = self
            .domains
            .entry(domain.to_string())
            .or_insert_with(DomainRate::new);
        let mut rate = entry;
        rate.record_error();
        debug!(
            "Rate limiter {domain}: error (delay now {}ms)",
            rate.current_delay_ms
        );
    }

    /// Get rate statistics for a domain.
    pub fn domain_stats(&self, domain: &str) -> Option<DomainStats> {
        self.domains.get(domain).map(|entry| DomainStats {
            domain: domain.to_string(),
            current_delay_ms: entry.current_delay_ms,
            successes: entry.successes,
            rate_limits: entry.rate_limits,
            errors: entry.errors,
        })
    }

    /// Get all domain statistics.
    pub fn all_stats(&self) -> Vec<DomainStats> {
        self.domains
            .iter()
            .map(|entry| DomainStats {
                domain: entry.key().clone(),
                current_delay_ms: entry.value().current_delay_ms,
                successes: entry.value().successes,
                rate_limits: entry.value().rate_limits,
                errors: entry.value().errors,
            })
            .collect()
    }

    /// Available global concurrency slots.
    pub fn available_permits(&self) -> usize {
        self.global_semaphore.available_permits()
    }
}

impl Default for AdaptiveRateLimiter {
    fn default() -> Self {
        Self::new(50) // Default: 50 concurrent requests globally
    }
}

/// RAII permit that releases the concurrency slot when dropped.
pub struct RateLimitPermit {
    _permit: tokio::sync::OwnedSemaphorePermit,
    domain: String,
    limiter: AdaptiveRateLimiter,
}

impl RateLimitPermit {
    /// Report the request outcome to update rate limits.
    pub fn report_success(self) {
        self.limiter.record_success(&self.domain);
    }

    /// Report that the request was rate-limited (429).
    pub fn report_rate_limited(self) {
        self.limiter.record_rate_limit(&self.domain);
    }

    /// Report a server error.
    pub fn report_error(self) {
        self.limiter.record_error(&self.domain);
    }
}

/// Statistics for a single domain's rate limiting.
#[derive(Debug, Clone)]
pub struct DomainStats {
    pub domain: String,
    pub current_delay_ms: u64,
    pub successes: u64,
    pub rate_limits: u64,
    pub errors: u64,
}

impl DomainStats {
    /// Error rate as a percentage (0.0–1.0).
    pub fn error_rate(&self) -> f64 {
        let total = self.successes + self.rate_limits + self.errors;
        if total == 0 {
            0.0
        } else {
            (self.rate_limits + self.errors) as f64 / total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rate_limiter_allows_requests() {
        let rl = AdaptiveRateLimiter::new(10);
        let permit = rl.acquire("example.com").await;
        permit.report_success();
    }

    #[tokio::test]
    async fn rate_limit_increases_delay() {
        let rl = AdaptiveRateLimiter::new(10);
        let permit = rl.acquire("slow.com").await;
        permit.report_rate_limited();

        let stats = rl.domain_stats("slow.com").unwrap();
        assert!(stats.current_delay_ms > DEFAULT_DELAY_MS);
        assert_eq!(stats.rate_limits, 1);
    }

    #[tokio::test]
    async fn success_decreases_delay() {
        let rl = AdaptiveRateLimiter::new(10);

        // First increase delay
        let permit = rl.acquire("learn.com").await;
        permit.report_rate_limited();
        let high_delay = rl.domain_stats("learn.com").unwrap().current_delay_ms;

        // Then succeed to decrease
        let permit = rl.acquire("learn.com").await;
        permit.report_success();
        let low_delay = rl.domain_stats("learn.com").unwrap().current_delay_ms;

        assert!(low_delay < high_delay);
    }

    #[tokio::test]
    async fn global_concurrency_limit() {
        let rl = AdaptiveRateLimiter::new(2);
        assert_eq!(rl.available_permits(), 2);

        let _p1 = rl.acquire("a.com").await;
        assert_eq!(rl.available_permits(), 1);

        let _p2 = rl.acquire("b.com").await;
        assert_eq!(rl.available_permits(), 0);

        // Third request would block, so we don't test that here
    }

    #[test]
    fn domain_stats_error_rate() {
        let stats = DomainStats {
            domain: "test.com".into(),
            current_delay_ms: 200,
            successes: 8,
            rate_limits: 1,
            errors: 1,
        };
        assert!((stats.error_rate() - 0.2).abs() < 0.01);
    }
}
