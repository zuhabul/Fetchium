//! HyperSearchX Core Library
//!
//! AI-native search engine for humans and agents.
//! This crate contains all core algorithms: search backends, content extraction,
//! ranking, validation, caching, and intelligence systems.
//!
//! ## Phase 2 additions:
//! - Multi-engine search: 10+ backends (HTTP + headless Chromium)
//! - HyperFusion 8-signal ranking
//! - QADD (Query-Aware DOM Distillation)
//! - CEP Layers 3-5 (headless rendering, PDF, OCR)

pub mod api_facade;
pub mod config;
pub mod error;
pub mod types;

// Core pipeline modules
pub mod cache;
pub mod extract;
pub mod http;
pub mod index;
pub mod output;
pub mod rank;
pub mod resource;
pub mod search;
pub mod token;

// Phase 2: QADD and browser pool
pub mod qadd;
pub mod browser;

// Phase 3+
pub mod citation;
pub mod validate;

// Phase 4+
pub mod ai;
pub mod research;

// Phase 5+ (feature-gated optional subsystems)
#[cfg(feature = "embeddings")]
pub mod embeddings;

// Always-available Phase 5 modules
pub mod compare;
pub mod export;
pub mod monitor;
pub mod query;

// Phase 6+
pub mod intelligence;

// Phase 7+
pub mod collab;
pub mod domain;
pub mod evolve;
pub mod multimodal;
pub mod plugin;
pub mod privacy;
pub mod proactive;

#[cfg(test)]
pub mod test_utils;

/// Re-export commonly used types.
pub mod prelude {
    pub use crate::config::HsxConfig;
    pub use crate::error::{HsxError, HsxResult};
    pub use crate::rank::{detect_intent, hyperfusion_rank, QueryIntent};
    pub use crate::types::*;
}
