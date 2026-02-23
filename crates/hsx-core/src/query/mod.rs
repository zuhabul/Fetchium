//! Query understanding and augmentation (Phase 5, PRD §21-22).
//!
//! - `hyde.rs` — Hypothetical Document Embeddings for ambiguous queries

pub mod hyde;

pub use hyde::{hyde_prompt, should_use_hyde};

#[cfg(feature = "embeddings")]
pub use hyde::smart_embed;
