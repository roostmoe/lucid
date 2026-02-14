use std::sync::Arc;

use dropshot::{ClientErrorStatusCode, HttpError, RequestContext};
use lucid_auth::authn;
use lucid_auth::context::{OpContext, OpKind};
use lucid_beacon_config::BeaconConfig;
use lucid_common::api::error::Error;
use lucid_db::datastore::DataStore;
use lucid_uuid_kinds::UserIdUuid;

use crate::app::Beacon;

#[derive(Clone)]
pub struct Context {
    pub beacon: Arc<Beacon>,
}

impl Context {
    pub async fn new(config: BeaconConfig) -> Result<Self, Error> {
        let datastore = Arc::new(
            DataStore::open(config.clone().database.url)
                .await
                .map_err(|e| Error::internal_anyhow("Failed to construct datastore".into(), e))?,
        );

        let beacon = Beacon::new(config, datastore.clone()).await?;

        Ok(Context { beacon })
    }
}

/// Authenticate the incoming request via JWT and build an [`OpContext`].
///
/// This is the primary entry point for authenticated endpoints.
#[allow(dead_code)]
pub(crate) async fn op_context_for_external_api(
    rqctx: &RequestContext<Context>,
) -> Result<OpContext, HttpError> {
    let ctx = rqctx.context();

    // Extract Authorization header
    let auth_header = rqctx
        .request
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let authn_context = match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let token = &header[7..];
            let claims = ctx.beacon.jwt.validate_token(token).map_err(|_| {
                HttpError::for_client_error(
                    None,
                    ClientErrorStatusCode::UNAUTHORIZED,
                    "invalid token".to_string(),
                )
            })?;

            let user_id = claims.sub.parse::<UserIdUuid>().map_err(|_| {
                HttpError::for_internal_error("invalid user id in token".to_string())
            })?;

            Arc::new(authn::Context::user(user_id))
        }
        _ => Arc::new(authn::Context::unauthenticated()),
    };

    Ok(OpContext::new(
        authn_context,
        |metadata| OpContext::load_request_metadata(rqctx, metadata),
        OpKind::ExternalApiRequest,
    ))
}
