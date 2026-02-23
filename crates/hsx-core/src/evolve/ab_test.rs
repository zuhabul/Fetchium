//! A/B testing for ranking variants (PRD §39).
//!
//! Uses a deterministic hash-based assignment so the same user (identified
//! by a stable session ID) always gets the same variant.

use crate::error::{HsxError, HsxResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// An A/B test definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbTest {
    pub name: String,
    pub description: String,
    /// Traffic split: 0.0–1.0 fraction going to `variant_b`.
    pub split_ratio: f64,
    pub variant_a: String,
    pub variant_b: String,
    pub is_active: bool,
    pub created_at: String,
}

/// The variant assigned to a session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Variant {
    A,
    B,
}

impl Variant {
    pub fn label(&self) -> &str {
        match self {
            Variant::A => "A",
            Variant::B => "B",
        }
    }
}

impl AbTest {
    pub fn new(name: &str, description: &str, variant_a: &str, variant_b: &str, split_ratio: f64) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            split_ratio: split_ratio.clamp(0.0, 1.0),
            variant_a: variant_a.to_string(),
            variant_b: variant_b.to_string(),
            is_active: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Deterministically assign a variant to a session ID.
    ///
    /// Uses SHA-256(session_id + test_name) to produce a stable hash,
    /// then maps it to [0, 1) and compares against `split_ratio`.
    pub fn assign(&self, session_id: &str) -> Variant {
        let mut hasher = Sha256::new();
        hasher.update(session_id.as_bytes());
        hasher.update(b":");
        hasher.update(self.name.as_bytes());
        let digest = hasher.finalize();
        // Take first 4 bytes as u32 and map to [0, 1).
        let bucket = u32::from_be_bytes([digest[0], digest[1], digest[2], digest[3]]);
        let fraction = bucket as f64 / u32::MAX as f64;
        if fraction < self.split_ratio {
            Variant::B
        } else {
            Variant::A
        }
    }

    /// Return the variant label name (e.g., "control" or "semantic-boost").
    pub fn variant_name(&self, variant: &Variant) -> &str {
        match variant {
            Variant::A => &self.variant_a,
            Variant::B => &self.variant_b,
        }
    }
}

/// A result observed during an A/B test run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbResult {
    pub test_name: String,
    pub session_id: String,
    pub variant: Variant,
    pub metric: String,
    pub value: f64,
    pub recorded_at: String,
}

impl AbResult {
    pub fn new(test_name: &str, session_id: &str, variant: Variant, metric: &str, value: f64) -> Self {
        Self {
            test_name: test_name.to_string(),
            session_id: session_id.to_string(),
            variant,
            metric: metric.to_string(),
            value,
            recorded_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Aggregate statistics for one variant of a test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantStats {
    pub variant: Variant,
    pub n: usize,
    pub mean: f64,
    pub std_dev: f64,
}

/// Compute per-variant statistics for a set of results.
pub fn compute_stats(results: &[AbResult], metric: &str) -> HsxResult<(VariantStats, VariantStats)> {
    let values_a: Vec<f64> = results
        .iter()
        .filter(|r| r.metric == metric && r.variant == Variant::A)
        .map(|r| r.value)
        .collect();
    let values_b: Vec<f64> = results
        .iter()
        .filter(|r| r.metric == metric && r.variant == Variant::B)
        .map(|r| r.value)
        .collect();

    if values_a.is_empty() || values_b.is_empty() {
        return Err(HsxError::Config(format!(
            "insufficient data for metric '{metric}' in A/B analysis"
        )));
    }

    Ok((stats_for(Variant::A, &values_a), stats_for(Variant::B, &values_b)))
}

fn stats_for(variant: Variant, values: &[f64]) -> VariantStats {
    let n = values.len();
    let mean = values.iter().sum::<f64>() / n as f64;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
    VariantStats { variant, n, mean, std_dev: variance.sqrt() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_assignment() {
        let test = AbTest::new("ranking-v2", "test", "control", "semantic-boost", 0.5);
        let v1 = test.assign("session-abc");
        let v2 = test.assign("session-abc");
        assert_eq!(v1, v2, "assignment must be deterministic");
    }

    #[test]
    fn test_split_ratio_zero_always_a() {
        let test = AbTest::new("test", "desc", "A", "B", 0.0);
        for i in 0..20 {
            assert_eq!(test.assign(&format!("session-{i}")), Variant::A);
        }
    }

    #[test]
    fn test_split_ratio_one_always_b() {
        let test = AbTest::new("test", "desc", "A", "B", 1.0);
        for i in 0..20 {
            assert_eq!(test.assign(&format!("session-{i}")), Variant::B);
        }
    }
}
