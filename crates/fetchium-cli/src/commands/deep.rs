//! `fetchium deep` — deep multi-agent research with evidence graphs (Mode E, PRD §8.8).

use crate::cli::DeepArgs;
use console::style;
use fetchium_core::config::HsxConfig;
use fetchium_core::http::client::HttpClient;
use fetchium_core::research::amrs::{AmrsConfig, Coordinator};
use fetchium_core::resource;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub async fn run(args: DeepArgs, config: &HsxConfig) -> anyhow::Result<()> {
    let http_client = HttpClient::new(config)?;

    // Set up progress spinner
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} [{elapsed_precise}] {msg}")
            .unwrap(),
    );
    spinner.set_message(format!("Deep research: {}...", &args.query));
    spinner.enable_steady_tick(Duration::from_millis(100));

    // Build AMRS config from resource tier (respect --max-depth CLI flag)
    let resource_tier = resource::detect_tier();
    let mut amrs_config = AmrsConfig::from_resource_tier(&resource_tier);
    amrs_config.max_depth = args.max_depth as usize;
    if args.max_agents > 0 {
        amrs_config.max_agents = args.max_agents as usize;
    }
    if let Some(timeout) = args.timeout {
        amrs_config.timeout_secs = timeout;
    }

    let result = if args.tree_of_thoughts {
        spinner.set_message("ToTR: Decomposing query...");

        // For simplicity, using heuristic decomposition in CLI to avoid complex trait bounds if AI client isn't directly exposed
        let perspectives = fetchium_core::intelligence::totr::decompose_query_heuristic(
            &args.query,
            amrs_config.max_agents.max(2),
        );

        spinner.set_message(format!(
            "ToTR: Generated {} reasoning paths",
            perspectives.len()
        ));

        let mut branches = Vec::new();
        for (i, (persp, queries)) in perspectives.into_iter().enumerate() {
            let mut findings = Vec::new();
            for q in &queries {
                spinner.set_message(format!("ToTR Path {}: {}", i + 1, q));
                let mut coord =
                    Coordinator::new(amrs_config.clone(), http_client.clone(), config.clone());
                if let Ok(res) = coord.run(q).await {
                    findings.push(res.report);
                }
            }
            branches.push(fetchium_core::intelligence::totr::ThoughtBranch {
                id: format!("branch-{}", i),
                perspective: persp,
                sub_queries: queries,
                findings,
                score: 0.8, // simplified scoring for now
                status: fetchium_core::intelligence::totr::BranchStatus::Active,
            });
        }

        fetchium_core::intelligence::totr::score_and_prune(&mut branches, 0.3);
        let synthesis = fetchium_core::intelligence::totr::synthesize(&branches, &args.query);
        let debate = if args.self_debate {
            Some(fetchium_core::intelligence::totr::self_debate_heuristic(
                &branches,
                &args.query,
            ))
        } else {
            None
        };

        let totr_res = fetchium_core::intelligence::totr::TotrResult {
            query: args.query.clone(),
            branches: branches.clone(),
            synthesis,
            debate,
            total_branches: branches.len(),
            pruned_branches: branches
                .iter()
                .filter(|b| {
                    matches!(
                        b.status,
                        fetchium_core::intelligence::totr::BranchStatus::Pruned { .. }
                    )
                })
                .count(),
        };

        // Wrap ToTR result into standard format
        let report = totr_res.to_markdown();
        let mut coordinator = Coordinator::new(amrs_config, http_client, config.clone());
        let mut dummy_res = coordinator.run(&args.query).await?; // fast dummy or partial
        dummy_res.report = report;
        dummy_res
    } else {
        let mut coordinator = Coordinator::new(amrs_config, http_client, config.clone());
        coordinator.run(&args.query).await?
    };

    spinner.finish_and_clear();

    // ─── Print Report ────────────────────────────────────────────
    println!("\n{}", style("Deep Research Report").bold().cyan());
    println!("{}\n", style("=".repeat(60)).dim());
    println!("{}", result.report);

    // ─── Contradictions ──────────────────────────────────────────
    if !result.contradictions.is_empty() {
        println!("\n{}", style("Contradictions Found").bold().yellow());
        println!("{}", style("-".repeat(40)).dim());
        for (i, c) in result.contradictions.iter().enumerate() {
            println!(
                "  {}. {} (severity: {:.0}%)\n     Source A says: {}\n     Source B says: {}\n",
                i + 1,
                c.claim,
                c.severity * 100.0,
                c.source_a_says,
                c.source_b_says,
            );
        }
    }

    // ─── Audit Trail ─────────────────────────────────────────────
    if args.audit {
        println!("\n{}", style("Audit Trail").bold().dim());
        println!("{}", style("-".repeat(40)).dim());
        for entry in &result.audit_trail {
            println!(
                "  [{}] {}: {} — {}",
                entry.timestamp.format("%H:%M:%S"),
                entry.agent,
                entry.action,
                entry.detail,
            );
        }
    }

    // ─── Evidence Graph Export ────────────────────────────────────
    if args.evidence_graph {
        let graph_json = serde_json::to_string_pretty(&result.evidence_graph)?;
        let path = format!(
            "evidence_graph_{}.json",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );
        std::fs::write(&path, &graph_json)?;
        println!("\nEvidence graph saved to: {}", style(&path).green());
    }

    // ─── File Output ─────────────────────────────────────────────
    if let Some(output_path) = &args.output {
        std::fs::write(output_path, &result.report)?;
        println!("Report saved to: {}", style(output_path).green());
    }

    // ─── Summary Footer ───────────────────────────────────────────
    println!("\n{}", style("-".repeat(60)).dim());
    println!(
        "Sources: {} │ Claims verified: {} │ Contradictions: {} │ Depth: {}",
        style(result.sources_analyzed).cyan(),
        style(result.claims_verified).cyan(),
        style(result.contradictions.len()).yellow(),
        style(result.depth_reached).cyan(),
    );

    Ok(())
}
