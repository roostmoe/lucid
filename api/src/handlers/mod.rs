use axum::extract::State;

use crate::{context::ApiContext, error::ApiError};

pub mod activation_keys;
pub mod agents;
pub mod auth;
pub mod ca;
pub mod hosts;
pub mod jwks;
pub mod well_known;

pub async fn health_check(State(ctx): State<ApiContext>) -> Result<&'static str, ApiError> {
    ctx.db.ping().await?;
    Ok("Healthy")
}
