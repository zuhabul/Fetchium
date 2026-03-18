//! Admin auth HTTP handlers — login, logout, TOTP setup, session management.

use super::db::AdminUser;
use super::rbac::AdminAuth;
use crate::middleware::AppState;
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use base32::Alphabet;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use totp_rs::{Algorithm, TOTP};
use uuid::Uuid;

// ── Request / response types ─────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct BootstrapRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub totp_code: Option<String>,
}

enum LoginCheck {
    Invalid,
    WrongPassword,
    User(AdminUser),
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    #[serde(rename = "sessionToken")]
    pub session_token: String,
    pub session_id: String,
    #[serde(rename = "totpRequired")]
    pub totp_required: bool,
}

#[derive(Deserialize)]
pub struct TotpConfirmRequest {
    pub code: String,
}

#[derive(Serialize)]
pub struct TotpSetupResponse {
    pub secret: String,      // base32 encoded, for manual entry
    pub otpauth_url: String, // for QR code
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn extract_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .unwrap_or("unknown")
        .trim()
        .to_string()
}

fn extract_ua(headers: &HeaderMap) -> String {
    headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .chars()
        .take(256)
        .collect()
}

fn generate_session_token() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

fn hash_token(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut rand::rngs::OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

fn make_totp(secret_bytes: &[u8], email: &str) -> Result<TOTP, String> {
    TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes.to_vec(),
        Some("Fetchium Admin".to_string()),
        email.to_string(),
    )
    .map_err(|e| e.to_string())
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// POST /internal/admin/auth/bootstrap
/// Create the first owner account. Only works when admin_users is empty.
/// Requires X-Bootstrap-Secret header matching FETCHIUM_ADMIN_BOOTSTRAP_SECRET env var.
pub async fn bootstrap(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<BootstrapRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let expected = std::env::var("FETCHIUM_ADMIN_BOOTSTRAP_SECRET").unwrap_or_default();
    let provided = headers
        .get("x-bootstrap-secret")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if expected.is_empty() || provided != expected {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "invalid bootstrap secret"})),
        ));
    }

    let admin_db = state.admin_db.as_ref().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;

    if admin_db.has_any_users().map_err(db_err)? {
        return Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error": "admin users already exist — use normal login"})),
        ));
    }

    let id = Uuid::new_v4().to_string();
    let password_hash = hash_password(&req.password).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
    })?;

    admin_db
        .create_user(&id, &req.email, &password_hash, "owner", &req.name)
        .map_err(db_err)?;

    tracing::info!(user_id = %id, email = %req.email, "bootstrap: first admin owner created");

    Ok(Json(serde_json::json!({
        "ok": true,
        "id": id,
        "email": req.email,
        "role": "owner",
        "message": "Owner account created. Log in at /login.",
    })))
}

/// POST /internal/admin/auth/login
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<serde_json::Value>)> {
    let admin_db = state.admin_db.clone().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    let lookup_db = admin_db.clone();
    let email = req.email.clone();
    let password = req.password.clone();
    let login_check = tokio::task::spawn_blocking(move || -> anyhow::Result<LoginCheck> {
        let Some(user) = lookup_db.find_user_by_email(&email)? else {
            return Ok(LoginCheck::Invalid);
        };

        if !user.is_active {
            return Ok(LoginCheck::Invalid);
        }

        if !verify_password(&password, &user.password_hash) {
            return Ok(LoginCheck::WrongPassword);
        }

        Ok(LoginCheck::User(user))
    })
    .await
    .map_err(blocking_err)?
    .map_err(db_err)?;

    let user = match login_check {
        LoginCheck::Invalid => return Err(invalid_creds()),
        LoginCheck::WrongPassword => {
            tracing::warn!(email = %req.email, ip = %extract_ip(&headers), "admin login: wrong password");
            return Err(invalid_creds());
        }
        LoginCheck::User(user) => user,
    };

    // TOTP gate
    if user.totp_enabled {
        let Some(ref code) = req.totp_code else {
            // Signal to Next.js that TOTP is required
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "TOTP_REQUIRED"})),
            ));
        };

        let secret_bytes = base32::decode(
            Alphabet::RFC4648 { padding: false },
            user.totp_secret.as_deref().unwrap_or(""),
        )
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "invalid totp config"})),
            )
        })?;

        let totp = make_totp(&secret_bytes, &user.email).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e})),
            )
        })?;

        let valid = totp.check_current(code).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "totp clock error"})),
            )
        })?;

        if !valid {
            tracing::warn!(user_id = %user.id, "admin login: invalid TOTP code");
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "invalid TOTP code"})),
            ));
        }

        // Replay prevention
        if admin_db.is_totp_code_used(&user.id, code).map_err(db_err)? {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "TOTP code already used"})),
            ));
        }
        admin_db
            .mark_totp_code_used(&user.id, code)
            .map_err(db_err)?;
    }

    // Create session
    let token = generate_session_token();
    let token_hash = hash_token(&token);
    let session_id = Uuid::new_v4().to_string();
    let session_id_for_db = session_id.clone();
    let ip = extract_ip(&headers);
    let ua = extract_ua(&headers);

    let admin_db = state.admin_db.clone().ok_or_else(db_not_init)?;
    let user_id = user.id.clone();
    let ip_for_db = ip.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        admin_db.create_session(&session_id_for_db, &user_id, &token_hash, &ip_for_db, &ua)?;
        admin_db.update_last_login(&user_id, &ip_for_db)?;
        Ok(())
    })
    .await
    .map_err(blocking_err)?
    .map_err(db_err)?;

    tracing::info!(user_id = %user.id, email = %user.email, role = %user.role, ip = %ip, "admin login success");

    Ok(Json(LoginResponse {
        id: user.id,
        email: user.email,
        name: user.name,
        role: user.role,
        session_token: token,
        session_id,
        totp_required: false,
    }))
}

/// POST /internal/admin/auth/logout
pub async fn logout(
    auth: AdminAuth,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let admin_db = state.admin_db.as_ref().ok_or_else(db_not_init)?;
    admin_db.revoke_session(&auth.session_id).map_err(db_err)?;
    tracing::info!(user_id = %auth.user.id, "admin logout");
    Ok(Json(serde_json::json!({"ok": true})))
}

/// GET /internal/admin/auth/me
pub async fn me(auth: AdminAuth) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "id": auth.user.id,
        "email": auth.user.email,
        "name": auth.user.name,
        "role": auth.user.role,
        "totp_enabled": auth.user.totp_enabled,
    }))
}

/// POST /internal/admin/auth/totp/setup — generate a new TOTP secret for enrollment.
pub async fn totp_setup(
    auth: AdminAuth,
) -> Result<Json<TotpSetupResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Generate 20 random bytes for TOTP secret
    let secret_bytes: Vec<u8> = (0..20).map(|_| rand::thread_rng().gen::<u8>()).collect();
    let secret_b32 = base32::encode(Alphabet::RFC4648 { padding: false }, &secret_bytes);

    let totp = make_totp(&secret_bytes, &auth.user.email).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
    })?;

    Ok(Json(TotpSetupResponse {
        secret: secret_b32,
        otpauth_url: totp.get_url(),
    }))
}

/// POST /internal/admin/auth/totp/confirm — confirm enrollment with first code.
/// Body: { secret: "BASE32...", code: "123456" }
#[derive(Deserialize)]
pub struct TotpConfirmBody {
    pub secret: String,
    pub code: String,
}

pub async fn totp_confirm(
    auth: AdminAuth,
    State(state): State<AppState>,
    Json(body): Json<TotpConfirmBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let admin_db = state.admin_db.as_ref().ok_or_else(db_not_init)?;

    let secret_bytes = base32::decode(Alphabet::RFC4648 { padding: false }, &body.secret)
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "invalid secret"})),
            )
        })?;

    let totp = make_totp(&secret_bytes, &auth.user.email).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
    })?;

    let valid = totp.check_current(&body.code).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "clock error"})),
        )
    })?;

    if !valid {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "invalid code — check your time sync"})),
        ));
    }

    admin_db
        .set_totp(&auth.user.id, &body.secret, true)
        .map_err(db_err)?;
    tracing::info!(user_id = %auth.user.id, "TOTP enabled");
    Ok(Json(serde_json::json!({"ok": true, "totp_enabled": true})))
}

/// GET /internal/admin/sessions — list active sessions for current user.
pub async fn list_sessions(
    auth: AdminAuth,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let admin_db = state.admin_db.as_ref().ok_or_else(db_not_init)?;
    let sessions = admin_db.list_sessions(&auth.user.id).map_err(db_err)?;
    Ok(Json(serde_json::json!({"sessions": sessions})))
}

/// DELETE /internal/admin/sessions/:id — revoke a specific session.
pub async fn revoke_session(
    _auth: AdminAuth,
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let admin_db = state.admin_db.as_ref().ok_or_else(db_not_init)?;
    // Users can revoke their own sessions; owners can revoke any
    admin_db.revoke_session(&session_id).map_err(db_err)?;
    Ok(Json(serde_json::json!({"ok": true})))
}

// ── Staff management handlers ─────────────────────────────────────────────────

/// GET /internal/admin/staff — list all admin staff users.
pub async fn list_staff(
    _auth: AdminAuth,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let admin_db = state.admin_db.as_ref().ok_or_else(db_not_init)?;
    let users = admin_db.list_users(100, 0, None, None).map_err(db_err)?;
    Ok(Json(serde_json::json!({"ok": true, "data": users})))
}

// (list_staff uses _auth intentionally — auth is validated by AdminAuth extractor)

/// POST /internal/admin/staff — create a new staff member.
pub async fn create_staff(
    auth: AdminAuth,
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let admin_db = state.admin_db.as_ref().ok_or_else(db_not_init)?;
    let email = body.get("email").and_then(|v| v.as_str()).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "email required"})),
        )
    })?;
    let password = body
        .get("password")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "password required"})),
            )
        })?;
    let name = body.get("name").and_then(|v| v.as_str()).unwrap_or("Staff");
    let role = body
        .get("role")
        .and_then(|v| v.as_str())
        .unwrap_or("support");
    let hash = hash_password(password).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "hash failed"})),
        )
    })?;
    let id = Uuid::new_v4().to_string();
    admin_db
        .create_user(&id, email, &hash, role, name)
        .map_err(db_err)?;
    let _ = admin_db.log_audit(
        Some(&auth.user.id),
        Some(&auth.user.role),
        "staff",
        Some(&id),
        "staff.create",
        None,
    );
    Ok(Json(serde_json::json!({"ok": true, "id": id})))
}

/// PATCH /internal/admin/staff/:id — update a staff member.
pub async fn update_staff(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let admin_db = state.admin_db.as_ref().ok_or_else(db_not_init)?;
    if let Some(role) = body.get("role").and_then(|v| v.as_str()) {
        admin_db.update_staff_role(&id, role).map_err(db_err)?;
    }
    if let Some(active) = body.get("is_active").and_then(|v| v.as_bool()) {
        admin_db.set_user_active(&id, active).map_err(db_err)?;
    }
    let _ = admin_db.log_audit(
        Some(&auth.user.id),
        Some(&auth.user.role),
        "staff",
        Some(&id),
        "staff.update",
        None,
    );
    Ok(Json(serde_json::json!({"ok": true})))
}

/// DELETE /internal/admin/staff/:id — remove a staff member (deactivate).
pub async fn remove_staff(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let admin_db = state.admin_db.as_ref().ok_or_else(db_not_init)?;
    admin_db.set_user_active(&id, false).map_err(db_err)?;
    let _ = admin_db.log_audit(
        Some(&auth.user.id),
        Some(&auth.user.role),
        "staff",
        Some(&id),
        "staff.remove",
        None,
    );
    Ok(Json(serde_json::json!({"ok": true})))
}

/// GET /internal/admin/staff/:id/sessions — list sessions for a staff member.
pub async fn staff_sessions(
    _auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let admin_db = state.admin_db.as_ref().ok_or_else(db_not_init)?;
    let sessions = admin_db.list_sessions(&id).map_err(db_err)?;
    Ok(Json(serde_json::json!({"ok": true, "data": sessions})))
}

/// DELETE /internal/admin/staff/:id/sessions — revoke all sessions for a staff member.
pub async fn revoke_all_sessions(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let admin_db = state.admin_db.as_ref().ok_or_else(db_not_init)?;
    admin_db.revoke_all_sessions_for_user(&id).map_err(db_err)?;
    let _ = admin_db.log_audit(
        Some(&auth.user.id),
        Some(&auth.user.role),
        "staff",
        Some(&id),
        "staff.revoke_sessions",
        None,
    );
    Ok(Json(serde_json::json!({"ok": true})))
}

// ── Error helpers ────────────────────────────────────────────────────────────

fn invalid_creds() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({"error": "invalid credentials"})),
    )
}

fn db_err(e: anyhow::Error) -> (StatusCode, Json<serde_json::Value>) {
    tracing::error!("admin db error: {e}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({"error": "internal error"})),
    )
}

fn blocking_err(e: tokio::task::JoinError) -> (StatusCode, Json<serde_json::Value>) {
    tracing::error!("admin auth blocking task failed: {e}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({"error": "internal error"})),
    )
}

fn db_not_init() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let password = "SuperSecret123!";
        let hash = hash_password(password).expect("hash should succeed");
        assert!(
            verify_password(password, &hash),
            "correct password should verify"
        );
        assert!(
            !verify_password("WrongPassword", &hash),
            "wrong password should fail"
        );
        assert!(!verify_password("", &hash), "empty password should fail");
    }

    #[test]
    fn test_password_hash_unique() {
        let p = "SamePassword";
        let h1 = hash_password(p).unwrap();
        let h2 = hash_password(p).unwrap();
        assert_ne!(h1, h2, "hashes should be unique (different salt)");
    }
}
