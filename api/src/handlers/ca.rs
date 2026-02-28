use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use lucid_common::views::{Ca, PaginatedList};
use lucid_db::{models::DbCa, storage::CaStore};
use pem_rfc7468::decode_vec;
use rcgen::{CertificateParams, KeyPair};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use ulid::Ulid;
use utoipa::ToSchema;

use crate::{
    auth::{Auth, encrypted_ca::EncryptedCa},
    context::ApiContext,
    error::ApiError,
};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Convert a `DbCa` to its public view type, computing the SHA-256 fingerprint
/// from the PEM cert in the process.
fn db_ca_to_view(ca: DbCa) -> Result<Ca, ApiError> {
    let cert_der = decode_vec(ca.cert_pem.as_bytes())
        .map_err(|e| anyhow::anyhow!("Failed to decode CA cert PEM: {e}"))?
        .1;

    let mut hasher = Sha256::new();
    hasher.update(&cert_der);
    let fingerprint = format!("sha256:{}", hex::encode(hasher.finalize()));

    Ok(Ca {
        id: ca.id.into(),
        cert_pem: ca.cert_pem,
        fingerprint,
        created_at: ca.created_at,
    })
}

/// Load the server's CA encryption key or return a 500.
fn get_encryption_key() -> Result<[u8; 32], ApiError> {
    EncryptedCa::encryption_key_from_env()
        .map_err(|e| anyhow::anyhow!("CA encryption key unavailable: {e}").into())
}

// ---------------------------------------------------------------------------
// Request bodies
// ---------------------------------------------------------------------------

/// Request body for importing an existing certificate authority.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ImportCaRequest {
    /// PEM-encoded CA certificate.
    pub cert_pem: String,
    /// PEM-encoded private key for the CA certificate (will be encrypted at
    /// rest using the server's `LUCID_CA_ENCRYPTION_KEY`).
    pub private_key_pem: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Generate a new self-signed Ed25519 CA certificate and store it.
#[utoipa::path(
    post,
    path = "/api/v1/cas",
    tags = ["cas"],
    responses(
        (status = 201, description = "Certificate authority generated", body = Ca),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn generate_ca(
    State(ctx): State<ApiContext>,
    Auth(caller): Auth,
) -> Result<(StatusCode, Json<Ca>), ApiError> {
    caller.require(lucid_common::caller::Permission::CaWrite)?;

    let encryption_key = get_encryption_key()?;

    let ca_info = crate::auth::encrypted_ca::generate_ca(&*ctx.db, &encryption_key, false)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to generate CA: {e}"))?;

    // Fetch the stored record back so we have an ID to return.
    let db_ca = lucid_db::storage::CaStore::list(&*ctx.db, lucid_common::caller::Caller::System)
        .await?
        .into_iter()
        .find(|c| c.cert_pem == ca_info.cert_pem)
        .ok_or_else(|| anyhow::anyhow!("Stored CA not found immediately after creation"))?;

    Ok((StatusCode::CREATED, Json(db_ca_to_view(db_ca)?)))
}

/// Import an existing CA certificate and private key.
#[utoipa::path(
    post,
    path = "/api/v1/cas/import",
    tags = ["cas"],
    request_body = ImportCaRequest,
    responses(
        (status = 201, description = "Certificate authority imported", body = Ca),
        (status = 400, description = "Invalid certificate or key"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn import_ca(
    State(ctx): State<ApiContext>,
    Auth(caller): Auth,
    Json(req): Json<ImportCaRequest>,
) -> Result<(StatusCode, Json<Ca>), ApiError> {
    // Validate that the cert and key are actually a matching CA pair before
    // storing them.
    let key_pair = KeyPair::from_pem(&req.private_key_pem)
        .map_err(|e| ApiError::bad_request(format!("Invalid private key PEM: {e}")))?;
    CertificateParams::from_ca_cert_pem(&req.cert_pem)
        .map_err(|e| ApiError::bad_request(format!("Invalid CA certificate PEM: {e}")))?;

    let encryption_key = get_encryption_key()?;

    // Pre-generate the ObjectId so we can bind the encrypted key to this
    // specific CA record via AAD (prevents ciphertext transplantation).
    let ca_id = Ulid::new();
    let aad = ca_id.to_string();

    let encrypted_private_key = crate::crypto::aes::encrypt(
        &encryption_key,
        req.private_key_pem.as_bytes(),
        aad.as_bytes(),
    )
    .map_err(|e| anyhow::anyhow!("Failed to encrypt private key: {e}"))?;

    drop(key_pair);

    let db_ca = DbCa {
        id: ca_id.into(),
        cert_pem: req.cert_pem,
        encrypted_private_key,
        created_at: chrono::Utc::now(),
    };

    let created = CaStore::create(&*ctx.db, caller, db_ca).await?;
    Ok((StatusCode::CREATED, Json(db_ca_to_view(created)?)))
}

#[utoipa::path(
    get,
    path = "/api/v1/cas",
    tags = ["cas"],
    responses(
        (status = 200, description = "List of certificate authorities", body = PaginatedList<Ca>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn list_cas(
    State(ctx): State<ApiContext>,
    Auth(caller): Auth,
) -> Result<Json<PaginatedList<Ca>>, ApiError> {
    let cas = CaStore::list(&*ctx.db, caller).await?;

    let items = cas
        .into_iter()
        .map(db_ca_to_view)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(PaginatedList {
        items,
        next_token: None,
        limit: None,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/cas/{id}",
    tags = ["cas"],
    responses(
        (status = 200, description = "Certificate authority details", body = Ca),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_ca(
    State(ctx): State<ApiContext>,
    Auth(caller): Auth,
    Path(id): Path<Ulid>,
) -> Result<Json<Ca>, ApiError> {
    let ca = CaStore::get(&*ctx.db, caller, id.into())
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(db_ca_to_view(ca)?))
}

#[utoipa::path(
    delete,
    path = "/api/v1/cas/{id}",
    tags = ["cas"],
    responses(
        (status = 204, description = "Certificate authority deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_ca(
    State(ctx): State<ApiContext>,
    Auth(caller): Auth,
    Path(id): Path<Ulid>,
) -> Result<StatusCode, ApiError> {
    CaStore::delete(&*ctx.db, caller, id.into()).await?;
    Ok(StatusCode::NO_CONTENT)
}
