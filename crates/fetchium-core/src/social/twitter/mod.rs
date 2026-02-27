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
pub mod oembed;
pub mod pipeline;
pub mod realtime;
pub mod search;
pub mod sentiment;
pub mod trends;
pub mod types;

pub use types::*;
