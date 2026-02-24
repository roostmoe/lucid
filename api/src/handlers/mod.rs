use axum::extract::State;

use crate::{context::ApiContext, error::ApiError};

pub mod auth;
pub mod hosts;
pub mod jwks;

pub async fn health_check(State(ctx): State<ApiContext>) -> Result<&'static str, ApiError> {
    ctx.db.ping().await?;
    Ok("Healthy")
}
