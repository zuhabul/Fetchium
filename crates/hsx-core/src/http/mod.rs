//! HTTP client module — pooled reqwest client with retry logic (PRD §14).

pub mod client;
pub mod robots;
pub mod sanitize;
pub mod tls;

pub use client::{FetchResult, HttpClient};
