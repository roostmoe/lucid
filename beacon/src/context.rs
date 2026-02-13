use std::borrow::Borrow;
use std::sync::Arc;

use dropshot::HttpError;
use lucid_auth::authn::external::{Authenticator, HttpAuthnScheme};
use lucid_auth::authn::external::session_cookie::HttpAuthnSessionCookie;
use lucid_auth::{authn, authz};
use lucid_auth::context::{OpContext, OpKind};
use lucid_common::api::error::Error;
use lucid_db::datastore::DataStore;

use lucid_beacon_config::{SessionConfig};

#[derive(Clone)]
pub struct Context {
    pub datastore: Arc<DataStore>,
    pub authn: Arc<Authenticator<DataStore>>,
    pub authz: Arc<authz::Authz>,
    pub(crate) session_config: SessionConfig,
    pub(crate) opctx_external_authn: OpContext,
}

impl Context {
    pub async fn new(
        database_url: String,
        session_config: SessionConfig,
    ) -> Result<Self, Error> {
        let schemes: Vec<Box<dyn HttpAuthnScheme<DataStore>>> = vec![
            Box::new(HttpAuthnSessionCookie),
        ];

        let datastore = Arc::new(
            DataStore::open(database_url).await
                .map_err(|e| Error::internal_anyhow("Failed to construct datastore".into(), e))?
        );

        let authn = Authenticator::new(schemes);
        let authz = Arc::new(authz::Authz::new());

        Ok(Context {
            datastore: datastore.clone(),
            authn: Arc::new(authn),
            authz: authz.clone(),
            session_config,

            opctx_external_authn: OpContext::for_background(
                authz,
                authn::Context::external_authn(),
                Arc::clone(&datastore).clone()
                    as Arc<dyn lucid_auth::storage::Storage>,
            ),
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
                ctx.authn.authn_request(rqctx).await?
            );
            let datastore = Arc::clone(&ctx.datastore);
            let authz = authz::Context::new(
                Arc::clone(&authn),
                Arc::clone(&ctx.authz),
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
        &self.datastore
    }
}
