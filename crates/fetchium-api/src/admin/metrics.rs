//! Admin metrics and observability handlers.

use crate::admin::rbac::AdminAuth;
use crate::middleware::AppState;
use axum::{extract::State, Json};
use libc;

pub async fn realtime(_auth: AdminAuth, State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": {}}))
}

pub async fn summary(_auth: AdminAuth, State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": {}}))
}

pub async fn provider_health(
    _auth: AdminAuth,
    State(_state): State<AppState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": []}))
}

/// GET /internal/admin/system/stats
pub async fn system_stats(
    _auth: AdminAuth,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    // Read /proc/self/status for memory
    let mem_used_mb = read_proc_mem_kb().unwrap_or(0) / 1024;
    // Read /proc/meminfo for total
    let mem_total_mb = read_total_mem_kb().unwrap_or(8 * 1024 * 1024) / 1024;
    // Disk from statvfs
    let (disk_used, disk_total) = read_disk_usage("/").unwrap_or((0.0, 1.0));
    // Admin DB size
    let admin_db_kb = state.admin_db.as_ref()
        .and_then(|db| db.db_size_kb().ok())
        .unwrap_or(0);

    Json(serde_json::json!({
        "mem_used_mb": mem_used_mb,
        "mem_total_mb": mem_total_mb,
        "cpu_pct": 0.0_f32,
        "disk_used_gb": disk_used,
        "disk_total_gb": disk_total,
        "admin_db_size_kb": admin_db_kb,
        "api_version": env!("CARGO_PKG_VERSION"),
        "ok": true
    }))
}

fn read_proc_mem_kb() -> Option<u64> {
    let content = std::fs::read_to_string("/proc/self/status").ok()?;
    for line in content.lines() {
        if line.starts_with("VmRSS:") {
            let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
            return Some(kb);
        }
    }
    None
}

fn read_total_mem_kb() -> Option<u64> {
    let content = std::fs::read_to_string("/proc/meminfo").ok()?;
    for line in content.lines() {
        if line.starts_with("MemTotal:") {
            let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
            return Some(kb);
        }
    }
    None
}

fn read_disk_usage(path: &str) -> Option<(f64, f64)> {
    use std::ffi::CString;
    let c_path = CString::new(path).ok()?;
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    let ret = unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) };
    if ret != 0 { return None; }
    let total = stat.f_blocks as f64 * stat.f_frsize as f64 / 1e9;
    let free = stat.f_bfree as f64 * stat.f_frsize as f64 / 1e9;
    Some((total - free, total))
}
