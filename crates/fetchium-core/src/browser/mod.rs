//! Browser pool module — managed headless Chromium instances (PRD §8.3).
//!
//! Only available when compiled with `--features headless`.
//! Used by CEP Layer 3-5 and headless search backends (Google, Bing, Scholar).

pub mod pool;
pub mod tab;

#[cfg(feature = "headless")]
pub use pool::{BrowserPool, BrowserTier};
#[cfg(feature = "headless")]
pub use tab::ManagedTab;
