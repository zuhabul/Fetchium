//! `fetchium intelligence` sub-commands — manage the Persistent Intelligence Engine.
//!
//! Sub-commands:
//! - `stats`  — show counts for all 4 PIE layers
//! - `reset`  — clear all learned data
//! - `export` — export PIE data as JSON
//! - `suggest <topic>` — predict follow-up queries for a topic
//! - `trust <domain>` — show trust score for a domain
//! - `totr <query>` — run Tree-of-Thoughts decomposition

use clap::Subcommand;
use colored::Colorize;
use fetchium_core::config::FetchiumConfig;
use fetchium_core::intelligence::{
    cce::ConfidenceCalibrationEngine,
    intelligence_data_dir,
    pie::PersistentIntelligenceEngine,
    totr::{run_totr_sync, TotrConfig},
};

// ─── Clap sub-commands ────────────────────────────────────────────────────────

#[derive(Debug, Subcommand)]
pub enum IntelligenceSubcmd {
    /// Show statistics for all PIE layers.
    Stats,

    /// Clear all learned intelligence data (irreversible).
    Reset {
        /// Skip the confirmation prompt.
        #[arg(long)]
        yes: bool,
    },

    /// Export all PIE data as pretty-printed JSON.
    Export {
        /// Write output to file instead of stdout.
        #[arg(long, short)]
        output: Option<String>,
    },

    /// Predict follow-up queries for a topic.
    Suggest {
        /// Topic to generate follow-ups for.
        topic: String,

        /// Maximum number of suggestions (default: 5).
        #[arg(long, default_value = "5")]
        limit: usize,
    },

    /// Show the learned trust score for a domain.
    Trust {
        /// Domain name, e.g. "arxiv.org".
        domain: String,
    },

    /// Run Tree-of-Thoughts decomposition on a complex query.
    Totr {
        /// The complex query to decompose.
        query: String,

        /// Maximum number of reasoning branches (2-5).
        #[arg(long, default_value = "3")]
        branches: usize,

        /// Enable the Advocate-Critic-Judge self-debate protocol.
        #[arg(long)]
        debate: bool,

        /// Output as JSON instead of markdown.
        #[arg(long)]
        json: bool,
    },
}

// ─── Handler ─────────────────────────────────────────────────────────────────

pub async fn run(
    config: &FetchiumConfig,
    sub: IntelligenceSubcmd,
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = config; // available for future use
    match sub {
        IntelligenceSubcmd::Stats => cmd_stats(),
        IntelligenceSubcmd::Reset { yes } => cmd_reset(yes),
        IntelligenceSubcmd::Export { output } => cmd_export(output),
        IntelligenceSubcmd::Suggest { topic, limit } => cmd_suggest(&topic, limit),
        IntelligenceSubcmd::Trust { domain } => cmd_trust(&domain),
        IntelligenceSubcmd::Totr {
            query,
            branches,
            debate,
            json,
        } => cmd_totr(&query, branches, debate, json),
    }
}

// ─── Command implementations ─────────────────────────────────────────────────

fn cmd_stats() -> Result<(), Box<dyn std::error::Error>> {
    let pie = PersistentIntelligenceEngine::new()?;
    let stats = pie.stats()?;

    println!("{}", "PIE Statistics".bold().cyan());
    println!("{}", "─".repeat(40));
    println!(
        "  {:.<30} {}",
        "Entities (PKG)",
        stats.entities.to_string().yellow()
    );
    println!(
        "  {:.<30} {}",
        "Relationships (PKG)",
        stats.relationships.to_string().yellow()
    );
    println!(
        "  {:.<30} {}",
        "Tracked domains (STM)",
        stats.tracked_domains.to_string().yellow()
    );
    println!(
        "  {:.<30} {}",
        "Failure patterns (FPM)",
        stats.failure_patterns.to_string().yellow()
    );
    println!(
        "  {:.<30} {}",
        "Query history (QPM)",
        stats.query_history_size.to_string().yellow()
    );

    // CCE calibration summary
    let cce_db = intelligence_data_dir().join("calibration.db");
    if cce_db.exists() {
        if let Ok(cce) = ConfidenceCalibrationEngine::new(&cce_db) {
            if let Ok(summary) = cce.calibration_summary() {
                println!();
                println!("{}", "CCE Calibration Coverage".bold().cyan());
                println!("{}", "─".repeat(40));
                if summary.is_empty() {
                    println!("  No calibration data yet.");
                } else {
                    for (cat, bins, preds) in &summary {
                        println!(
                            "  {:.<28} {} bins, {} predictions",
                            cat,
                            bins.to_string().yellow(),
                            preds.to_string().yellow(),
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

fn cmd_reset(yes: bool) -> Result<(), Box<dyn std::error::Error>> {
    if !yes {
        print!(
            "{} This will delete ALL learned intelligence data. Continue? [y/N] ",
            "Warning:".red().bold()
        );
        use std::io::BufRead;
        let _ = std::io::Write::flush(&mut std::io::stdout());
        let mut line = String::new();
        std::io::stdin().lock().read_line(&mut line)?;
        if !line.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }
    let pie = PersistentIntelligenceEngine::new()?;
    pie.reset_all()?;
    println!("{}", "All PIE data cleared.".green());
    Ok(())
}

fn cmd_export(output: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let pie = PersistentIntelligenceEngine::new()?;
    let json = pie.export_json()?;
    match output {
        Some(path) => {
            std::fs::write(&path, &json)?;
            println!("{} PIE data exported to {path}", "✓".green());
        }
        None => println!("{json}"),
    }
    Ok(())
}

fn cmd_suggest(topic: &str, limit: usize) -> Result<(), Box<dyn std::error::Error>> {
    let pie = PersistentIntelligenceEngine::new()?;
    let suggestions = pie.qpm.predict_follow_ups(topic, limit)?;

    if suggestions.is_empty() {
        println!(
            "{} No follow-up suggestions for '{}' yet. Run more searches on this topic.",
            "ℹ".blue(),
            topic
        );
    } else {
        println!(
            "{} for topic '{}'\n",
            "Suggested follow-up queries".bold().cyan(),
            topic
        );
        for (i, s) in suggestions.iter().enumerate() {
            println!("  {}. {}", i + 1, s);
        }
    }
    Ok(())
}

fn cmd_trust(domain: &str) -> Result<(), Box<dyn std::error::Error>> {
    let pie = PersistentIntelligenceEngine::new()?;
    let trust = pie.stm.get_trust(domain)?;
    let color_trust = format!("{:.1}%", trust * 100.0);
    let colored = if trust >= 0.7 {
        color_trust.green()
    } else if trust >= 0.4 {
        color_trust.yellow()
    } else {
        color_trust.red()
    };
    println!(
        "Trust score for {}: {} (Bayesian Beta distribution)",
        domain.bold(),
        colored,
    );
    Ok(())
}

fn cmd_totr(
    query: &str,
    branches: usize,
    debate: bool,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let branches = branches.clamp(2, 5);
    let config = TotrConfig {
        max_branches: branches,
        prune_threshold: 0.3,
        self_debate: debate,
    };

    println!(
        "{} Decomposing query into {} perspectives...",
        "ToTR".bold().cyan(),
        branches
    );

    // Heuristic sync run (no async LLM required at CLI level).
    let result = run_totr_sync(query, &config, |_q| {
        // In the CLI, we don't do live search here — just decompose.
        // For full research, use `fetchium deep "query" --tree-of-thoughts`.
        vec![]
    });

    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("{}", result.to_markdown());
        println!(
            "\n{} Total: {} branches, {} pruned",
            "Summary:".bold(),
            result.total_branches,
            result.pruned_branches,
        );
    }
    Ok(())
}
