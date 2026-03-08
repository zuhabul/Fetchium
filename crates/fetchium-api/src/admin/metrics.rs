//! Admin metrics and observability handlers.

use crate::admin::rbac::AdminAuth;
use crate::middleware::AppState;
use axum::{extract::State, Json};
use libc;

pub async fn realtime(_auth: AdminAuth, State(state): State<AppState>) -> Json<serde_json::Value> {
    let (requests_last_hour, active_users, error_rate) = state.admin_db.as_ref()
        .map(|db| {
            let reqs = db.run_select_query(
                "SELECT COUNT(*) FROM audit_events WHERE created_at >= datetime('now','-1 hour')", 1,
            ).ok().and_then(|r| r.rows.first()?.first()?.as_i64()).unwrap_or(0);
            let users = db.run_select_query(
                "SELECT COUNT(DISTINCT admin_user_id) FROM audit_events WHERE created_at >= datetime('now','-1 hour')", 1,
            ).ok().and_then(|r| r.rows.first()?.first()?.as_i64()).unwrap_or(0);
            let errors = db.run_select_query(
                "SELECT COUNT(*) FROM audit_events WHERE (action LIKE '%.error' OR action LIKE '%.fail') AND created_at >= datetime('now','-1 hour')", 1,
            ).ok().and_then(|r| r.rows.first()?.first()?.as_i64()).unwrap_or(0);
            (reqs, users, if reqs > 0 { errors * 100 / reqs } else { 0 })
        })
        .unwrap_or((0, 0, 0));

    Json(serde_json::json!({
        "ok": true,
        "requests_last_hour": requests_last_hour,
        "active_users": active_users,
        "error_rate_pct": error_rate,
        "latency_p50_ms": 0,
        "latency_p99_ms": 0,
    }))
}

pub async fn summary(_auth: AdminAuth, State(state): State<AppState>) -> Json<serde_json::Value> {
    let (total_orgs, open_incidents, open_tickets, admin_db_size_bytes) = state.admin_db.as_ref()
        .map(|db| {
            let orgs = db.count_orgs().unwrap_or(0);
            let incidents = db.list_incidents().unwrap_or_default()
                .iter()
                .filter(|i| i.get("status").and_then(|s| s.as_str()) != Some("resolved"))
                .count() as i64;
            let tickets = db.count_tickets_by_status("open").unwrap_or(0);
            let db_size = db.db_size_bytes();
            (orgs, incidents, tickets, db_size)
        })
        .unwrap_or((0, 0, 0, 0));

    Json(serde_json::json!({
        "ok": true,
        "total_orgs": total_orgs,
        "open_incidents": open_incidents,
        "open_tickets": open_tickets,
        "admin_db_size_bytes": admin_db_size_bytes,
        "jobs_active": 0,
        "jobs_queued": 0,
        "jobs_completed_today": 0,
        "jobs_failed_today": 0,
    }))
}

pub async fn provider_health(
    _auth: AdminAuth,
    State(_state): State<AppState>,
) -> Json<serde_json::Value> {
    // Static provider registry — status checked via config/env presence
    let providers = vec![
        ("Google", "google", std::env::var("PROXY_USER").is_ok()),
        ("DuckDuckGo", "ddg", true),
        ("Bing", "bing", true),
        ("Brave", "brave", true),
        ("SearXNG", "searxng", true),
        ("OpenAI", "openai", std::env::var("OPENAI_API_KEY").is_ok()),
        ("Anthropic", "anthropic", std::env::var("ANTHROPIC_API_KEY").is_ok()),
        ("Gemini", "gemini", std::env::var("GEMINI_API_KEY").is_ok() || std::env::var("GEMINI_API_KEYS").is_ok()),
    ];

    let data: Vec<_> = providers.iter().map(|(name, id, configured)| {
        serde_json::json!({
            "name": name,
            "id": id,
            "status": if *configured { "ok" } else { "unconfigured" },
        })
    }).collect();

    Json(serde_json::json!({"ok": true, "data": data}))
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

/// GET /internal/admin/system/logs?service=fetchium-api&lines=200
pub async fn system_logs(
    _auth: AdminAuth,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let service = params.get("service").map(|s| s.as_str()).unwrap_or("fetchium-api");
    let lines = params.get("lines").and_then(|n| n.parse::<u32>().ok()).unwrap_or(100).min(500);

    // Validate service name (whitelist only)
    let allowed = ["fetchium-api", "fetchium-admin", "fetchium-mcp"];
    if !allowed.contains(&service) {
        return Json(serde_json::json!({"ok": false, "lines": [], "error": "unknown service"}));
    }

    let output = std::process::Command::new("journalctl")
        .args(["-u", service, "--no-pager", "-n", &lines.to_string(), "--output=short-iso"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            let log_lines: Vec<serde_json::Value> = text.lines().map(|line| {
                // Detect log level from content
                let level = if line.contains("ERROR") || line.contains("error") { "ERROR" }
                    else if line.contains("WARN") || line.contains("warn") { "WARN" }
                    else if line.contains("DEBUG") || line.contains("debug") { "DEBUG" }
                    else { "INFO" };
                serde_json::json!({"line": line, "level": level})
            }).collect();
            Json(serde_json::json!({"ok": true, "lines": log_lines, "service": service}))
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            Json(serde_json::json!({"ok": false, "lines": [], "error": stderr.trim()}))
        }
        Err(e) => Json(serde_json::json!({"ok": false, "lines": [], "error": e.to_string()})),
    }
}

/// GET /internal/admin/system/jobs - recent audit events as job history
pub async fn system_jobs(
    auth: AdminAuth,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let rows = state.admin_db.as_ref()
        .and_then(|db| db.list_audit(50, 0).ok())
        .unwrap_or_default();

    Json(serde_json::json!({
        "ok": true,
        "jobs": rows,
        "total": rows.len(),
    }))
}
