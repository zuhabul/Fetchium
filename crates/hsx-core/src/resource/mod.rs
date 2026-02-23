//! System resource profiling and monitoring (PRD §13).

use crate::types::ResourceTier;

/// Detect the current system resource tier.
pub fn detect_tier() -> ResourceTier {
    crate::config::HsxConfig::detect_resource_tier()
}
