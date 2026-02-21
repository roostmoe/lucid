use axum::{
    Json,
    extract::{Path, Query},
};
use chrono::Utc;
use lucid_common::{
    params::PaginationParams,
    views::{Host, PaginatedList},
};
use ulid::Ulid;

use crate::error::ApiError;

#[utoipa::path(
    get,
    path = "/v1/hosts",
    tags = ["hosts"],
    responses((status = 200, description = "List of hosts", body = PaginatedList<Host>))
)]
pub async fn list_hosts(
    Query(_query): Query<PaginationParams>,
) -> Result<Json<PaginatedList<Host>>, ApiError> {
    Ok(Json(PaginatedList {
        items: vec![],
        next_token: None,
        limit: None,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/hosts/{id}",
    tags = ["hosts"],
    responses((status = 200, description = "Resolved host", body = Host))
)]
pub async fn get_host(Path(id): Path<Ulid>) -> Result<Json<Host>, ApiError> {
    let created_at = chrono::DateTime::<Utc>::from(id.datetime());

    Ok(Json(Host {
        id,
        hostname: "example.com".into(),
        ifaces: vec![],
        created_at: created_at,
        updated_at: created_at,
    }))
}
