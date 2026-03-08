//! Proxy rotation system for bypassing search engine IP blocks.
//!
//! Features:
//! - Load proxies from file (`~/.fetchium/proxies.txt`)
//! - Round-robin rotation with automatic failover
//! - Per-proxy health tracking (latency, success/fail counts)
//! - Automatic cooldown for failed/blocked proxies
//! - Thread-safe concurrent access via Arc + atomic ops
//! - Live stats and status reporting for admin dashboard

pub mod dataimpulse;
pub mod pool;

pub use dataimpulse::DataImpulseClient;
pub use pool::{ProxyEntry, ProxyPool, ProxyProtocol, ProxyStats, ProxyStatus};
