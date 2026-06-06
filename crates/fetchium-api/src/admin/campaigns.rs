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
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
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
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    // Fetch touches for this campaign
    let touches = db.run_select_query_params1(
        "SELECT id, touch_type, org_id, occurred_at FROM attribution_touches WHERE campaign_id = ?1 ORDER BY occurred_at DESC LIMIT 100",
        &id,
        100,
    ).unwrap_or_else(|_| crate::admin::db::QueryResult { columns: vec![], rows: vec![] });
    Ok(Json(
        serde_json::json!({"id": id, "touches": {"columns": touches.columns, "rows": touches.rows}}),
    ))
}

pub async fn create(
    auth: AdminAuth,
    State(state): State<AppState>,
    Json(body): Json<CreateCampaign>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::CrmWrite)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    let id = Uuid::new_v4().to_string();
    db.create_campaign(
        &id,
        &body.name,
        body.campaign_type.as_deref().unwrap_or("email"),
        &auth.user.id,
    )
    .map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;
    Ok(Json(serde_json::json!({"ok": true, "id": id})))
}

pub async fn attribution_report(
    auth: AdminAuth,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::CampaignsRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    let data = db
        .run_select_query(
            "SELECT c.id, c.name, c.type, c.status,
                COUNT(at.id) as touch_count,
                COUNT(DISTINCT at.org_id) as unique_orgs,
                MIN(at.occurred_at) as first_touch,
                MAX(at.occurred_at) as last_touch
         FROM campaigns c
         LEFT JOIN attribution_touches at ON at.campaign_id = c.id
         GROUP BY c.id ORDER BY touch_count DESC LIMIT 100",
            100,
        )
        .map(|r| {
            r.rows
                .iter()
                .map(|row| {
                    serde_json::json!({
                        "campaign_id": row.first(), "campaign_name": row.get(1),
                        "type": row.get(2), "status": row.get(3),
                        "touch_count": row.get(4), "unique_orgs": row.get(5),
                        "first_touch": row.get(6), "last_touch": row.get(7),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Ok(Json(serde_json::json!({"data": data, "total": data.len()})))
}

pub async fn funnel(
    auth: AdminAuth,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::CampaignsRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    // Funnel: draft → active → touch → conversion (org created after touch)
    let stages = db.run_select_query(
        "SELECT
           (SELECT COUNT(*) FROM campaigns) as total_campaigns,
           (SELECT COUNT(*) FROM campaigns WHERE status='active') as active_campaigns,
           (SELECT COUNT(*) FROM attribution_touches) as total_touches,
           (SELECT COUNT(DISTINCT org_id) FROM attribution_touches WHERE org_id IS NOT NULL) as touched_orgs",
        1,
    ).map(|r| {
        r.rows.first().map(|row| serde_json::json!({
            "stages": [
                {"name": "Campaigns Created", "count": row.first()},
                {"name": "Active Campaigns", "count": row.get(1)},
                {"name": "Attribution Touches", "count": row.get(2)},
                {"name": "Touched Orgs", "count": row.get(3)},
            ]
        }))
    }).unwrap_or(None);
    Ok(Json(serde_json::json!({"data": stages})))
}
