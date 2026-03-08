//! Admin backend — session-authenticated internal operations console.
//! All routes live under /internal/admin/* and require AdminAuth extractor.

pub mod anomaly;
pub mod approval;
pub mod audit;
pub mod auth;
pub mod billing;
pub mod campaigns;
pub mod crm;
pub mod db;
pub mod export;
pub mod flags;
pub mod incidents;
pub mod keys;
pub mod metrics;
pub mod orgs;
pub mod proxy_ops;
pub mod rbac;
pub mod support;
pub mod usage;
pub mod users;
