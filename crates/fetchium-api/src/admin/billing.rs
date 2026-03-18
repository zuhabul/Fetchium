//! Admin billing and subscription handlers.

use crate::admin::rbac::{require, AdminAuth, Permission};
use crate::middleware::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreditBody {
    pub amount_cents: i64,
    pub reason: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct RefundBody {
    pub amount_cents: Option<i64>,
    pub reason: Option<String>,
}

pub async fn list_subscriptions(
    auth: AdminAuth,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::BillingRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    let data = db.list_subscriptions(50, 0).unwrap_or_default();
    let total = data.len() as i64;
    Ok(Json(serde_json::json!({"data": data, "total": total})))
}

pub async fn for_org(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::BillingRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    let subscription = db.get_subscription_for_org(&org_id).unwrap_or(None);
    let invoices = db.list_invoices_for_org(&org_id).unwrap_or_default();
    let credits = db.list_credits_for_org(&org_id).unwrap_or_default();
    Ok(Json(serde_json::json!({
        "subscription": subscription,
        "invoices": invoices,
        "credits": credits,
    })))
}

pub async fn refund(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    body: Option<Json<RefundBody>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::BillingRefund)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    let body = body.map(|Json(b)| b).unwrap_or_default();
    let refund_id = db
        .create_pending_refund(
            &org_id,
            body.amount_cents,
            body.reason.as_deref(),
            Some(&auth.user.id),
        )
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;
    let _ = db.log_audit(
        Some(&auth.user.id),
        Some(&auth.user.role),
        "org",
        Some(&org_id),
        "billing.refund",
        None,
    );
    Ok(Json(serde_json::json!({
        "status": "queued",
        "refund_id": refund_id,
        "message": "Refund queued for manual processing by billing team"
    })))
}

pub async fn credit(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    Json(body): Json<CreditBody>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::BillingCredit)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    db.add_credit(
        &org_id,
        body.amount_cents,
        body.reason.as_deref(),
        Some(&auth.user.id),
    )
    .map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;
    let _ = db.log_audit(
        Some(&auth.user.id),
        Some(&auth.user.role),
        "org",
        Some(&org_id),
        "billing.credit",
        None,
    );
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn invoices(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::BillingRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    let data = db.list_invoices_for_org(&org_id).unwrap_or_default();
    Ok(Json(serde_json::json!({"data": data})))
}

pub async fn credits(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::BillingRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    let data = db.list_credits_for_org(&org_id).unwrap_or_default();
    Ok(Json(serde_json::json!({"data": data})))
}

pub async fn webhook_log(
    auth: AdminAuth,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::BillingRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    let data = db.list_payment_events(100).unwrap_or_default();
    Ok(Json(serde_json::json!(data)))
}

pub async fn webhook_replay(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::BillingRefund)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    // Fetch original event — 404 if it doesn't exist
    let event = db
        .get_payment_event(&id)
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?
        .ok_or_else(|| {
            (
                axum::http::StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "webhook event not found"})),
            )
        })?;
    // Stamp replayed_at
    db.mark_payment_event_replayed(&id).map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;
    let _ = db.log_audit(
        Some(&auth.user.id),
        Some(&auth.user.role),
        "billing",
        Some(&id),
        "webhook.replay",
        None,
    );
    Ok(Json(serde_json::json!({
        "ok": true,
        "replayed": event,
        "message": "Webhook event replayed and timestamped"
    })))
}
