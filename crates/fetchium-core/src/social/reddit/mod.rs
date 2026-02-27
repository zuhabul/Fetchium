//! Reddit Intelligence System (RIS).
//!
//! Uses Reddit's free public JSON API — no authentication required.
//! Append `.json` to any Reddit URL to get structured data.
//!
//! ## Architecture
//!
//! ```text
//! search (JSON API) → analysis → pipeline
//! ```

pub mod analysis;
pub mod pipeline;
pub mod search;
pub mod types;

pub use types::*;
