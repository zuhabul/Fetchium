//! Resilience layer — circuit breakers, adaptive rate limiting, health monitoring.
//!
//! Implements production-grade fault tolerance patterns:
//! - **Circuit Breaker**: Per-backend failure detection with half-open recovery
//! - **Adaptive Rate Limiter**: Per-domain AIMD-based rate control
//! - **Bulkhead**: Concurrency isolation to prevent cascade failures
//! - **Request Coalescing**: Dedup identical in-flight requests
//!
//! These patterns ensure Fetchium degrades gracefully under load,
//! never hammers failing backends, and auto-recovers when services heal.

pub mod bulkhead;
pub mod circuit_breaker;
pub mod rate_limiter;

pub use bulkhead::Bulkhead;
pub use circuit_breaker::{CircuitBreaker, CircuitState};
pub use rate_limiter::AdaptiveRateLimiter;
