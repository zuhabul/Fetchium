//! Confidence calibration — weighted aggregation of 6 layer scores (PRD §19).

use crate::validate::types::*;

/// Per-layer weights for confidence calibration.
pub struct CalibrationWeights {
    pub v1_source: f64,
    pub v2_content: f64,
    pub v3_freshness: f64,
    pub v4_cross_source: f64,
    pub v5_extraction: f64,
    pub v6_output: f64,
}

impl Default for CalibrationWeights {
    fn default() -> Self {
        Self {
            v1_source: 0.15,
            v2_content: 0.25,
            v3_freshness: 0.15,
            v4_cross_source: 0.25,
            v5_extraction: 0.10,
            v6_output: 0.10,
        }
    }
}

/// Aggregates layer results into a single calibrated confidence score.
#[derive(Default)]
pub struct ConfidenceCalibrator {
    weights: CalibrationWeights,
}

impl ConfidenceCalibrator {
    /// Compute weighted confidence from layer results.
    pub fn calibrate(&self, layer_results: &[LayerResult]) -> f64 {
        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;

        for lr in layer_results {
            let w = self.weight_for(lr.layer);
            weighted_sum += lr.score * w;
            weight_total += w;
        }

        if weight_total == 0.0 {
            return 0.5;
        }

        (weighted_sum / weight_total).clamp(0.0, 1.0)
    }

    /// Build a complete `ValidationResult` from all layer results.
    pub fn build_result(
        &self,
        mode: ValidationMode,
        layer_results: Vec<LayerResult>,
        contradictions: Vec<Contradiction>,
        consensus: Vec<ClaimConsensus>,
    ) -> ValidationResult {
        let confidence = self.calibrate(&layer_results);
        let passed = layer_results.iter().all(|lr| lr.passed);
        ValidationResult {
            layers_run: layer_results.iter().map(|lr| lr.layer).collect(),
            layer_results,
            passed,
            confidence,
            contradictions,
            consensus,
            mode,
        }
    }

    fn weight_for(&self, layer: ValidationLayerId) -> f64 {
        match layer {
            ValidationLayerId::V1Source => self.weights.v1_source,
            ValidationLayerId::V2Content => self.weights.v2_content,
            ValidationLayerId::V3Freshness => self.weights.v3_freshness,
            ValidationLayerId::V4CrossSource => self.weights.v4_cross_source,
            ValidationLayerId::V5ExtractionQuality => self.weights.v5_extraction,
            ValidationLayerId::V6OutputIntegrity => self.weights.v6_output,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_perfect_layers() {
        let cal = ConfidenceCalibrator::default();
        let layers = vec![
            LayerResult {
                layer: ValidationLayerId::V1Source,
                passed: true,
                score: 1.0,
                issues: vec![],
                duration_ms: 1,
            },
            LayerResult {
                layer: ValidationLayerId::V2Content,
                passed: true,
                score: 1.0,
                issues: vec![],
                duration_ms: 1,
            },
        ];
        let conf = cal.calibrate(&layers);
        assert!(conf > 0.9);
    }

    #[test]
    fn failed_layer_drops_confidence() {
        let cal = ConfidenceCalibrator::default();
        let layers = vec![LayerResult {
            layer: ValidationLayerId::V4CrossSource,
            passed: false,
            score: 0.1,
            issues: vec![],
            duration_ms: 1,
        }];
        let conf = cal.calibrate(&layers);
        assert!(conf < 0.2);
    }

    #[test]
    fn empty_layers_returns_half() {
        let cal = ConfidenceCalibrator::default();
        assert_eq!(cal.calibrate(&[]), 0.5);
    }

    #[test]
    fn build_result_populates_all_fields() {
        let cal = ConfidenceCalibrator::default();
        let lr = LayerResult {
            layer: ValidationLayerId::V1Source,
            passed: true,
            score: 0.8,
            issues: vec![],
            duration_ms: 5,
        };
        let result = cal.build_result(ValidationMode::Standard, vec![lr], vec![], vec![]);
        assert!(result.passed);
        assert!(result.confidence > 0.0);
        assert_eq!(result.mode, ValidationMode::Standard);
    }
}
