//! Admin CSV export handler.

use crate::admin::rbac::AdminAuth;
use crate::middleware::AppState;
use axum::{
    extract::{Path, State},
    Json,
};

pub async fn export_csv(
    _auth: AdminAuth,
    State(_state): State<AppState>,
    Path(_entity): Path<String>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": null, "message": "export not yet implemented"}))
}
