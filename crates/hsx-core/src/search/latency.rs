//! Latency Predictor (LP) — predicts backend response times for optimal timeout tuning.
//!
//! Uses exponential moving average (EMA) to track per-backend latency and dynamically
//! adjust timeouts. Fast backends get tighter timeouts; slow backends get more slack.
//! This prevents a single slow backend from blocking the entire search.

use crate::types::BackendId;
use std::collections::HashMap;
use std::time::Duration;

/// Configuration for the latency predictor.
#[derive(Debug, Clone)]
pub struct LatencyConfig {
    /// EMA smoothing factor (0.0–1.0). Higher = more weight to recent observations.
    pub alpha: f64,
    /// Minimum timeout for any backend.
    pub min_timeout: Duration,
    /// Maximum timeout for any backend.
    pub max_timeout: Duration,
    /// Multiplier applied to predicted latency to get timeout (headroom).
    pub timeout_multiplier: f64,
    /// Default predicted latency for unknown backends.
    pub default_latency: Duration,
}

impl Default for LatencyConfig {
    fn default() -> Self {
        Self {
            alpha: 0.3,
            min_timeout: Duration::from_millis(500),
            max_timeout: Duration::from_secs(30),
            timeout_multiplier: 2.5,
            default_latency: Duration::from_secs(3),
        }
    }
}

/// Per-backend latency statistics.
#[derive(Debug, Clone)]
pub struct LatencyStats {
    /// Exponential moving average of response time.
    pub ema_ms: f64,
    /// Exponential moving average of variance (for percentile estimation).
    pub ema_variance: f64,
    /// Total number of observations recorded.
    pub sample_count: u64,
    /// Minimum observed latency.
    pub min_ms: f64,
    /// Maximum observed latency.
    pub max_ms: f64,
    /// Number of timeouts observed.
    pub timeout_count: u64,
}

impl LatencyStats {
    fn new(initial_ms: f64) -> Self {
        Self {
            ema_ms: initial_ms,
            ema_variance: 0.0,
            sample_count: 0,
            min_ms: initial_ms,
            max_ms: initial_ms,
            timeout_count: 0,
        }
    }
}

/// Latency predictor with per-backend tracking.
#[derive(Debug, Clone)]
pub struct LatencyPredictor {
    config: LatencyConfig,
    stats: HashMap<BackendId, LatencyStats>,
}

impl LatencyPredictor {
    /// Create a new predictor with the given configuration.
    pub fn new(config: LatencyConfig) -> Self {
        Self {
            config,
            stats: HashMap::new(),
        }
    }

    /// Record a successful response latency for a backend.
    pub fn record_latency(&mut self, backend: &BackendId, latency: Duration) {
        let ms = latency.as_secs_f64() * 1000.0;
        let alpha = self.config.alpha;
        let default_ms = self.config.default_latency.as_secs_f64() * 1000.0;

        let stats = self
            .stats
            .entry(backend.clone())
            .or_insert_with(|| LatencyStats::new(default_ms));

        // Update EMA
        let prev_ema = stats.ema_ms;
        stats.ema_ms = alpha * ms + (1.0 - alpha) * prev_ema;

        // Update variance EMA (for p95 estimation)
        let diff = ms - stats.ema_ms;
        stats.ema_variance = alpha * (diff * diff) + (1.0 - alpha) * stats.ema_variance;

        // Update extremes
        if ms < stats.min_ms || stats.sample_count == 0 {
            stats.min_ms = ms;
        }
        if ms > stats.max_ms || stats.sample_count == 0 {
            stats.max_ms = ms;
        }
        stats.sample_count += 1;
    }

    /// Record a timeout for a backend.
    pub fn record_timeout(&mut self, backend: &BackendId) {
        let default_ms = self.config.default_latency.as_secs_f64() * 1000.0;
        let stats = self
            .stats
            .entry(backend.clone())
            .or_insert_with(|| LatencyStats::new(default_ms));

        stats.timeout_count += 1;
        // Timeouts push the EMA toward max timeout (penalty)
        let max_ms = self.config.max_timeout.as_secs_f64() * 1000.0;
        let alpha = self.config.alpha;
        stats.ema_ms = alpha * max_ms + (1.0 - alpha) * stats.ema_ms;
        stats.sample_count += 1;
    }

    /// Predict the response time for a backend.
    pub fn predict(&self, backend: &BackendId) -> Duration {
        match self.stats.get(backend) {
            Some(stats) if stats.sample_count > 0 => Duration::from_secs_f64(stats.ema_ms / 1000.0),
            _ => self.config.default_latency,
        }
    }

    /// Compute an adaptive timeout for a backend.
    ///
    /// Timeout = predicted_latency * multiplier, clamped to [min, max].
    /// Includes a p95 estimate using variance for extra headroom.
    pub fn adaptive_timeout(&self, backend: &BackendId) -> Duration {
        let predicted = self.predict(backend);
        let predicted_ms = predicted.as_secs_f64() * 1000.0;

        // Add variance-based headroom (approximate p95)
        let variance_headroom = match self.stats.get(backend) {
            Some(stats) if stats.sample_count >= 3 => stats.ema_variance.sqrt() * 1.65,
            _ => 0.0,
        };

        let timeout_ms = (predicted_ms + variance_headroom) * self.config.timeout_multiplier;

        let timeout = Duration::from_secs_f64(timeout_ms / 1000.0);
        timeout.clamp(self.config.min_timeout, self.config.max_timeout)
    }

    /// Get statistics for a specific backend.
    pub fn stats(&self, backend: &BackendId) -> Option<&LatencyStats> {
        self.stats.get(backend)
    }

    /// Rank backends by predicted speed (fastest first).
    pub fn rank_by_speed(&self, backends: &[BackendId]) -> Vec<BackendId> {
        let mut ranked: Vec<(BackendId, Duration)> = backends
            .iter()
            .map(|b| (b.clone(), self.predict(b)))
            .collect();
        ranked.sort_by(|a, b| a.1.cmp(&b.1));
        ranked.into_iter().map(|(b, _)| b).collect()
    }

    /// Get the timeout reliability score for a backend (0.0 = always timeouts, 1.0 = never).
    pub fn reliability(&self, backend: &BackendId) -> f64 {
        match self.stats.get(backend) {
            Some(stats) if stats.sample_count > 0 => {
                1.0 - (stats.timeout_count as f64 / stats.sample_count as f64)
            }
            _ => 0.5, // unknown
        }
    }
}

impl Default for LatencyPredictor {
    fn default() -> Self {
        Self::new(LatencyConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_prediction_for_unknown() {
        let predictor = LatencyPredictor::default();
        let pred = predictor.predict(&BackendId::DuckDuckGo);
        assert_eq!(pred, Duration::from_secs(3));
    }

    #[test]
    fn ema_tracks_latency() {
        let mut predictor = LatencyPredictor::default();
        let backend = BackendId::Google;

        // Record several fast responses
        for _ in 0..10 {
            predictor.record_latency(&backend, Duration::from_millis(200));
        }

        let pred = predictor.predict(&backend);
        // Should converge toward 200ms
        assert!(
            pred.as_millis() < 1000,
            "prediction should approach 200ms, got {:?}",
            pred
        );
    }

    #[test]
    fn timeout_penalty_increases_prediction() {
        let mut predictor = LatencyPredictor::default();
        let backend = BackendId::Bing;

        // Record some normal responses
        for _ in 0..5 {
            predictor.record_latency(&backend, Duration::from_millis(500));
        }
        let before = predictor.predict(&backend);

        // Record timeouts
        for _ in 0..3 {
            predictor.record_timeout(&backend);
        }
        let after = predictor.predict(&backend);

        assert!(after > before, "timeouts should increase prediction");
    }

    #[test]
    fn adaptive_timeout_clamped() {
        let config = LatencyConfig {
            min_timeout: Duration::from_secs(1),
            max_timeout: Duration::from_secs(10),
            ..LatencyConfig::default()
        };
        let mut predictor = LatencyPredictor::new(config);

        // Very fast backend
        for _ in 0..10 {
            predictor.record_latency(&BackendId::DuckDuckGo, Duration::from_millis(50));
        }
        let timeout = predictor.adaptive_timeout(&BackendId::DuckDuckGo);
        assert!(
            timeout >= Duration::from_secs(1),
            "timeout below min: {:?}",
            timeout
        );

        // Very slow backend with timeouts
        for _ in 0..10 {
            predictor.record_timeout(&BackendId::Arxiv);
        }
        let timeout = predictor.adaptive_timeout(&BackendId::Arxiv);
        assert!(
            timeout <= Duration::from_secs(10),
            "timeout above max: {:?}",
            timeout
        );
    }

    #[test]
    fn rank_by_speed_orders_correctly() {
        let mut predictor = LatencyPredictor::default();

        // Fast backend
        for _ in 0..5 {
            predictor.record_latency(&BackendId::DuckDuckGo, Duration::from_millis(100));
        }
        // Slow backend
        for _ in 0..5 {
            predictor.record_latency(&BackendId::Arxiv, Duration::from_millis(2000));
        }
        // Medium backend
        for _ in 0..5 {
            predictor.record_latency(&BackendId::Google, Duration::from_millis(500));
        }

        let ranked =
            predictor.rank_by_speed(&[BackendId::Arxiv, BackendId::DuckDuckGo, BackendId::Google]);

        assert_eq!(ranked[0], BackendId::DuckDuckGo);
        assert_eq!(ranked[2], BackendId::Arxiv);
    }

    #[test]
    fn reliability_tracking() {
        let mut predictor = LatencyPredictor::default();
        let backend = BackendId::Google;

        // 8 successes, 2 timeouts = 80% reliability
        for _ in 0..8 {
            predictor.record_latency(&backend, Duration::from_millis(300));
        }
        for _ in 0..2 {
            predictor.record_timeout(&backend);
        }

        let rel = predictor.reliability(&backend);
        assert!(
            (rel - 0.8).abs() < 0.01,
            "reliability should be ~0.8, got {rel}"
        );
    }

    #[test]
    fn unknown_backend_reliability() {
        let predictor = LatencyPredictor::default();
        assert_eq!(predictor.reliability(&BackendId::Brave), 0.5);
    }

    #[test]
    fn stats_tracking() {
        let mut predictor = LatencyPredictor::default();
        let backend = BackendId::DuckDuckGo;

        predictor.record_latency(&backend, Duration::from_millis(100));
        predictor.record_latency(&backend, Duration::from_millis(500));
        predictor.record_latency(&backend, Duration::from_millis(200));

        let stats = predictor.stats(&backend).unwrap();
        assert_eq!(stats.sample_count, 3);
        assert!(stats.min_ms <= 100.0 + 1.0);
        assert!(stats.max_ms >= 499.0);
    }

    #[test]
    fn variance_affects_timeout() {
        let mut stable = LatencyPredictor::default();
        let mut unstable = LatencyPredictor::default();
        let backend = BackendId::Google;

        // Stable: consistent 300ms
        for _ in 0..10 {
            stable.record_latency(&backend, Duration::from_millis(300));
        }

        // Unstable: alternating 100ms and 500ms
        for i in 0..10 {
            let ms = if i % 2 == 0 { 100 } else { 500 };
            unstable.record_latency(&backend, Duration::from_millis(ms));
        }

        let stable_timeout = stable.adaptive_timeout(&backend);
        let unstable_timeout = unstable.adaptive_timeout(&backend);

        // Unstable backend should get a longer timeout due to higher variance
        assert!(
            unstable_timeout >= stable_timeout,
            "unstable timeout {:?} should be >= stable {:?}",
            unstable_timeout,
            stable_timeout
        );
    }
}
