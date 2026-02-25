use axum::{Json, extract::State};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{context::ApiContext, error::ApiError};

/// GET /.well-known/lucid/agent
/// Returns CA certificate information for agents.
#[utoipa::path(
    get,
    path = "/.well-known/lucid/agent",
    tags = ["well-known"],
    responses(
        (status = 200, description = "Agent configuration", body = AgentWellKnownResponse),
        (status = 503, description = "CA not initialized"),
    )
)]
pub async fn get_agent_well_known(
    State(ctx): State<ApiContext>,
) -> Result<Json<AgentWellKnownResponse>, ApiError> {
    let ca = ctx
        .ca
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("CA not initialized"))?;

    let ca_info = ca
        .get_ca_info()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get CA info: {}", e)))?;

    let response = AgentWellKnownResponse {
        server_version: env!("CARGO_PKG_VERSION").to_string(),
        cas: vec![CaInfoResponse {
            cert_pem: ca_info.cert_pem,
            fingerprint: ca_info.fingerprint,
            issued_at: ca_info.issued_at,
            expires_at: ca_info.expires_at,
        }],
    };

    Ok(Json(response))
}

#[derive(Serialize, ToSchema)]
pub struct AgentWellKnownResponse {
    pub server_version: String,
    pub cas: Vec<CaInfoResponse>,
}

#[derive(Serialize, ToSchema)]
pub struct CaInfoResponse {
    pub cert_pem: String,
    pub fingerprint: String,
    pub issued_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct ServerWellKnownResponse {
    pub server_version: String,
}

/// GET /.well-known/lucid/agent
/// Returns CA certificate information for agents.
#[utoipa::path(
    get,
    path = "/.well-known/lucid/server",
    tags = ["well-known"],
    responses(
        (status = 200, description = "Server configuration", body = AgentWellKnownResponse),
    )
)]
pub async fn get_server_well_known() -> Result<Json<ServerWellKnownResponse>, ApiError> {
    let response = ServerWellKnownResponse {
        server_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    Ok(Json(response))
}
