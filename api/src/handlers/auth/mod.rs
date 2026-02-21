use axum::Json;
use lucid_common::{params::AuthLoginParams, views::AuthLoginResponse};
use tracing::info;

use crate::error::ApiError;

#[axum::debug_handler]
#[utoipa::path(
    post,
    path = "/v1/auth/login",
    tags = ["auth", "console_sessions"],
    request_body(content = AuthLoginParams, content_type = "application/json"),
    responses((status = 201, description = "Successful login", body = AuthLoginResponse))
)]
pub async fn auth_login(
    Json(_body): Json<AuthLoginParams>,
) -> Result<Json<AuthLoginResponse>, ApiError> {
    Ok(Json(AuthLoginResponse::Session))
}

#[axum::debug_handler]
#[utoipa::path(
    post,
    path = "/v1/auth/logout",
    tags = ["auth", "console_sessions"],
    responses((status = 200, description = "Successful logout"))
)]
pub async fn auth_logout() -> String {
    "Logout Endpoint".into()
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = "/v1/auth/me",
    tags = ["auth"],
    responses((status = 200, description = "User information"))
)]
pub async fn auth_whoami() -> String {
    info!("Hello!");
    "Whoami Endpoint".into()
}
