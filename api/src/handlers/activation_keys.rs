use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use lucid_common::{
    params::PaginationParams,
    views::{ActivationKey, PaginatedList},
};
use lucid_db::{
    models::DbActivationKey,
    storage::{ActivationKeyFilter, ActivationKeyStore},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    auth::{Auth, jwt::generate_activation_key_jwt},
    context::ApiContext,
    error::ApiError,
};

/// Request body for creating an activation key.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateActivationKeyRequest {
    /// User-provided identifier for this key
    pub key_id: String,
    /// Human-readable description
    pub description: String,
}

/// Response for activation key creation - includes the JWT token.
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateActivationKeyResponse {
    /// The created activation key metadata
    pub key: ActivationKey,
    /// The JWT token - only returned on creation, store it securely!
    pub token: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/activation-keys",
    tags = ["activation-keys"],
    request_body = CreateActivationKeyRequest,
    responses(
        (status = 201, description = "Activation key created", body = CreateActivationKeyResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn create_activation_key(
    State(ctx): State<ApiContext>,
    Auth(caller): Auth,
    Json(req): Json<CreateActivationKeyRequest>,
) -> Result<(StatusCode, Json<CreateActivationKeyResponse>), ApiError> {
    let db_key = DbActivationKey {
        id: None,
        key_id: req.key_id,
        description: req.description,
    };

    let created = ActivationKeyStore::create(&*ctx.db, caller, db_key).await?;

    let internal_id = created
        .id
        .map(|oid| oid.to_string())
        .ok_or_else(|| anyhow::anyhow!("Failed to get created key ID"))?;

    // Generate JWT
    let pem = ctx._config.get_signing_key_pem()?;

    let token =
        generate_activation_key_jwt(
            ctx.session_signer.inner().clone(),
            &pem,
            &ctx._config.public_url,
            &created.key_id,
            &internal_id,
        )
            .map_err(|e| anyhow::anyhow!(e))?;

    let key: ActivationKey = created.into();

    Ok((
        StatusCode::CREATED,
        Json(CreateActivationKeyResponse { key, token }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/activation-keys",
    tags = ["activation-keys"],
    responses(
        (status = 200, description = "List of activation keys", body = PaginatedList<ActivationKey>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn list_activation_keys(
    State(ctx): State<ApiContext>,
    Auth(caller): Auth,
    Query(query): Query<PaginationParams>,
) -> Result<Json<PaginatedList<ActivationKey>>, ApiError> {
    let keys =
        ActivationKeyStore::list(&*ctx.db, caller, ActivationKeyFilter::default(), query).await?;

    Ok(Json(PaginatedList {
        items: keys.into_iter().map(|k| k.into()).collect(),
        next_token: None,
        limit: None,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/activation-keys/{id}",
    tags = ["activation-keys"],
    responses(
        (status = 200, description = "Activation key details", body = ActivationKey),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_activation_key(
    State(ctx): State<ApiContext>,
    Auth(caller): Auth,
    Path(id): Path<String>,
) -> Result<Json<ActivationKey>, ApiError> {
    let key = ActivationKeyStore::get(&*ctx.db, caller, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(key.into()))
}

#[utoipa::path(
    delete,
    path = "/api/v1/activation-keys/{id}",
    tags = ["activation-keys"],
    responses(
        (status = 204, description = "Activation key deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn delete_activation_key(
    State(ctx): State<ApiContext>,
    Auth(caller): Auth,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    ActivationKeyStore::delete(&*ctx.db, caller, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
