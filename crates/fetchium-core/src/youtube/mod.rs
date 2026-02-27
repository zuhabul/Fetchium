//! YouTube Intelligence System (YIS).
//!
//! World-class YouTube intelligence module: search, scrape, transcribe,
//! analyze, rank, fact-check, and teach from YouTube videos.
//!
//! 100% free (no API keys), blazing fast (parallel async), model-agnostic.
//!
//! ## Architecture
//!
//! ```text
//! search → metadata → transcript → comments → ranking → intelligence → pipeline
//! ```
//!
//! ## Key Features
//! - Multi-source search (Invidious, Piped, yt-dlp fallback)
//! - Enhanced transcript with speaker detection and key moments
//! - VideoFusion 8-signal ranking engine
//! - Cross-video fact checking with negation detection
//! - Learning path generation and teaching mode

pub mod comments;
pub mod intelligence;
pub mod metadata;
pub mod pipeline;
pub mod ranking;
pub mod search;
pub mod transcript;
pub mod types;
pub mod universal;

pub use types::*;
