//! Environment setup utilities — Chromium download, path resolution, dependency checks.
//!
//! These functions power `hsx setup` and are also used internally by the browser
//! pool to locate Chrome without manual configuration.

pub mod checker;
pub mod chromium;
pub mod searxng;

pub use chromium::{chrome_binary_in, current_platform, download_chromium, resolve_chrome_path};
