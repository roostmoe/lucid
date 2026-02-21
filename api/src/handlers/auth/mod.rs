use axum::{Json, extract::State};
use lucid_common::{
    caller::CallerError,
    params::AuthLoginParams,
    views::{AuthLoginResponse, User},
};
use lucid_db::storage::UserStore;
use tracing::info;

use crate::{context::ApiContext, error::ApiError};

#[utoipa::path(
    post,
    path = "/v1/auth/login",
    tags = ["auth", "console_sessions"],
    request_body(content = AuthLoginParams, content_type = "application/json"),
    responses((status = 201, description = "Successful login", body = AuthLoginResponse))
)]
pub async fn auth_login(
    State(ctx): State<ApiContext>,
    Json(body): Json<AuthLoginParams>,
) -> Result<Json<AuthLoginResponse>, ApiError> {
    let caller = UserStore::auth_local(&*ctx.db, body.username, body.password).await?;
    if caller.is_anonymous() {
        return Err(ApiError::CallerError(CallerError::unauthorized(None)));
    }
    info!("Logged in user {}", caller.api_caller()?.id()?);
    Ok(Json(AuthLoginResponse::Session))
}

#[utoipa::path(
    post,
    path = "/v1/auth/logout",
    tags = ["auth", "console_sessions"],
    responses((status = 200, description = "Successful logout"))
)]
pub async fn auth_logout() -> String {
    "Logout Endpoint".into()
}

#[utoipa::path(
    get,
    path = "/v1/auth/me",
    tags = ["auth"],
    responses((status = 200, description = "User information", body = User))
)]
pub async fn auth_whoami() -> Result<Json<User>, ApiError> {
    Ok(Json(User {
        id: "".into(),
        display_name: "John Doe".into(),
        email: "example@roost.moe".into(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }))
}
