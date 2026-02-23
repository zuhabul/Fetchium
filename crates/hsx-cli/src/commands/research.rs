//! `hsx research` — multi-source research (Mode B).

use crate::cli::ResearchArgs;
use hsx_core::config::HsxConfig;

pub async fn run(args: ResearchArgs, _config: &HsxConfig) -> anyhow::Result<()> {
    println!("Researching: {}", args.query);
    // TODO: Implement in Phase 3 (P3-E3-T1)
    Ok(())
}
