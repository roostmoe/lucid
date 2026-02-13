use std::borrow::Borrow;
use std::sync::Arc;

use dropshot::HttpError;
use lucid_auth::authz;
use lucid_auth::context::{OpContext, OpKind};
use lucid_common::api::error::Error;
use lucid_db::datastore::DataStore;

use lucid_beacon_config::BeaconConfig;

use crate::app::Beacon;

#[derive(Clone)]
pub struct Context {
    pub beacon: Arc<Beacon>,
}

impl Context {
    pub async fn new(
        config: BeaconConfig,
    ) -> Result<Self, Error> {
        let datastore = Arc::new(
            DataStore::open(config.clone().database.url).await
                .map_err(|e| Error::internal_anyhow("Failed to construct datastore".into(), e))?
        );

        let beacon = Beacon::new(
            config,
            datastore.clone(),
        ).await?;

        Ok(Context {
            beacon,
        })
    }
}

/// Authenticate the incoming request and build an [`OpContext`] scoped to
/// the actor's organisation.
///
/// This is the primary entry point for authenticated endpoints.
pub(crate) async fn op_context_for_external_api(
    rqctx: &dropshot::RequestContext<Context>,
) -> Result<OpContext, HttpError> {
    let ctx = rqctx.context();
    OpContext::new_async(
        async {
            let authn = Arc::new(
                ctx.beacon.authn.authn_request(rqctx).await?
            );
            let datastore = Arc::clone(&ctx.beacon.datastore);
            let authz = authz::Context::new(
                Arc::clone(&authn),
                Arc::clone(&ctx.beacon.authz),
                datastore,
            );
            Ok((authn, authz))
        },
        |metadata| OpContext::load_request_metadata(rqctx, metadata),
        OpKind::ExternalApiRequest,
    ).await
}

/// Allow `Authenticator<DataStore>` to borrow the `DataStore` out of our
/// Dropshot context. The authn pipeline calls `rqctx.context().borrow()` to
/// get `&DataStore`, which it then uses as a `SessionStore`.
impl Borrow<DataStore> for Context {
    fn borrow(&self) -> &DataStore {
        &self.beacon.datastore
    }
}
