//! Admin campaign and attribution handlers.

use crate::admin::rbac::{require, AdminAuth, Permission};
use crate::middleware::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ListParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreateCampaign {
    pub name: String,
    #[serde(rename = "type")]
    pub campaign_type: Option<String>,
}

pub async fn list(
    auth: AdminAuth,
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::CampaignsRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let limit = p.limit.unwrap_or(50).min(200);
    let offset = p.offset.unwrap_or(0);
    let data = db.list_campaigns(limit, offset).unwrap_or_default();
    let total = db.count_campaigns().unwrap_or(0);
    Ok(Json(serde_json::json!({"data": data, "total": total})))
}

pub async fn get(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::CampaignsRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    // Fetch touches for this campaign
    let touches = db.run_select_query(
        &format!("SELECT id, touch_type, org_id, occurred_at FROM attribution_touches WHERE campaign_id = '{id}' ORDER BY occurred_at DESC LIMIT 100"),
        100,
    ).unwrap_or_else(|_| crate::admin::db::QueryResult { columns: vec![], rows: vec![] });
    Ok(Json(serde_json::json!({"id": id, "touches": {"columns": touches.columns, "rows": touches.rows}})))
}

pub async fn create(
    auth: AdminAuth,
    State(state): State<AppState>,
    Json(body): Json<CreateCampaign>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::CrmWrite)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let id = Uuid::new_v4().to_string();
    db.create_campaign(
        &id,
        &body.name,
        body.campaign_type.as_deref().unwrap_or("email"),
        &auth.user.id,
    ).map_err(|e| (
        axum::http::StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": e.to_string()})),
    ))?;
    Ok(Json(serde_json::json!({"ok": true, "id": id})))
}

pub async fn attribution_report(
    auth: AdminAuth,
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::CampaignsRead)?;
    Ok(Json(serde_json::json!({"data": []})))
}

pub async fn funnel(
    auth: AdminAuth,
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::CampaignsRead)?;
    Ok(Json(serde_json::json!({"data": []})))
}
