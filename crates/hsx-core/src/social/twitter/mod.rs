//! Twitter/X Intelligence System (XIS).
//!
//! Free, no-API-key scraping via Nitter instances + RSS feeds.
//!
//! ## Architecture
//!
//! ```text
//! search (nitter RSS/HTML) → analysis → pipeline
//! ```

pub mod analysis;
pub mod pipeline;
pub mod search;
pub mod types;

pub use types::*;
