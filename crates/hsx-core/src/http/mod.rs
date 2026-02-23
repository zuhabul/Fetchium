//! HTTP client module — pooled reqwest client with retry logic (PRD §14).

pub mod client;

pub use client::HttpClient;
