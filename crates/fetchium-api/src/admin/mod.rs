//! Admin backend — session-authenticated internal operations console.
//! All routes live under /internal/admin/* and require AdminAuth extractor.

pub mod auth;
pub mod db;
pub mod rbac;
