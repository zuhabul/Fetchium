//! 6-layer validation + RAR self-correction (PRD §19, §8.6).
//!
//! ## Validation Pipeline
//!
//! Results pass through 6 validators in sequence:
//!
//! | Layer | Validator | What it checks |
//! |-------|-----------|----------------|
//! | V1 | `AuthorityScorer` | Domain reputation, SSL, redirect chains |
//! | V2 | `ContentValidator` | Paywall detection, relevance, min length |
//! | V3 | `TemporalValidator` | Freshness with exponential decay |
//! | V4 | `CrossSourceVerifier` | Cross-source claim agreement |
//! | V5 | `ExtractionValidator` | Truncation, encoding errors, segments |
//! | V6 | `OutputValidator` | Citation reachability, content hash drift |
//!
//! The [`ConfidenceCalibrator`] aggregates all layer scores into a single
//! calibrated confidence score. The [`RarEngine`] drives 5-checkpoint
//! self-correction (R1–R5) when confidence falls below threshold.

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
