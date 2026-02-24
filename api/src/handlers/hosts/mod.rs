use axum::{
    Json,
    extract::{Path, Query, State},
};
use chrono::Utc;
use lucid_common::{
    params::PaginationParams,
    views::{Host, PaginatedList},
};
use lucid_db::storage::{HostFilter, HostStore};

use crate::{auth::Auth, context::ApiContext, error::ApiError};

#[utoipa::path(
    get,
    path = "/api/v1/hosts",
    tags = ["hosts"],
    responses((status = 200, description = "List of hosts", body = PaginatedList<Host>))
)]
pub async fn list_hosts(
    State(ctx): State<ApiContext>,
    Auth(caller): Auth,
    Query(query): Query<PaginationParams>,
) -> Result<Json<PaginatedList<Host>>, ApiError> {
    let hosts = HostStore::list(&*ctx.db, caller, HostFilter::default(), query).await?;

    Ok(Json(PaginatedList {
        // TODO: Find a way to do this without cloning
        items: hosts.iter().map(|h| h.clone().into()).collect(),
        next_token: None,
        limit: None,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/hosts/{id}",
    tags = ["hosts"],
    responses((status = 200, description = "Resolved host", body = Host))
)]
pub async fn get_host(Path(id): Path<String>) -> Result<Json<Host>, ApiError> {
    let created_at = Utc::now();

    Ok(Json(Host {
        id,
        hostname: "example.com".into(),
        os_id: "".into(),
        os_name: "".into(),
        os_version: "".into(),
        architecture: "x86_64".into(),
        ifaces: vec![],
        created_at,
        updated_at: created_at,
        last_seen_at: created_at,
    }))
}
