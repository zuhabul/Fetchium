//! Pipeline telemetry and observability.
//!
//! Provides real-time metrics collection for the entire Fetchium pipeline:
//! - **Latency histograms**: Per-backend, per-layer, per-operation timing
//! - **Throughput counters**: Requests/sec, tokens/sec, bytes processed
//! - **Health dashboard**: Backend availability, circuit breaker states, rate limits
//! - **Pipeline tracing**: End-to-end request flow with timing breakdown
//!
//! All metrics are lock-free (atomic operations) for zero-overhead in hot paths.

pub mod metrics;

pub use metrics::{OperationTimer, PipelineMetrics};
