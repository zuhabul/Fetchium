//! Circuit breaker pattern — prevents cascading failures from broken backends.
//!
//! State machine: Closed → Open → HalfOpen → Closed/Open
//!
//! - **Closed**: Requests flow normally. Failures are counted.
//! - **Open**: All requests are immediately rejected (fail-fast). Waits for recovery timeout.
//! - **HalfOpen**: A single probe request is allowed. Success → Closed, Failure → Open.
//!
//! Configurable thresholds per backend with exponential recovery backoff.

use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation — requests flow through.
    Closed,
    /// Backend is failing — requests are rejected immediately.
    Open,
    /// Recovery probe — one request allowed to test if backend recovered.
    HalfOpen,
}

impl std::fmt::Display for CircuitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Closed => write!(f, "closed"),
            Self::Open => write!(f, "open"),
            Self::HalfOpen => write!(f, "half_open"),
        }
    }
}

/// Configuration for a circuit breaker.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening the circuit.
    pub failure_threshold: u32,
    /// How long to wait before transitioning from Open → HalfOpen.
    pub recovery_timeout: Duration,
    /// Maximum recovery timeout (exponential backoff cap).
    pub max_recovery_timeout: Duration,
    /// Number of successes in HalfOpen needed to close the circuit.
    pub success_threshold: u32,
    /// Sliding window for failure counting (older failures expire).
    pub failure_window: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            max_recovery_timeout: Duration::from_secs(300),
            success_threshold: 2,
            failure_window: Duration::from_secs(60),
        }
    }
}

/// Per-backend circuit state.
struct BackendCircuit {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure: Option<Instant>,
    last_state_change: Instant,
    current_recovery_timeout: Duration,
    config: CircuitBreakerConfig,
    /// Total requests tracked for metrics.
    total_requests: u64,
    /// Total rejections tracked for metrics.
    total_rejections: u64,
}

impl BackendCircuit {
    fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure: None,
            last_state_change: Instant::now(),
            current_recovery_timeout: config.recovery_timeout,
            config,
            total_requests: 0,
            total_rejections: 0,
        }
    }

    fn should_allow_request(&mut self) -> bool {
        self.total_requests += 1;
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if self.last_state_change.elapsed() >= self.current_recovery_timeout {
                    self.transition(CircuitState::HalfOpen);
                    true // Allow probe request
                } else {
                    self.total_rejections += 1;
                    false
                }
            }
            CircuitState::HalfOpen => {
                // Only allow limited probes
                self.success_count == 0
            }
        }
    }

    fn record_success(&mut self) {
        match self.state {
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.success_threshold {
                    self.transition(CircuitState::Closed);
                    // Reset recovery timeout on successful recovery
                    self.current_recovery_timeout = self.config.recovery_timeout;
                }
            }
            CircuitState::Closed => {
                // Decay failure count on success
                self.failure_count = self.failure_count.saturating_sub(1);
            }
            CircuitState::Open => {}
        }
    }

    fn record_failure(&mut self) {
        match self.state {
            CircuitState::Closed => {
                // Check the PREVIOUS failure's age before overwriting last_failure.
                // If we set last_failure = now() first, elapsed() would always be ~0
                // and the window would never expire.
                let window_expired = self
                    .last_failure
                    .map_or(true, |prev| prev.elapsed() > self.config.failure_window);

                if window_expired {
                    self.failure_count = 1; // Reset window
                } else {
                    self.failure_count += 1;
                }
                self.last_failure = Some(Instant::now());

                if self.failure_count >= self.config.failure_threshold {
                    self.transition(CircuitState::Open);
                }
            }
            CircuitState::HalfOpen => {
                // Probe failed — back to open with increased timeout
                self.last_failure = Some(Instant::now());
                self.current_recovery_timeout = self
                    .current_recovery_timeout
                    .mul_f64(1.5)
                    .min(self.config.max_recovery_timeout);
                self.transition(CircuitState::Open);
            }
            CircuitState::Open => {
                self.last_failure = Some(Instant::now());
            }
        }
    }

    fn transition(&mut self, new_state: CircuitState) {
        let old_state = self.state;
        self.state = new_state;
        self.last_state_change = Instant::now();
        match new_state {
            CircuitState::Closed => {
                self.failure_count = 0;
                self.success_count = 0;
            }
            CircuitState::HalfOpen => {
                self.success_count = 0;
            }
            CircuitState::Open => {
                self.success_count = 0;
            }
        }
        debug!("Circuit transition: {old_state} → {new_state}");
    }
}

/// Thread-safe circuit breaker registry for all backends.
///
/// Each backend gets its own circuit with independent state tracking.
/// The circuit breaker prevents cascading failures by fast-failing
/// requests to backends that are known to be down.
#[derive(Clone)]
pub struct CircuitBreaker {
    circuits: Arc<DashMap<String, parking_lot::Mutex<BackendCircuit>>>,
    default_config: CircuitBreakerConfig,
    /// Global rejection counter for observability.
    global_rejections: Arc<AtomicU64>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker registry with default config.
    pub fn new() -> Self {
        Self::with_config(CircuitBreakerConfig::default())
    }

    /// Create with custom default config.
    pub fn with_config(config: CircuitBreakerConfig) -> Self {
        Self {
            circuits: Arc::new(DashMap::new()),
            default_config: config,
            global_rejections: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Check if a request to the given backend should be allowed.
    /// Returns false if the circuit is open (backend is failing).
    pub fn should_allow(&self, backend_id: &str) -> bool {
        let entry = self
            .circuits
            .entry(backend_id.to_string())
            .or_insert_with(|| {
                parking_lot::Mutex::new(BackendCircuit::new(self.default_config.clone()))
            });
        let mut circuit = entry.value().lock();
        let allowed = circuit.should_allow_request();
        if !allowed {
            self.global_rejections.fetch_add(1, Ordering::Relaxed);
            warn!(
                "Circuit OPEN for {backend_id} — rejecting request (failures: {}, timeout: {:?})",
                circuit.failure_count, circuit.current_recovery_timeout
            );
        }
        allowed
    }

    /// Record a successful request to a backend.
    pub fn record_success(&self, backend_id: &str) {
        if let Some(entry) = self.circuits.get(backend_id) {
            let mut circuit = entry.value().lock();
            let old_state = circuit.state;
            circuit.record_success();
            if old_state != circuit.state {
                info!(
                    "Circuit breaker {backend_id}: {old_state} → {} (recovered)",
                    circuit.state
                );
            }
        }
    }

    /// Record a failed request to a backend.
    pub fn record_failure(&self, backend_id: &str) {
        let entry = self
            .circuits
            .entry(backend_id.to_string())
            .or_insert_with(|| {
                parking_lot::Mutex::new(BackendCircuit::new(self.default_config.clone()))
            });
        let mut circuit = entry.value().lock();
        let old_state = circuit.state;
        circuit.record_failure();
        if old_state != circuit.state {
            warn!(
                "Circuit breaker {backend_id}: {old_state} → {} (failures: {})",
                circuit.state, circuit.failure_count
            );
        }
    }

    /// Get the current state of a backend's circuit.
    pub fn state(&self, backend_id: &str) -> CircuitState {
        self.circuits
            .get(backend_id)
            .map(|e| e.value().lock().state)
            .unwrap_or(CircuitState::Closed)
    }

    /// Get a snapshot of all backend health states.
    pub fn health_snapshot(&self) -> Vec<BackendHealth> {
        self.circuits
            .iter()
            .map(|entry| {
                let circuit = entry.value().lock();
                BackendHealth {
                    backend_id: entry.key().clone(),
                    state: circuit.state,
                    failure_count: circuit.failure_count,
                    total_requests: circuit.total_requests,
                    total_rejections: circuit.total_rejections,
                    time_in_state: circuit.last_state_change.elapsed(),
                }
            })
            .collect()
    }

    /// Total number of rejected requests across all backends.
    pub fn total_rejections(&self) -> u64 {
        self.global_rejections.load(Ordering::Relaxed)
    }

    /// Reset a specific backend's circuit to closed.
    pub fn reset(&self, backend_id: &str) {
        if let Some(entry) = self.circuits.get(backend_id) {
            let mut circuit = entry.value().lock();
            circuit.transition(CircuitState::Closed);
            circuit.current_recovery_timeout = circuit.config.recovery_timeout;
            info!("Circuit breaker {backend_id}: manually reset to closed");
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

/// Health status snapshot for a single backend.
#[derive(Debug, Clone)]
pub struct BackendHealth {
    pub backend_id: String,
    pub state: CircuitState,
    pub failure_count: u32,
    pub total_requests: u64,
    pub total_rejections: u64,
    pub time_in_state: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circuit_starts_closed() {
        let cb = CircuitBreaker::new();
        assert_eq!(cb.state("test-backend"), CircuitState::Closed);
        assert!(cb.should_allow("test-backend"));
    }

    #[test]
    fn circuit_opens_after_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        let cb = CircuitBreaker::with_config(config);

        for _ in 0..3 {
            cb.should_allow("backend-a");
            cb.record_failure("backend-a");
        }

        assert_eq!(cb.state("backend-a"), CircuitState::Open);
        assert!(!cb.should_allow("backend-a"));
    }

    #[test]
    fn success_decays_failure_count() {
        let config = CircuitBreakerConfig {
            failure_threshold: 5,
            ..Default::default()
        };
        let cb = CircuitBreaker::with_config(config);

        // Record some failures but not enough to open
        for _ in 0..3 {
            cb.should_allow("backend-b");
            cb.record_failure("backend-b");
        }
        // Success should decay count
        cb.record_success("backend-b");

        assert_eq!(cb.state("backend-b"), CircuitState::Closed);
    }

    #[test]
    fn health_snapshot_works() {
        let cb = CircuitBreaker::new();
        cb.should_allow("a");
        cb.should_allow("b");
        cb.record_failure("a");

        let snap = cb.health_snapshot();
        assert_eq!(snap.len(), 2);
    }

    #[test]
    fn manual_reset() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            ..Default::default()
        };
        let cb = CircuitBreaker::with_config(config);
        cb.should_allow("x");
        cb.record_failure("x");
        assert_eq!(cb.state("x"), CircuitState::Open);

        cb.reset("x");
        assert_eq!(cb.state("x"), CircuitState::Closed);
    }
}
