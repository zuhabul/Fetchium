//! `fetchium cache` — cache management.

use crate::cli::{CacheAction, CacheArgs};
use fetchium_core::config::FetchiumConfig;

pub async fn run(args: CacheArgs, config: &FetchiumConfig) -> anyhow::Result<()> {
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
