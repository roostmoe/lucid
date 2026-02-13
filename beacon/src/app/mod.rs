use std::sync::Arc;

use lucid_auth::{authn::{self, external::{Authenticator, HttpAuthnScheme, session_cookie::HttpAuthnSessionCookie}}, authz, context::OpContext};
use lucid_beacon_config::BeaconConfig;
use lucid_db::datastore::DataStore;
use lucid_common::api::error::Error;

pub(crate) mod console_session;

pub struct Beacon {
    pub datastore: Arc<DataStore>,
    pub authn: Arc<Authenticator<DataStore>>,
    pub authz: Arc<authz::Authz>,
    pub(crate) config: BeaconConfig,
    pub(crate) opctx_external_authn: OpContext,
}

impl Beacon {
    pub async fn new(
        config: BeaconConfig,
        datastore: Arc<DataStore>,
    ) -> Result<Arc<Beacon>, Error> {
        let schemes: Vec<Box<dyn HttpAuthnScheme<DataStore>>> = vec![
            Box::new(HttpAuthnSessionCookie),
        ];

        let authn = Authenticator::new(schemes);
        let authz = Arc::new(authz::Authz::new());

        let opctx_external_authn = OpContext::for_background(
            authz.clone(),
            authn::Context::external_authn(),
            Arc::clone(&datastore).clone()
            as Arc<dyn lucid_auth::storage::Storage>,
        );

        Ok(Arc::new(Self {
            config,
            datastore,
            authn: Arc::new(authn),
            authz,
            opctx_external_authn,
        }))
    }
}
