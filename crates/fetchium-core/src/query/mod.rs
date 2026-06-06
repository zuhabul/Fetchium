//! Query understanding and augmentation (Phase 5, PRD §21-22).
//!
//! - `hyde.rs` — Hypothetical Document Embeddings for ambiguous queries
//! - `fingerprint.rs` — Query Fingerprinting and Deduplication (QFD)

pub mod autoprompt;
pub mod complexity;
pub mod crosslingual;
pub mod expansion;
pub mod fingerprint;
pub mod hyde;
pub mod locale;

pub use autoprompt::{autoprompt, AutopromptResult};
pub use complexity::{
    estimate_complexity, suggest_intent, ComplexityAssessment, ComplexityConfig, ComplexityLevel,
};
pub use crosslingual::{expand_crosslingual, CrossLingualExpansion, CrossLingualResult, Language};
pub use expansion::{expand_query, should_expand, ExpandedQuery};
pub use fingerprint::{fingerprint as query_fingerprint, QueryFingerprint};
pub use hyde::{hyde_prompt, should_use_hyde};

#[cfg(feature = "embeddings")]
pub use hyde::smart_embed;
