//! HyperSearchX Core Library
//!
//! AI-native search engine for humans and agents.
//! This crate contains all core algorithms: search backends, content extraction,
//! ranking, validation, caching, and intelligence systems.

pub mod config;
pub mod error;
pub mod types;

// Module stubs — each will be expanded in subsequent phases
pub mod cache;
pub mod extract;
pub mod http;
pub mod index;
pub mod output;
pub mod rank;
pub mod resource;
pub mod search;
pub mod token;

// Phase 3+
pub mod citation;
pub mod validate;

// Phase 4+
pub mod ai;
pub mod research;

// Phase 6+
pub mod intelligence;

// Phase 7+
pub mod collab;
pub mod domain;
pub mod plugin;
pub mod privacy;

/// Re-export commonly used types.
pub mod prelude {
    pub use crate::config::HsxConfig;
    pub use crate::error::{HsxError, HsxResult};
    pub use crate::types::*;
}
