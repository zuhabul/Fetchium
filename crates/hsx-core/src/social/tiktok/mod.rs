//! TikTok Intelligence System (TIS).
//!
//! Free data collection via unofficial public endpoints + Creative Center API.
//!
//! ## Architecture
//!
//! ```text
//! search (public APIs / HTML) → analysis → pipeline
//! ```

pub mod analysis;
pub mod pipeline;
pub mod search;
pub mod types;

pub use types::*;
