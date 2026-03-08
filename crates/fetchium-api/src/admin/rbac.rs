//! RBAC — permission atoms and role-based access control.
//! AdminAuth is the axum extractor that validates admin sessions.

use super::db::AdminUser;
use crate::middleware::AppState;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, State},
    http::{request::Parts, StatusCode},
    Json,
};
use sha2::{Digest, Sha256};

/// Permission atoms — every mutable admin action maps to one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    // Keys
    KeysRead,
    KeysRevoke,
    KeysCreate,
    // Orgs
    OrgsRead,
    OrgsSuspend,
    OrgsPlanChange,
    OrgsQuotaOverride,
    OrgsDelete,
    // Users
    UsersRead,
    UsersSuspend,
    // Billing
    BillingRead,
    BillingRefund,
    BillingCredit,
    // Support
    SupportRead,
    SupportReply,
    SupportClose,
    // CRM
    CrmRead,
    CrmWrite,
    // Incidents
    IncidentsRead,
    IncidentsManage,
    // Audit
    AuditRead,
    // Flags
    FlagsRead,
    FlagsWrite,
    // Proxy
    ProxyRead,
    ProxyReset,
    // Campaigns
    CampaignsRead,
    // Admin staff management
    AdminStaffManage,
}

impl Permission {
    pub fn name(self) -> &'static str {
        match self {
            Self::KeysRead => "keys.read",
            Self::KeysRevoke => "keys.revoke",
            Self::KeysCreate => "keys.create",
            Self::OrgsRead => "orgs.read",
            Self::OrgsSuspend => "orgs.suspend",
            Self::OrgsPlanChange => "orgs.plan_change",
            Self::OrgsQuotaOverride => "orgs.quota_override",
            Self::OrgsDelete => "orgs.delete",
            Self::UsersRead => "users.read",
            Self::UsersSuspend => "users.suspend",
            Self::BillingRead => "billing.read",
            Self::BillingRefund => "billing.refund",
            Self::BillingCredit => "billing.credit",
            Self::SupportRead => "support.read",
            Self::SupportReply => "support.reply",
            Self::SupportClose => "support.close",
            Self::CrmRead => "crm.read",
            Self::CrmWrite => "crm.write",
            Self::IncidentsRead => "incidents.read",
            Self::IncidentsManage => "incidents.manage",
            Self::AuditRead => "audit.read",
            Self::FlagsRead => "flags.read",
            Self::FlagsWrite => "flags.write",
            Self::ProxyRead => "proxy.read",
            Self::ProxyReset => "proxy.reset",
            Self::CampaignsRead => "campaigns.read",
            Self::AdminStaffManage => "admin.staff_manage",
        }
    }
}

/// Role-to-permission matrix. Returns true if `role` has `perm`.
pub fn has_permission(role: &str, perm: Permission) -> bool {
    match role {
        "owner" => true, // owner has all permissions
        "ops" => matches!(
            perm,
            Permission::KeysRead
                | Permission::KeysRevoke
                | Permission::KeysCreate
                | Permission::OrgsRead
                | Permission::OrgsSuspend
                | Permission::OrgsQuotaOverride
                | Permission::UsersRead
                | Permission::UsersSuspend
                | Permission::SupportRead
                | Permission::SupportReply
                | Permission::SupportClose
                | Permission::IncidentsRead
                | Permission::IncidentsManage
                | Permission::AuditRead
                | Permission::FlagsRead
                | Permission::FlagsWrite
                | Permission::ProxyRead
                | Permission::ProxyReset
        ),
        "support" => matches!(
            perm,
            Permission::KeysRead
                | Permission::KeysCreate
                | Permission::OrgsRead
                | Permission::UsersRead
                | Permission::SupportRead
                | Permission::SupportReply
                | Permission::SupportClose
                | Permission::CrmRead
                | Permission::CrmWrite
                | Permission::IncidentsRead
                | Permission::AuditRead
        ),
        "finance" => matches!(
            perm,
            Permission::KeysRead
                | Permission::OrgsRead
                | Permission::OrgsPlanChange
                | Permission::UsersRead
                | Permission::BillingRead
                | Permission::BillingRefund
                | Permission::BillingCredit
                | Permission::CrmRead
                | Permission::CampaignsRead
        ),
        "growth" => matches!(
            perm,
            Permission::OrgsRead
                | Permission::UsersRead
                | Permission::CrmRead
                | Permission::CrmWrite
                | Permission::CampaignsRead
        ),
        "readonly" => matches!(
            perm,
            Permission::KeysRead
                | Permission::OrgsRead
                | Permission::UsersRead
                | Permission::SupportRead
                | Permission::IncidentsRead
                | Permission::AuditRead
                | Permission::FlagsRead
                | Permission::CrmRead
                | Permission::CampaignsRead
        ),
        _ => false,
    }
}

/// Check permission, returning a 403 response if denied.
pub fn require(
    user: &AdminUser,
    perm: Permission,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    if has_permission(&user.role, perm) {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "insufficient permissions",
                "required": perm.name(),
                "your_role": user.role,
            })),
        ))
    }
}

/// Axum extractor: validates admin session from `Authorization: Bearer <token>`.
/// Injects the authenticated `AdminUser` and `session_id` into handlers.
pub struct AdminAuth {
    pub user: AdminUser,
    pub session_id: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AdminAuth
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let State(app_state) = State::<AppState>::from_request_parts(parts, state)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "state unavailable"})),
                )
            })?;

        let token = parts
            .headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({"error": "missing authorization header"})),
                )
            })?
            .to_string();

        let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));

        let admin_db = app_state.admin_db.as_ref().ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "admin not initialized"})),
            )
        })?;

        let (user, session_id) = admin_db
            .validate_session(&token_hash)
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "db error"})),
                )
            })?
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({"error": "invalid or expired session"})),
                )
            })?;

        // Refresh session TTL in background — don't block response on this
        let db = admin_db.clone();
        let sid = session_id.clone();
        tokio::spawn(async move {
            let _ = db.touch_session(&sid);
        });

        Ok(AdminAuth { user, session_id })
    }
}
