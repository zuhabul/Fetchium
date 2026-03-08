//! Read-only SQL query runner for admin DB inspection.

use axum::{extract::State, Json};
use axum::http::StatusCode;
use serde::Deserialize;
use crate::admin::rbac::AdminAuth;
use crate::middleware::AppState;

#[derive(Deserialize)]
pub struct QueryRequest {
    pub sql: String,
    pub db: Option<String>, // "admin" (default) | future: "auth"
}

/// POST /internal/admin/db/query
/// Executes a read-only SQL query against admin.db.
/// Only SELECT statements allowed. LIMIT enforced at 1000 rows.
pub async fn run_query(
    auth: AdminAuth,
    State(state): State<AppState>,
    Json(req): Json<QueryRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Validate it's a SELECT
    let sql = req.sql.trim().to_lowercase();
    if !sql.starts_with("select") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Only SELECT statements are permitted"})),
        ));
    }
    // Disallow dangerous keywords even in SELECT
    for forbidden in &["insert", "update", "delete", "drop", "alter", "create", "pragma write"] {
        if sql.contains(forbidden) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("Forbidden keyword: {forbidden}")})),
            ));
        }
    }

    let admin_db = state.admin_db.as_ref().ok_or_else(|| (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;

    let result = admin_db.run_select_query(&req.sql, 1000)
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        ))?;

    Ok(Json(serde_json::json!({
        "ok": true,
        "columns": result.columns,
        "rows": result.rows,
        "row_count": result.rows.len(),
        "executed_by": auth.user.email,
    })))
}
