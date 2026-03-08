use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};
use parking_lot::Mutex;

const FAILURE_THRESHOLD: u32 = 5;
const COOLDOWN_PERIOD: Duration = Duration::from_secs(60);

/// Circuit breaker to prevent cascading failures and manage backend health.
pub struct CircuitBreaker {
    failures: AtomicU32,
    last_failure: Mutex<Option<Instant>>,
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self {
            failures: AtomicU32::new(0),
            last_failure: Mutex::new(None),
        }
    }

    /// Returns true if the circuit is open (unhealthy).
    pub fn is_open(&self) -> bool {
        let count = self.failures.load(Ordering::Relaxed);
        if count >= FAILURE_THRESHOLD {
            let last = self.last_failure.lock();
            if let Some(t) = *last {
                if t.elapsed() < COOLDOWN_PERIOD {
                    return true;
                }
            }
        }
        false
    }

    /// Reset failure count on success.
    pub fn report_success(&self) {
        self.failures.store(0, Ordering::Relaxed);
    }

    /// Increment failure count and update timestamp.
    pub fn report_failure(&self) {
        self.failures.fetch_add(1, Ordering::Relaxed);
        *self.last_failure.lock() = Some(Instant::now());
    }
}
