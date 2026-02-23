//! `hsx fetch` / `hsx view` — URL content extraction (Mode D).

use crate::cli::FetchArgs;
use hsx_core::config::HsxConfig;

pub async fn run(args: FetchArgs, _config: &HsxConfig) -> anyhow::Result<()> {
    println!("Fetching: {}", args.url);
    // TODO: Implement in Phase 1 (P1-E1-T3)
    Ok(())
}
