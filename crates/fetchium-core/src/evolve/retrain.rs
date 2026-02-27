//! CEP retrain report generation (PRD §39).
//!
//! Analyses the CEP ML predictor's performance over recent extractions
//! and generates a report recommending whether retraining is needed.

use crate::error::HsxResult;
use serde::{Deserialize, Serialize};

/// A data point for the retrain analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionRecord {
    pub url: String,
    /// Layer predicted by the ML predictor.
    pub predicted_layer: u8,
    /// Layer actually used after escalation.
    pub actual_layer: u8,
    /// Token count extracted.
    pub tokens_extracted: usize,
    pub success: bool,
    pub recorded_at: String,
}

/// Report summarising whether the CEP predictor needs retraining.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrainReport {
    pub total_samples: usize,
    pub correct_predictions: usize,
    pub accuracy: f64,
    pub layer_confusion: Vec<LayerConfusion>,
    pub recommendation: String,
    pub generated_at: String,
}

/// Number of times predicted layer X was actually Y.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerConfusion {
    pub predicted: u8,
    pub actual: u8,
    pub count: usize,
}

impl RetrainReport {
    /// Build a retrain report from a set of extraction records.
    pub fn from_records(records: &[ExtractionRecord]) -> HsxResult<Self> {
        let total = records.len();
        if total == 0 {
            return Ok(Self {
                total_samples: 0,
                correct_predictions: 0,
                accuracy: 1.0,
                layer_confusion: vec![],
                recommendation: "No data — retrain not applicable.".into(),
                generated_at: chrono::Utc::now().to_rfc3339(),
            });
        }

        let correct = records
            .iter()
            .filter(|r| r.predicted_layer == r.actual_layer)
            .count();
        let accuracy = correct as f64 / total as f64;

        // Build confusion counts
        let mut conf: std::collections::HashMap<(u8, u8), usize> = std::collections::HashMap::new();
        for r in records {
            if r.predicted_layer != r.actual_layer {
                *conf.entry((r.predicted_layer, r.actual_layer)).or_insert(0) += 1;
            }
        }
        let mut layer_confusion: Vec<LayerConfusion> = conf
            .into_iter()
            .map(|((p, a), c)| LayerConfusion {
                predicted: p,
                actual: a,
                count: c,
            })
            .collect();
        layer_confusion.sort_by(|a, b| b.count.cmp(&a.count));

        let recommendation = if accuracy >= 0.90 {
            format!(
                "Predictor healthy ({:.1}% accuracy). No retrain needed.",
                accuracy * 100.0
            )
        } else if accuracy >= 0.75 {
            format!(
                "Moderate drift ({:.1}% accuracy). Consider retraining when >1000 new samples collected.",
                accuracy * 100.0
            )
        } else {
            format!(
                "Significant drift ({:.1}% accuracy). Retrain recommended immediately.",
                accuracy * 100.0
            )
        };

        Ok(Self {
            total_samples: total,
            correct_predictions: correct,
            accuracy,
            layer_confusion,
            recommendation,
            generated_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub fn to_markdown(&self) -> String {
        let mut md = format!(
            "# CEP Predictor Retrain Report\n\n\
             | Metric | Value |\n|--------|-------|\n\
             | Samples | {} |\n\
             | Correct | {} |\n\
             | Accuracy | {:.1}% |\n\
             | Generated | {} |\n\n",
            self.total_samples,
            self.correct_predictions,
            self.accuracy * 100.0,
            self.generated_at,
        );
        md.push_str(&format!("**Recommendation:** {}\n\n", self.recommendation));
        if !self.layer_confusion.is_empty() {
            md.push_str("## Top Layer Confusions\n\n| Predicted | Actual | Count |\n|-----------|--------|-------|\n");
            for lc in self.layer_confusion.iter().take(10) {
                md.push_str(&format!(
                    "| L{} | L{} | {} |\n",
                    lc.predicted, lc.actual, lc.count
                ));
            }
        }
        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(pred: u8, actual: u8) -> ExtractionRecord {
        ExtractionRecord {
            url: "https://example.com".into(),
            predicted_layer: pred,
            actual_layer: actual,
            tokens_extracted: 500,
            success: true,
            recorded_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn test_perfect_accuracy() {
        let records: Vec<_> = (0..20).map(|_| make_record(1, 1)).collect();
        let report = RetrainReport::from_records(&records).unwrap();
        assert!((report.accuracy - 1.0).abs() < 1e-9);
        assert!(report.recommendation.contains("healthy"));
    }

    #[test]
    fn test_low_accuracy_triggers_retrain() {
        let mut records: Vec<ExtractionRecord> = (0..10).map(|_| make_record(1, 1)).collect();
        records.extend((0..90).map(|_| make_record(1, 3)));
        let report = RetrainReport::from_records(&records).unwrap();
        assert!(report.accuracy < 0.75);
        assert!(report.recommendation.contains("immediately"));
    }

    #[test]
    fn test_empty_records() {
        let report = RetrainReport::from_records(&[]).unwrap();
        assert_eq!(report.total_samples, 0);
    }
}
