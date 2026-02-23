//! 6-layer validation + RAR self-correction (PRD §19, §8.6).

pub mod authority;
pub mod calibration;
pub mod content;
pub mod cross_source;
pub mod extraction;
pub mod output;
pub mod rar;
pub mod temporal;
pub mod types;

pub use authority::AuthorityScorer;
pub use calibration::ConfidenceCalibrator;
pub use cross_source::{CrossSourceVerifier, SourceContent};
pub use rar::{RarConfig, RarEngine, RarState};
pub use temporal::TemporalValidator;
pub use types::*;
