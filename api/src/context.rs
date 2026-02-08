use std::borrow::Borrow;
use std::sync::Arc;

use dropshot::HttpError;
use lucid_auth::authn::external::Authenticator;
use lucid_auth::authz::AuthzStorage;
use lucid_auth::context::OpContext;
use lucid_db::datastore::DataStore;

pub struct Context {
    pub datastore: Arc<DataStore>,
    pub authenticator: Authenticator<DataStore>,
}

impl Context {
    pub fn new(
        datastore: Arc<DataStore>,
        authenticator: Authenticator<DataStore>,
    ) -> Self {
        Context {
            datastore,
            authenticator,
        }
    }

    /// Authenticate the incoming request and build an [`OpContext`] scoped to
    /// the actor's organisation.
    ///
    /// This is the primary entry point for authenticated endpoints.
    pub async fn op_context_for_external_api(
        &self,
        rqctx: &dropshot::RequestContext<Context>,
    ) -> Result<OpContext, HttpError> {
        let authn_ctx = Arc::new(self.authenticator.authn_request(rqctx).await?);

        let actor = authn_ctx.actor_required().map_err(HttpError::from)?;

        let organisation_id =
            actor.organisation_id().ok_or_else(|| {
                HttpError::for_internal_error(
                    "authenticated actor has no organisation_id".to_string(),
                )
            })?;

        let opctx = OpContext::for_external_api(
            authn_ctx,
            organisation_id,
            self.datastore.clone() as Arc<dyn AuthzStorage>,
            |metadata| {
                OpContext::load_request_metadata(rqctx, metadata);
            },
        )
        .await
        .map_err(HttpError::from)?;

        Ok(opctx)
    }
}

/// Allow `Authenticator<DataStore>` to borrow the `DataStore` out of our
/// Dropshot context. The authn pipeline calls `rqctx.context().borrow()` to
/// get `&DataStore`, which it then uses as a `SessionStore`.
impl Borrow<DataStore> for Context {
    fn borrow(&self) -> &DataStore {
        &self.datastore
    }
}
