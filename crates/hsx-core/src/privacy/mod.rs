//! Privacy modes — private, tor, air-gap, PII redaction, auto-expire,
//! cache encryption (PRD §36).

pub mod encryption;
pub mod expiry;
pub mod modes;
pub mod redact;
pub mod tor;

pub use encryption::CacheEncryption;
pub use expiry::ExpiryScheduler;
pub use modes::{apply_mode, PrivacyMode, RuntimeConfig};
pub use redact::redact_pii;
pub use tor::{is_tor_available, require_tor};
