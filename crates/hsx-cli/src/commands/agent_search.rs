//! `hsx agent-search` — agent-optimized search (JSON segments).

use crate::cli::AgentSearchArgs;
use hsx_core::config::HsxConfig;

pub async fn run(args: AgentSearchArgs, _config: &HsxConfig) -> anyhow::Result<()> {
    // Agent commands always output JSON
    let result = serde_json::json!({
        "status": "not_implemented",
        "query": args.query,
        "budget": args.budget,
    });
    println!("{}", serde_json::to_string_pretty(&result)?);
    // TODO: Implement in Phase 1 (P1-E4-T1)
    Ok(())
}
