//! `hsx deep` — deep multi-agent research (Mode E).

use crate::cli::DeepArgs;
use hsx_core::config::HsxConfig;

pub async fn run(args: DeepArgs, _config: &HsxConfig) -> anyhow::Result<()> {
    println!("Deep research: {}", args.query);
    // TODO: Implement in Phase 4 (P4-E2-T1)
    Ok(())
}
