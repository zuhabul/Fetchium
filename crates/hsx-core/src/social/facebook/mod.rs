//! Facebook Social Intelligence module.
//!
//! ## Approach
//!
//! Facebook is the most restrictive major social platform — it blocks direct
//! scraping and requires OAuth for most Graph API endpoints. We use a
//! tiered approach:
//!
//! 1. **DDG `site:facebook.com`** — completely free, finds public posts/pages
//! 2. **OpenGraph metadata** — public OG tags give title, description, type
//! 3. **Graph API** — richer data when an optional app token is supplied
//!    (token = `{APP_ID}|{APP_SECRET}`, available for free from developers.facebook.com)
//!
//! ## Limitations
//!
//! - Engagement counts (likes/shares/comments) require Graph API token
//! - Private profiles and groups are inaccessible (by design)
//! - Post content from personal profiles requires user OAuth
//!
//! ## Headless Browser Note
//!
//! For maximum data richness on Facebook, enable the `headless` feature and
//! use `hsx-core::browser` — Chromium can render JS-gated content. This is
//! the most reliable approach but requires ~200MB of browser binary.

pub mod analysis;
pub mod pipeline;
pub mod search;
pub mod types;

pub use types::*;
