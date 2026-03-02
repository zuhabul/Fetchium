//! `fetchium monitor` — watch a URL for content changes.

use anyhow::{Context, Result};
use fetchium_core::config::HsxConfig;
use fetchium_core::extract::pipeline;
use fetchium_core::http::client::HttpClient;
use fetchium_core::monitor::diff::{compute_diff_summary, DiffLine};
use fetchium_core::monitor::parse_interval;
use fetchium_core::monitor::snapshot::SnapshotStore;

use crate::cli::{MonitorAction, MonitorArgs};

pub async fn run(args: MonitorArgs, config: &HsxConfig) -> Result<()> {
    let store = SnapshotStore::new()?;

    match args.action {
        MonitorAction::Add {
            url,
            interval,
            notify,
        } => {
            let interval_secs = parse_interval(&interval).unwrap_or(3600);
            store.register(&url, interval_secs, notify.as_deref())?;
            println!("Monitoring {url} every {interval} ({}s)", interval_secs);
        }

        MonitorAction::Remove { url } => {
            let removed = store.unregister(&url)?;
            if removed {
                println!("Removed {url} from monitor list.");
            } else {
                eprintln!("URL not found in monitor list: {url}");
            }
        }

        MonitorAction::Check { url } => {
            let http = HttpClient::new(config).context("Failed to build HTTP client")?;
            let fetch_result = http
                .fetch(&url)
                .await
                .with_context(|| format!("Failed to fetch {url}"))?;
            let extracted = pipeline::extract(&fetch_result.body, &fetch_result.url);
            let changed = store.save_snapshot(&url, &extracted.text)?;

            if changed {
                println!("CHANGED: {url}");
                if let (Some(prev), Some(_curr)) =
                    (store.get_previous(&url)?, store.get_latest(&url)?)
                {
                    // Fetch latest after save for diffing
                    if let Some(latest) = store.get_latest(&url)? {
                        let diff = compute_diff_summary(&prev.content, &latest.content);
                        println!(
                            "  +{} lines / -{} lines (similarity {:.0}%)",
                            diff.additions,
                            diff.deletions,
                            diff.similarity * 100.0
                        );
                    }
                }
            } else {
                println!("No change: {url}");
            }
        }

        MonitorAction::List => {
            let entries = store.list_monitors()?;
            if entries.is_empty() {
                println!("No URLs being monitored.");
            } else {
                println!("{:<50} {:>12}  Last checked", "URL", "Interval(s)");
                println!("{}", "-".repeat(80));
                for e in &entries {
                    let last = e.last_checked.as_deref().unwrap_or("never");
                    println!("{:<50} {:>12}  {}", e.url, e.interval_secs, last);
                }
            }
        }

        MonitorAction::Diff { url } => match (store.get_previous(&url)?, store.get_latest(&url)?) {
            (Some(prev), Some(curr)) => {
                let diff = compute_diff_summary(&prev.content, &curr.content);
                println!("Diff for {url}");
                println!(
                    "Additions: {}  Deletions: {}  Similarity: {:.0}%",
                    diff.additions,
                    diff.deletions,
                    diff.similarity * 100.0
                );
                for line in &diff.changes {
                    match line {
                        DiffLine::Added(s) => println!("+ {s}"),
                        DiffLine::Removed(s) => println!("- {s}"),
                        DiffLine::Equal(_) => {}
                    }
                }
            }
            _ => {
                eprintln!(
                    "Not enough snapshots for {url}. \
                         Run `fetchium monitor check {url}` first."
                );
            }
        },
    }

    Ok(())
}
