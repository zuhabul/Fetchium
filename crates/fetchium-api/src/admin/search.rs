//! Universal entity search across orgs, users, keys, tickets, incidents.

use crate::admin::rbac::AdminAuth;
use crate::middleware::AppState;
use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub limit: Option<usize>,
}

#[derive(serde::Serialize)]
pub struct SearchResult {
    pub entity_type: String,
    pub id: String,
    pub display_name: String,
    pub subtitle: Option<String>,
    pub href: String,
}

/// GET /internal/admin/search?q=<query>&limit=10
pub async fn search(
    _auth: AdminAuth,
    Query(params): Query<SearchParams>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let limit = params.limit.unwrap_or(10).min(50);
    let q = params.q.trim().to_lowercase();

    if q.len() < 2 {
        return Json(serde_json::json!({"results": [], "total": 0}));
    }

    let admin_db = match state.admin_db.as_ref() {
        Some(db) => db,
        None => return Json(serde_json::json!({"results": [], "total": 0})),
    };

    let mut results: Vec<SearchResult> = Vec::new();

    // Search organizations
    if let Ok(orgs) = admin_db.search_orgs(&q, limit) {
        for org in orgs {
            results.push(SearchResult {
                entity_type: "org".to_string(),
                id: org
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                display_name: org
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                subtitle: org.get("plan").and_then(|v| v.as_str()).map(String::from),
                href: format!(
                    "/orgs/{}",
                    org.get("id").and_then(|v| v.as_str()).unwrap_or("")
                ),
            });
        }
    }

    // Search incidents
    if let Ok(incidents) = admin_db.search_incidents(&q, limit) {
        for inc in incidents {
            results.push(SearchResult {
                entity_type: "incident".to_string(),
                id: inc
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                display_name: inc
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                subtitle: inc
                    .get("severity")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                href: format!(
                    "/incidents/{}",
                    inc.get("id").and_then(|v| v.as_str()).unwrap_or("")
                ),
            });
        }
    }

    results.truncate(limit);
    let total = results.len();
    Json(serde_json::json!({"results": results, "total": total}))
}
