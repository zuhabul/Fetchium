//! `hsx cache` — cache management.

use crate::cli::{CacheAction, CacheArgs};
use hsx_core::config::HsxConfig;

pub async fn run(args: CacheArgs, _config: &HsxConfig) -> anyhow::Result<()> {
    match args.action {
        CacheAction::Stats => {
            println!("Cache statistics:");
            // TODO: Implement in Phase 1 (P1-E6-T1)
            println!("  Memory entries: 0");
            println!("  Disk size: 0 MB");
        }
        CacheAction::Clear => {
            println!("Cache cleared");
            // TODO: Implement
        }
        CacheAction::Prune => {
            println!("Expired entries pruned");
            // TODO: Implement
        }
    }
    Ok(())
}
