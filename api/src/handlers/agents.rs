use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header},
};
use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use lucid_common::{caller::Caller, params::RegisterAgentRequest, views::RegisterAgentResponse};
use lucid_db::{
    models::{DbAgent, DbHost, OperatingSystem},
    storage::{ActivationKeyStore, AgentStore, HostStore},
};
use mongodb::bson::oid::ObjectId;
use tracing::{debug, info, instrument};
use x509_parser::prelude::*;

use crate::{auth::jwt::ActivationKeyClaims, context::ApiContext, error::ApiError};

/// POST /api/v1/agents/register
///
/// Register a new agent using an activation key JWT.
#[utoipa::path(
    post,
    path = "/api/v1/agents/register",
    tags = ["agents"],
    request_body = RegisterAgentRequest,
    responses(
        (status = 200, description = "Agent registered successfully", body = RegisterAgentResponse),
        (status = 400, description = "Invalid CSR"),
        (status = 401, description = "Invalid or expired activation key"),
        (status = 409, description = "Activation key already used"),
        (status = 503, description = "CA not initialized"),
    ),
    security(
        ("activation_key" = [])
    )
)]
#[instrument(skip(ctx))]
pub async fn register_agent(
    State(ctx): State<ApiContext>,
    headers: HeaderMap,
    Json(req): Json<RegisterAgentRequest>,
) -> Result<(StatusCode, Json<RegisterAgentResponse>), ApiError> {
    debug!("Agent registration request received");

    // 1. Extract Bearer token from Authorization header
    let token = extract_bearer_token(&headers)?;

    // 2. Manually validate activation key JWT to extract the activation key ID
    let (claims, activation_key) = validate_activation_key_jwt(&ctx, &token).await?;

    debug!(
        key_id = %activation_key.key_id,
        ak = %claims.ak,
        "Activation key validated"
    );

    // 2. Check activation key not used
    let activation_key_id = activation_key
        .id
        .ok_or_else(|| ApiError::internal("Activation key missing ID"))?;

    if ActivationKeyStore::is_used(&*ctx.db, activation_key_id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to check key usage: {}", e)))?
    {
        return Err(ApiError::conflict("Activation key already used"));
    }

    debug!("Activation key unused, proceeding with registration");

    // 3. Get CA from context
    let ca = ctx
        .ca
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("CA not initialized"))?;

    // 4. Extract public key from CSR
    let public_key_pem = extract_public_key_pem(&req.csr_pem)?;

    debug!("Public key extracted from CSR");

    // 5. Create new agent UUID
    let agent_id = ObjectId::new();

    debug!(agent_id = %agent_id, "Generated agent ID");

    // 6. Sign CSR via CA
    let signed_cert = ca
        .sign_csr(&req.csr_pem, agent_id)
        .await
        .map_err(|e| ApiError::bad_request(format!("Failed to sign CSR: {}", e)))?;

    debug!("CSR signed successfully");

    // 7. Create DbHost with minimal info
    let host = DbHost {
        id: None, // Will be assigned by DB
        hostname: req.hostname.clone(),
        architecture: "unknown".to_string(),
        operating_system: OperatingSystem {
            id: "unknown".to_string(),
            name: "Unknown".to_string(),
            version: "0".to_string(),
        },
        agent_id: Some(agent_id),
        updated_at: Utc::now(),
        last_seen_at: Utc::now(),
    };

    let created_host = HostStore::create(&*ctx.db, lucid_common::caller::Caller::System, host)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to create host: {}", e)))?;

    let host_id = created_host
        .id
        .ok_or_else(|| ApiError::internal("Host missing ID after creation"))?;

    debug!(host_id = %host_id, "Host created");

    // 8. Create DbAgent linking to host
    let agent = DbAgent {
        id: Some(agent_id),
        name: req.hostname.clone(),
        host_id,
        public_key_pem,
        certificate_pem: signed_cert.cert_pem.clone(),
        cert_issued_at: signed_cert.issued_at,
        cert_expires_at: signed_cert.expires_at,
        last_seen_at: None,
        revoked_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    AgentStore::create(&*ctx.db, agent)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to create agent: {}", e)))?;

    debug!(agent_id = %agent_id, "Agent created");

    // 9. Mark activation key as used
    ActivationKeyStore::mark_as_used(&*ctx.db, activation_key_id, agent_id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to mark key as used: {}", e)))?;

    debug!("Activation key marked as used");

    // 10. Get CA certificate
    let ca_cert_pem = ca
        .get_ca_cert_pem()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get CA cert: {}", e)))?;

    // 11. Return response
    Ok((
        StatusCode::OK,
        Json(RegisterAgentResponse {
            agent_id: agent_id.to_string(),
            certificate_pem: signed_cert.cert_pem,
            ca_certificate_pem: ca_cert_pem,
            expires_at: signed_cert.expires_at,
            api_base_url: ctx._config.public_url.clone(),
        }),
    ))
}

/// Validate activation key JWT and return claims + activation key record
async fn validate_activation_key_jwt(
    ctx: &ApiContext,
    token: &str,
) -> Result<(ActivationKeyClaims, lucid_db::models::DbActivationKey), ApiError> {
    // Decode and verify JWT
    let public_key_bytes = ctx.session_signer.inner().public_key_bytes();
    let decoding_key = DecodingKey::from_ed_der(public_key_bytes);
    let mut validation = Validation::new(Algorithm::EdDSA);
    validation.validate_exp = false;
    validation.required_spec_claims.clear();
    validation.set_issuer(&[&ctx._config.public_url]);

    let token_data = decode::<ActivationKeyClaims>(token, &decoding_key, &validation)
        .map_err(|e| ApiError::unauthorized(format!("Invalid JWT: {}", e)))?;

    let claims = token_data.claims;
    info!(?claims, "Validating token with claims...");

    // Look up activation key
    let activation_key = ActivationKeyStore::get(&*ctx.db, Caller::System, claims.ak.clone())
        .await
        .map_err(|e| ApiError::internal(format!("DB error: {}", e)))?
        .ok_or_else(|| ApiError::unauthorized("Invalid activation key"))?;

    Ok((claims, activation_key))
}

/// Extract Ed25519 public key from CSR in PEM format
fn extract_public_key_pem(csr_pem: &str) -> Result<String, ApiError> {
    // Parse the PEM-encoded CSR
    let (_, pem) = parse_x509_pem(csr_pem.as_bytes())
        .map_err(|e| ApiError::bad_request(format!("Invalid PEM format: {}", e)))?;

    // Parse the CSR
    let (_, csr) = X509CertificationRequest::from_der(&pem.contents)
        .map_err(|e| ApiError::bad_request(format!("Invalid CSR: {}", e)))?;

    // Verify the CSR signature
    csr.verify_signature()
        .map_err(|e| ApiError::bad_request(format!("CSR signature verification failed: {}", e)))?;

    // Extract the public key bytes
    let spki = csr.certification_request_info.subject_pki;

    // Convert to PEM format
    // Ed25519 public keys in PEM format use the "PUBLIC KEY" label with SPKI structure
    use pem_rfc7468::{LineEnding, encode_string};
    let public_key_pem = encode_string("PUBLIC KEY", LineEnding::LF, spki.raw)
        .map_err(|e| ApiError::internal(format!("Failed to encode public key as PEM: {}", e)))?;

    Ok(public_key_pem)
}

/// Extract Bearer token from Authorization header
fn extract_bearer_token(headers: &HeaderMap) -> Result<String, ApiError> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .ok_or_else(|| ApiError::unauthorized("Missing Authorization header"))?
        .to_str()
        .map_err(|_| ApiError::unauthorized("Invalid Authorization header"))?;

    auth_header
        .strip_prefix("Bearer ")
        .map(|s| s.to_string())
        .ok_or_else(|| ApiError::unauthorized("Invalid Bearer token format"))
}
