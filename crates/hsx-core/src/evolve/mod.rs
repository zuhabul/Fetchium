//! Self-Evolving Architecture — AutoML weight tuning, A/B testing, retrain (PRD §39).

pub mod ab_test;
pub mod automl;
pub mod retrain;

pub use automl::HyperFusionWeights;
pub use ab_test::AbTest;
pub use retrain::RetrainReport;
