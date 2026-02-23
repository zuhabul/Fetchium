//! `hsx search` — web search (Mode A).

use crate::cli::SearchArgs;
use hsx_core::config::HsxConfig;

pub async fn run(args: SearchArgs, _config: &HsxConfig) -> anyhow::Result<()> {
    println!("Searching for: {}", args.query);
    // TODO: Implement in Phase 1 (P1-E2-T3)
    Ok(())
}
