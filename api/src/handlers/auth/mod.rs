use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderValue, header},
};
use lucid_common::{
    caller::Caller,
    params::AuthLoginParams,
    views::{AuthLoginResponse, User},
};
use lucid_db::storage::{SessionStore, UserStore};
use rand::Rng;
use tracing::info;

use crate::{auth::Auth, context::ApiContext, error::ApiError};

/// Authenticate user and create session.
///
/// This endpoint validates user credentials and creates a new session stored in the database.
/// On success, it returns a session cookie and a CSRF token.
///
/// # Flow
///
/// 1. Validates username/password against database
/// 2. Generates unique session_id (ULID) and csrf_token (32 random chars)
/// 3. Creates session in database with 30-day TTL
/// 4. Signs session_id with Ed25519 key
/// 5. Returns signed token in `lucid_session` cookie + CSRF token in response body
///
/// # Cookie Format
///
/// - Name: `lucid_session`
/// - Value: `{session_id}.{ed25519_signature}`
/// - Flags: HttpOnly, SameSite=Lax, Path=/, Max-Age=2592000 (30 days)
/// - Secure: Only set when `public_url` starts with https://
///
/// # CSRF Token
///
/// The CSRF token must be stored by the client (e.g., in memory or localStorage) and sent
/// in the `X-CSRF-Token` header for all state-changing requests (POST, PUT, DELETE).
///
/// # Example
///
/// ```bash
/// curl -X POST http://localhost:3000/v1/auth/login \
///   -H "Content-Type: application/json" \
///   -d '{"username": "admin", "password": "secret"}' \
///   -c cookies.txt
/// ```
///
/// # Errors
///
/// - 401 Unauthorized: Invalid username or password
/// - 500 Internal Server Error: Database or signing failure
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tags = ["auth", "console_sessions"],
    request_body(content = AuthLoginParams, content_type = "application/json"),
    responses((status = 201, description = "Successful login", body = AuthLoginResponse))
)]
pub async fn auth_login(
    State(ctx): State<ApiContext>,
    Json(body): Json<AuthLoginParams>,
) -> Result<(HeaderMap, Json<AuthLoginResponse>), ApiError> {
    // 1. Authenticate user
    let caller = UserStore::auth_local(&*ctx.db, body.username, body.password).await?;

    // 2. Extract user_id from Caller
    let user_id = match &caller {
        Caller::User { id, .. } => mongodb::bson::oid::ObjectId::parse_str(id)
            .map_err(|e| anyhow::anyhow!("invalid user id: {}", e))?,
        _ => return Err(anyhow::anyhow!("expected user caller").into()),
    };

    // 3. Generate session_id and csrf_token
    let session_id = ulid::Ulid::new().to_string();
    let csrf_token: String = rand::rng()
        .sample_iter(rand::distr::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    // 4. Create session in DB (30 day TTL)
    SessionStore::create_session(
        &*ctx.db,
        user_id,
        session_id.clone(),
        csrf_token.clone(),
        chrono::Duration::days(30),
    )
    .await?;

    info!("Logged in user {}", caller.id());

    // 5. Sign the session_id
    let signed_token = ctx
        .session_signer
        .sign(&session_id)
        .map_err(|e| anyhow::anyhow!("failed to sign session: {}", e))?;

    // 6. Build cookie
    let secure_flag = if ctx._config.public_url.starts_with("https://") {
        "; Secure"
    } else {
        ""
    };
    let cookie = format!(
        "lucid_session={}; HttpOnly; SameSite=Lax; Path=/; Max-Age={}{}",
        signed_token,
        30 * 24 * 60 * 60, // 30 days in seconds
        secure_flag
    );

    // 7. Set cookie header
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie)
            .map_err(|e| anyhow::anyhow!("invalid cookie value: {}", e))?,
    );

    Ok((headers, Json(AuthLoginResponse::Session { csrf_token })))
}

/// End the current session.
///
/// This endpoint deletes the user's session from the database and clears the session cookie.
/// Requires both session cookie authentication AND the CSRF token.
///
/// # Flow
///
/// 1. Extracts and verifies session cookie from request
/// 2. Validates CSRF token (via Auth extractor)
/// 3. Deletes session from database
/// 4. Returns cookie with Max-Age=0 to clear it from browser
///
/// # Security
///
/// This is a state-changing operation, so it requires CSRF protection. The session cookie
/// alone is not sufficient - the CSRF token must also be provided.
///
/// # Example
///
/// ```bash
/// curl -X POST http://localhost:3000/v1/auth/logout \
///   -H "X-CSRF-Token: {csrf_token_from_login}" \
///   -b cookies.txt
/// ```
///
/// # Errors
///
/// - 401 Unauthorized: Missing or invalid session cookie
/// - 403 Forbidden: Invalid CSRF token
/// - 500 Internal Server Error: Database failure
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    tags = ["auth", "console_sessions"],
    responses((status = 200, description = "Successful logout"))
)]
pub async fn auth_logout(
    State(ctx): State<ApiContext>,
    Auth(caller): Auth,
    headers: HeaderMap,
) -> Result<(HeaderMap, &'static str), ApiError> {
    // 1. Extract session cookie
    let signed_cookie = headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies
                .split(';')
                .map(|s| s.trim())
                .find(|s| s.starts_with("lucid_session="))
                .and_then(|s| s.strip_prefix("lucid_session="))
        })
        .ok_or_else(|| anyhow::anyhow!("session cookie not found"))?;

    // 2. Verify and extract session_id
    let session_id = ctx
        .session_signer
        .verify(signed_cookie)
        .ok_or_else(|| anyhow::anyhow!("invalid session signature"))?;

    // 3. Delete session from DB
    SessionStore::delete_session(&*ctx.db, &session_id).await?;

    info!("Logged out user {}", caller.id());

    // 4. Clear cookie (must match login cookie flags, especially Secure)
    let secure_flag = if ctx._config.public_url.starts_with("https://") {
        "; Secure"
    } else {
        ""
    };
    let cookie = format!(
        "lucid_session=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0{}",
        secure_flag
    );
    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie)
            .map_err(|e| anyhow::anyhow!("invalid cookie value: {}", e))?,
    );

    Ok((response_headers, "Logged out successfully"))
}

/// Get information about the authenticated user.
///
/// Returns the current user's profile information including ID, username, display name,
/// and email. Requires session cookie authentication (no CSRF token needed for GET requests).
///
/// # Example
///
/// ```bash
/// curl http://localhost:3000/v1/auth/me \
///   -b cookies.txt
/// ```
///
/// # Response
///
/// ```json
/// {
///   "id": "user_object_id",
///   "username": "admin",
///   "display_name": "Administrator",
///   "email": "admin@example.com"
/// }
/// ```
///
/// # Errors
///
/// - 401 Unauthorized: Missing or invalid session cookie
/// - 404 Not Found: User no longer exists in database (stale session)
/// - 500 Internal Server Error: Database failure
#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    tags = ["auth"],
    responses((status = 200, description = "User information", body = User))
)]
pub async fn auth_whoami(
    State(ctx): State<ApiContext>,
    Auth(caller): Auth,
) -> Result<Json<User>, ApiError> {
    // Fetch full user from database
    let user = UserStore::get(&*ctx.db, caller.id().to_string())
        .await?
        .ok_or_else(|| anyhow::anyhow!("user not found"))?;

    Ok(Json(user.into()))
}

#[cfg(test)]
mod tests;
