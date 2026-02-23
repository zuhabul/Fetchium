//! Workspace filesystem sync (PRD §37).

use crate::collab::workspace::SyncMethod;
use crate::error::HsxError;
use std::path::Path;

/// Sync a workspace to its configured destination.
///
/// For `Local` sync: copies changed files to the shared directory.
/// For `Git` sync: calls `git add -A && git commit && git push`.
pub fn sync_workspace(
    workspace_path: &Path,
    method: &SyncMethod,
) -> Result<SyncReport, HsxError> {
    match method {
        SyncMethod::Local { shared_dir } => sync_local(workspace_path, shared_dir),
        SyncMethod::Git { remote_url } => sync_git(workspace_path, remote_url),
    }
}

#[derive(Debug)]
pub struct SyncReport {
    pub method: String,
    pub files_synced: usize,
    pub message: String,
}

fn sync_local(src: &Path, dst: &Path) -> Result<SyncReport, HsxError> {
    std::fs::create_dir_all(dst)?;
    let count = copy_changed(src, dst)?;
    Ok(SyncReport {
        method: "local".into(),
        files_synced: count,
        message: format!("Synced {count} file(s) to {:?}", dst),
    })
}

fn sync_git(workspace_path: &Path, remote_url: &str) -> Result<SyncReport, HsxError> {
    let git_dir = workspace_path.join(".git");
    if !git_dir.exists() {
        run_git(workspace_path, &["init"])?;
        run_git(workspace_path, &["remote", "add", "origin", remote_url])?;
    }
    run_git(workspace_path, &["add", "-A"])?;
    let timestamp = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    run_git(
        workspace_path,
        &[
            "commit",
            "-m",
            &format!("hsx workspace sync {timestamp}"),
        ],
    )?;
    run_git(workspace_path, &["push", "-u", "origin", "HEAD"])?;
    Ok(SyncReport {
        method: "git".into(),
        files_synced: 0,
        message: format!("Pushed to {remote_url}"),
    })
}

fn run_git(dir: &Path, args: &[&str]) -> Result<(), HsxError> {
    let status = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()?;
    if !status.success() {
        return Err(HsxError::Config(format!(
            "git {} failed",
            args.join(" ")
        )));
    }
    Ok(())
}

fn copy_changed(src: &Path, dst: &Path) -> Result<usize, HsxError> {
    let mut count = 0;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            if entry.file_name() == ".git" {
                continue;
            }
            count += copy_changed(&src_path, &dst_path)?;
        } else {
            let should_copy = if dst_path.exists() {
                let src_meta = std::fs::metadata(&src_path)?;
                let dst_meta = std::fs::metadata(&dst_path)?;
                src_meta.len() != dst_meta.len()
            } else {
                true
            };
            if should_copy {
                if let Some(parent) = dst_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(&src_path, &dst_path)?;
                count += 1;
            }
        }
    }
    Ok(count)
}
