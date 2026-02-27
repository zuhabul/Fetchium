//! Social Media Intelligence System (SMIS).
//!
//! Multi-platform intelligence: Twitter/X, Reddit, TikTok, HackerNews, YouTube, Facebook.
//!
//! 100% free (no API keys), blazing fast (all platforms in parallel), authentic data.
//!
//! ## Platform Coverage
//!
//! | Platform      | Method                                     | Free? | Speed |
//! |---------------|--------------------------------------------|-------|-------|
//! | Twitter/X     | Nitter RSS + HTML scraping                 | ✅    | Fast  |
//! | Reddit        | Official JSON API (no auth)                | ✅    | Fast  |
//! | TikTok        | Public API + Creative Center               | ✅    | Med   |
//! | HackerNews    | Official Firebase + Algolia API            | ✅    | Fast  |
//! | YouTube       | Invidious/Piped (see youtube module)       | ✅    | Med   |
//! | Facebook      | DDG site:search + Open Graph (+ Graph API) | ✅*   | Slow  |
//!
//! *Facebook: basic data free via DDG/OG; richer data needs Graph API token.
//!
//! ## Headless Browser (optional)
//!
//! Enable `--features headless` to use Chromium for JS-rendered content.
//! This is the most reliable fallback for all platforms, especially Facebook.
//!
//! ## Unified Research Pipeline
//!
//! ```text
//! query → [parallel: Twitter + Reddit + TikTok + HN + YouTube + Facebook]
//!       → normalise → deduplicate (bigram-Jaccard)
//!       → cross-platform trend detection
//!       → ViralBurst detection
//!       → ContentIdeaEngine (20 ideas per run)
//!       → unified markdown report
//! ```

pub mod facebook;
pub mod hackernews;
pub mod reddit;
pub mod tiktok;
pub mod twitter;
pub mod types;
pub mod unified;
pub mod validate;

pub use types::*;
