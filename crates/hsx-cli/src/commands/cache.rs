//! `hsx cache` — cache management.

use crate::cli::{CacheAction, CacheArgs};
use hsx_core::config::HsxConfig;

pub async fn run(args: CacheArgs, config: &HsxConfig) -> anyhow::Result<()> {
    match args.action {
        CacheAction::Stats => {
            println!("Cache statistics:");
            println!("  Memory Cache Enabled: {}", config.cache.enabled);
            println!(
                "  Memory Cache Max Entries: {}",
                config.cache.memory_max_entries
            );
            println!("  Memory Cache TTL: {}s", config.cache.ttl_secs);
            println!("  Disk caching is not yet implemented (SQLite pending).");
        }
        CacheAction::Clear => {
            println!("Memory Cache is ephemeral and clears on exit.");
            println!("Disk caching is not yet implemented.");
        }
        CacheAction::Prune => {
            println!("Memory Cache automatically prunes expired entries.");
            println!("Disk caching is not yet implemented.");
        }
    }
    Ok(())
}
