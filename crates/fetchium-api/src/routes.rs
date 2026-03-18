//! REST API route definitions — v1 API under /v1/ prefix.

use crate::admin;
use crate::middleware::AppState;
use crate::{handlers, handlers_auth, handlers_proxy};
use axum::{
    routing::{delete, get, patch, post},
    Router,
};

/// Build the axum router with all v1 API routes.
pub fn build_router(state: AppState) -> Router {
    // Authenticated endpoints (require Bearer fetchium_xxx)
    let v1_authed = Router::new()
        .route("/search", post(handlers::search))
        .route("/scrape", post(handlers::scrape))
        .route("/fetch", post(handlers::fetch))
        .route("/research", post(handlers::research))
        .route("/youtube/search", post(handlers::youtube_search))
        .route("/youtube/analyze", post(handlers::youtube_analyze))
        .route("/social/research", post(handlers::social_research))
        .route(
            "/social/research/jobs",
            post(handlers::submit_social_research_job),
        )
        .route("/social/reddit", post(handlers::reddit_search))
        .route(
            "/social/reddit/jobs",
            post(handlers::submit_reddit_search_job),
        )
        .route("/social/hackernews", post(handlers::hackernews_search))
        .route(
            "/social/hackernews/jobs",
            post(handlers::submit_hackernews_search_job),
        )
        .route("/estimate", post(handlers::estimate))
        .route("/research/jobs", post(handlers::submit_research_job))
        .route(
            "/youtube/search/jobs",
            post(handlers::submit_youtube_search_job),
        )
        .route(
            "/youtube/analyze/jobs",
            post(handlers::submit_youtube_analyze_job),
        )
        .route("/jobs/:id", get(handlers::get_job_status))
        .route("/dashboard/billing", get(handlers_auth::get_dashboard_billing))
        .route("/dashboard/overview", get(handlers_auth::get_dashboard_overview))
        .route("/dashboard/quickstart", get(handlers_auth::get_dashboard_quickstart))
        .route("/dashboard/settings", get(handlers_auth::get_dashboard_settings).patch(handlers_auth::update_dashboard_settings))
        .route("/dashboard/usage", get(handlers_auth::get_dashboard_usage))
        .route("/usage", get(handlers_auth::get_usage))
        .route("/meta/routes", get(handlers_auth::get_route_registry));

    // Admin endpoints (require X-Admin-Secret header, for MVP)
    let v1_admin = Router::new()
        .route("/keys", post(handlers_auth::create_key))
        .route("/keys", get(handlers_auth::list_keys))
        .route("/keys/:id", delete(handlers_auth::revoke_key))
        // Proxy management
        .route("/proxy/stats", get(handlers_proxy::proxy_stats))
        .route("/proxy/reset", post(handlers_proxy::proxy_reset))
        .route("/proxy/purge", post(handlers_proxy::proxy_purge))
        .route("/proxy/test", post(handlers_proxy::proxy_test));

    // Internal admin routes — session-authenticated, staff only
    let internal_admin = Router::new()
        // Auth
        .route("/auth/bootstrap", post(admin::auth::bootstrap))
        .route("/auth/login", post(admin::auth::login))
        .route("/auth/logout", post(admin::auth::logout))
        .route("/auth/me", get(admin::auth::me))
        .route("/auth/totp/setup", post(admin::auth::totp_setup))
        .route("/auth/totp/confirm", post(admin::auth::totp_confirm))
        // Sessions
        .route("/sessions", get(admin::auth::list_sessions))
        .route("/sessions/:id", delete(admin::auth::revoke_session))
        // Orgs
        .route("/orgs", get(admin::orgs::list).post(admin::orgs::create))
        .route(
            "/orgs/:id",
            get(admin::orgs::get).patch(admin::orgs::update),
        )
        .route("/orgs/:id/suspend", post(admin::orgs::suspend))
        .route("/orgs/:id/reactivate", post(admin::orgs::reactivate))
        .route("/orgs/:id/plan", patch(admin::orgs::change_plan))
        .route("/orgs/:id/quota", patch(admin::orgs::override_quota))
        .route("/orgs/:id/keys", get(admin::keys::list_for_org))
        .route("/orgs/:id/usage", get(admin::usage::for_org))
        .route("/orgs/:id/billing", get(admin::billing::for_org))
        .route("/orgs/:id/tickets", get(admin::support::for_org))
        .route("/orgs/:id/members", get(admin::orgs::members))
        .route("/orgs/:id/audit", get(admin::orgs::org_audit))
        .route(
            "/orgs/:id/crm",
            get(admin::crm::get).patch(admin::crm::update),
        )
        // Users
        .route("/users", get(admin::users::list))
        .route("/users/:id", get(admin::users::get))
        .route("/users/:id/suspend", post(admin::users::suspend))
        .route("/users/:id/force-logout", post(admin::users::force_logout))
        // Keys
        .route("/keys", get(admin::keys::list).post(admin::keys::create))
        .route(
            "/keys/:id",
            get(admin::keys::get).delete(admin::keys::revoke),
        )
        .route("/keys/:id/rotate", post(admin::keys::rotate))
        // Usage
        .route("/usage", get(admin::usage::summary))
        .route("/usage/forensics/:request_id", get(admin::usage::forensics))
        .route("/usage/top-orgs", get(admin::usage::top_orgs))
        .route("/usage/heatmap", get(admin::usage::endpoint_heatmap))
        // Billing — static paths before dynamic :org_id
        .route("/billing", get(admin::billing::list_subscriptions))
        .route("/billing/webhooks", get(admin::billing::webhook_log))
        .route(
            "/billing/webhooks/:id/replay",
            post(admin::billing::webhook_replay),
        )
        .route("/billing/:org_id", get(admin::billing::for_org))
        .route("/billing/:org_id/credits", get(admin::billing::credits))
        .route("/billing/:org_id/refund", post(admin::billing::refund))
        .route("/billing/:org_id/credit", post(admin::billing::credit))
        .route("/billing/:org_id/invoices", get(admin::billing::invoices))
        // CRM
        .route("/crm/accounts", get(admin::crm::list))
        .route(
            "/crm/accounts/:org_id",
            get(admin::crm::get).patch(admin::crm::update),
        )
        .route("/crm/accounts/:org_id/notes", post(admin::crm::add_note))
        // Support
        .route("/support/tickets", get(admin::support::list).post(admin::support::create))
        .route("/support/tickets/:id", get(admin::support::get))
        .route("/support/tickets/:id/notes", post(admin::support::add_note))
        .route("/support/tickets/:id/assign", patch(admin::support::assign))
        .route(
            "/support/tickets/:id/status",
            patch(admin::support::set_status),
        )
        .route(
            "/support/macros",
            get(admin::support::list_macros).post(admin::support::create_macro),
        )
        // Incidents
        .route(
            "/incidents",
            get(admin::incidents::list).post(admin::incidents::create),
        )
        .route(
            "/incidents/:id",
            get(admin::incidents::get).patch(admin::incidents::update),
        )
        .route(
            "/incidents/:id/timeline",
            post(admin::incidents::add_timeline),
        )
        .route("/incidents/:id/resolve", post(admin::incidents::resolve))
        .route(
            "/incidents/:id/postmortem",
            post(admin::incidents::postmortem),
        )
        // Campaigns — static paths before dynamic :id
        .route(
            "/campaigns",
            get(admin::campaigns::list).post(admin::campaigns::create),
        )
        .route(
            "/campaigns/attribution",
            get(admin::campaigns::attribution_report),
        )
        .route("/campaigns/funnel", get(admin::campaigns::funnel))
        .route("/campaigns/:id", get(admin::campaigns::get))
        // Audit
        .route("/audit", get(admin::audit::list))
        .route("/audit/:id", get(admin::audit::get))
        // Flags
        .route("/flags", get(admin::flags::list).post(admin::flags::create))
        .route(
            "/flags/:id",
            get(admin::flags::get).patch(admin::flags::update),
        )
        // Metrics
        .route("/metrics/realtime", get(admin::metrics::realtime))
        .route("/metrics/summary", get(admin::metrics::summary))
        .route("/metrics/providers", get(admin::metrics::provider_health))
        // System
        .route("/system/stats", get(admin::metrics::system_stats))
        .route("/system/logs", get(admin::metrics::system_logs))
        .route("/system/jobs", get(admin::metrics::system_jobs))
        .route("/cache/clear", post(admin::metrics::cache_clear))
        .route("/db/query", post(admin::db_query::run_query))
        // Universal search
        .route("/search", get(admin::search::search))
        // Proxy ops
        .route("/proxy/stats", get(admin::proxy_ops::stats))
        .route("/proxy/reset", post(admin::proxy_ops::reset))
        .route("/proxy/purge", post(admin::proxy_ops::purge))
        .route("/proxy/geo", get(admin::proxy_ops::geo_distribution))
        // Anomaly
        .route("/anomaly/alerts", get(admin::anomaly::alerts))
        .route("/anomaly/tenants", get(admin::anomaly::suspicious_tenants))
        // Export
        .route("/export/:entity", get(admin::export::export_csv))
        // Staff management
        .route(
            "/staff",
            get(admin::auth::list_staff).post(admin::auth::create_staff),
        )
        .route(
            "/staff/:id",
            patch(admin::auth::update_staff).delete(admin::auth::remove_staff),
        )
        .route(
            "/staff/:id/sessions",
            get(admin::auth::staff_sessions).delete(admin::auth::revoke_all_sessions),
        )
        // Approval workflows
        .route(
            "/approvals",
            get(admin::approval::list).post(admin::approval::create),
        )
        .route("/approvals/:id/approve", post(admin::approval::approve))
        .route("/approvals/:id/reject", post(admin::approval::reject));

    Router::new()
        // Public endpoints (no auth)
        .route("/health", get(handlers_auth::health))
        .route("/v1/health", get(handlers_auth::health))
        .route("/", get(api_root))
        // Versioned public/customer API
        .nest("/v1", v1_authed)
        .nest("/v1", v1_admin)
        // Internal admin namespace (session auth, not X-Admin-Secret)
        .nest("/internal/admin", internal_admin)
        // Legacy unversioned routes (backwards compat)
        .route("/api/health", get(handlers_auth::health))
        .with_state(state)
}

async fn api_root() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "name": "Fetchium API",
        "version": env!("CARGO_PKG_VERSION"),
        "docs": "https://docs.fetchium.com",
        "endpoints": {
            "search":   "POST /v1/search",
            "scrape":   "POST /v1/scrape",
            "research": "POST /v1/research",
            "research_jobs": "POST /v1/research/jobs",
            "job_status": "GET /v1/jobs/:id",
            "usage":    "GET  /v1/usage",
            "health":   "GET  /v1/health",
            "keys":     "POST /v1/keys  (X-Admin-Secret required)",
        }
    }))
}
