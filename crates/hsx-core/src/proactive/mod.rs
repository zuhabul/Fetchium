//! Proactive Intelligence — subscriptions, radar, digest, prefetch, anomaly (PRD §33).

pub mod anomaly;
pub mod digest;
pub mod prefetch;
pub mod radar;
pub mod subscription;

pub use digest::{Digest, DigestBuilder, DigestPeriod};
pub use prefetch::{generate_candidates, PrefetchEntry, PrefetchQueue};
pub use radar::{build_radar, RadarItem};
pub use subscription::{parse_interval, NotifyMethod, Subscription, SubscriptionStore};
