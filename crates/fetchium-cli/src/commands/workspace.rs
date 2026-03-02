//! `fetchium workspace` — collaborative research workspaces (PRD §37).

use clap::Subcommand;
use colored::Colorize;
use fetchium_core::collab::{fork_session, merge_sessions, sync_workspace, Workspace};

#[derive(Debug, Subcommand)]
pub enum WorkspaceCommand {
    /// Create a new workspace.
    Create {
        /// Workspace name.
        name: String,
        /// Directory path (defaults to ~/.fetchium/workspaces/<name>).
        #[arg(long)]
        path: Option<String>,
    },
    /// List all workspaces.
    List,
    /// Fork a research session into a new branch.
    Fork {
        /// Workspace name.
        workspace: String,
        /// Session ID to fork.
        session_id: String,
        /// Name for the new branch.
        #[arg(long)]
        name: String,
    },
    /// Merge two sessions (with deduplication).
    Merge {
        /// Workspace name.
        workspace: String,
        /// First session ID.
        session_a: String,
        /// Second session ID.
        session_b: String,
    },
    /// Sync workspace to its configured remote.
    Sync {
        /// Workspace name.
        name: String,
    },
}

pub fn run(cmd: WorkspaceCommand) -> anyhow::Result<()> {
    let base = Workspace::default_base_dir();
    std::fs::create_dir_all(&base)?;
    match cmd {
        WorkspaceCommand::Create { name, path } => {
            let dir = path
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| base.join(&name));
            let ws = Workspace::create(&name, &dir)?;
            println!(
                "{} Workspace '{}' created at {:?}",
                "OK".green(),
                ws.name,
                dir
            );
        }
        WorkspaceCommand::List => {
            let workspaces = Workspace::list(&base)?;
            if workspaces.is_empty() {
                println!(
                    "{} No workspaces. Use `fetchium workspace create <name>`.",
                    "i".blue()
                );
            } else {
                println!("{}", "Workspaces".bold().cyan());
                println!("{}", "\u{2500}".repeat(40));
                for ws in &workspaces {
                    println!("  {:.<30} {}", ws.name, ws.created_at.dimmed());
                }
            }
        }
        WorkspaceCommand::Fork {
            workspace,
            session_id,
            name,
        } => {
            let ws_path = base.join(&workspace);
            let sessions_dir = ws_path.join("sessions");
            let meta = fork_session(&sessions_dir, &session_id, &name)?;
            println!(
                "{} Session '{}' forked to '{}' ({})",
                "OK".green(),
                session_id,
                name,
                meta.id
            );
        }
        WorkspaceCommand::Merge {
            workspace,
            session_a,
            session_b,
        } => {
            let ws_path = base.join(&workspace);
            let sessions_dir = ws_path.join("sessions");
            let result = merge_sessions(&sessions_dir, &session_a, &session_b)?;
            println!(
                "{} Merged: {} findings, {} added, {} deduplicated",
                "OK".green(),
                result.merged.len(),
                result.added,
                result.deduplicated
            );
        }
        WorkspaceCommand::Sync { name } => {
            let ws_path = base.join(&name);
            let ws = Workspace::load(&ws_path)?;
            let report = sync_workspace(&ws_path, &ws.sync_method)?;
            println!("{} {}", "OK".green(), report.message);
        }
    }
    Ok(())
}
