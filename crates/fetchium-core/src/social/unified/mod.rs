//! Unified Social Research Engine (USRE).
//!
//! Orchestrates Twitter/X + Reddit + TikTok + HackerNews + YouTube simultaneously,
//! deduplicates results, cross-validates trends, produces unified trend reports,
//! and generates fresh, authentic content ideas.
//!
//! ## Algorithm
//!
//! ```text
//! Query → [parallel platform fetches] → normalise → deduplicate
//!       → cross-platform trend detection → viral signal fusion
//!       → content idea generation → unified report
//! ```
//!
//! ## Key Features
//!
//! - **CrossSignal Fusion**: 5-platform engagement weighting
//! - **ViralBurst Detection**: sudden velocity spike across ≥2 platforms
//! - **ContentIdeaEngine**: generates platform-optimised content ideas from viral patterns
//! - **Authenticity Filter**: removes bot/spam signals using cross-platform consensus

pub mod engine;
pub mod ideas;
pub mod trend;

pub use engine::{run_social_pipeline, SocialPipelineRunner};
