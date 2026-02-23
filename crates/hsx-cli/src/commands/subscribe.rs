//! `hsx subscribe` — topic subscriptions (PRD §33).

use clap::Subcommand;
use colored::Colorize;
use hsx_core::proactive::subscription::{NotifyMethod, SubscriptionStore};
use hsx_core::proactive::parse_interval;

#[derive(Debug, Subcommand)]
pub enum SubscribeCommand {
    /// Add a subscription.
    Add {
        /// Topic to watch.
        topic: String,
        /// Check interval (e.g. 1h, 24h, 7d). Default: 1h.
        #[arg(long, default_value = "1h")]
        interval: String,
        /// Webhook URL for notifications.
        #[arg(long)]
        webhook: Option<String>,
    },
    /// List active subscriptions.
    List,
    /// Remove a subscription by ID.
    Remove {
        /// Subscription ID.
        id: String,
    },
    /// Check which subscriptions are due now.
    Due,
}

pub fn run(cmd: SubscribeCommand) -> anyhow::Result<()> {
    let db_path = SubscriptionStore::default_db_path();
    if let Some(p) = db_path.parent() {
        std::fs::create_dir_all(p)?;
    }
    let store = SubscriptionStore::new(&db_path)?;

    match cmd {
        SubscribeCommand::Add {
            topic,
            interval,
            webhook,
        } => {
            let secs = parse_interval(&interval)
                .map_err(|_| anyhow::anyhow!("Invalid interval '{}'. Use e.g. 30s, 5m, 2h, 7d", interval))?;
            let method = if let Some(url) = webhook {
                NotifyMethod::Webhook { url }
            } else {
                NotifyMethod::Stdout
            };
            let id = store.add(&topic, secs, &method)?;
            println!("{} Subscribed to '{}' (ID: {})", "OK".green(), topic, id);
        }
        SubscribeCommand::List => {
            let subs = store.list()?;
            if subs.is_empty() {
                println!("{} No active subscriptions.", "i".blue());
            } else {
                println!("{}", "Subscriptions".bold().cyan());
                println!("{}", "\u{2500}".repeat(50));
                for s in &subs {
                    let id_short = &s.id[..8.min(s.id.len())];
                    let last = s.last_checked_at.as_deref().unwrap_or("never");
                    println!(
                        "  {} \u{2014} {} (every {}s, last: {})",
                        id_short.yellow(),
                        s.topic,
                        s.interval_secs,
                        last.dimmed()
                    );
                }
            }
        }
        SubscribeCommand::Remove { id } => {
            if store.remove(&id)? {
                println!("{} Subscription {} removed.", "OK".green(), id);
            } else {
                eprintln!("Subscription '{}' not found.", id);
            }
        }
        SubscribeCommand::Due => {
            let due = store.due()?;
            if due.is_empty() {
                println!("{} No subscriptions are due for a check.", "OK".green());
            } else {
                println!("{} subscriptions due:", due.len().to_string().yellow());
                for s in &due {
                    let id_short = &s.id[..8.min(s.id.len())];
                    println!("  - {} ({})", s.topic, id_short.dimmed());
                }
            }
        }
    }
    Ok(())
}
