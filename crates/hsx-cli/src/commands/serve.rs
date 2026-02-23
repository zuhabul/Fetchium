//! `hsx serve` — start API/MCP server.

use crate::cli::ServeArgs;
use hsx_core::config::HsxConfig;

pub async fn run(args: ServeArgs, _config: &HsxConfig) -> anyhow::Result<()> {
    println!("Starting {:?} server on port {}...", args.mode, args.port);
    // TODO: Implement in Phase 4 (P4-E4-T1, P4-E5-T1)
    Ok(())
}
