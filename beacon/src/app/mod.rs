use std::sync::Arc;

use lucid_auth::authn::{JwtManager, OidcClient};
use lucid_beacon_config::BeaconConfig;
use lucid_common::api::error::Error;
use lucid_db::datastore::DataStore;

pub(crate) mod oidc_state;

pub struct Beacon {
    pub datastore: Arc<DataStore>,
    pub jwt: Arc<JwtManager>,
    pub oidc: Arc<OidcClient>,
    pub oidc_state: Arc<oidc_state::OidcStateCache>,
    #[allow(dead_code)]
    pub(crate) config: BeaconConfig,
}

impl Beacon {
    pub async fn new(
        config: BeaconConfig,
        datastore: Arc<DataStore>,
    ) -> Result<Arc<Beacon>, Error> {
        let jwt_config = lucid_auth::authn::JwtConfig {
            secret: config.auth.jwt.secret.clone(),
            issuer: config.auth.jwt.issuer.clone(),
            audience: config.auth.jwt.audience.clone(),
            expiry_hours: config.auth.jwt.expiry_hours,
        };
        let jwt = Arc::new(JwtManager::new(jwt_config));

        let oidc_config = lucid_auth::authn::OidcConfig {
            discovery_url: config.auth.oidc.discovery_url.clone(),
            client_id: config.auth.oidc.client_id.clone(),
            client_secret: config.auth.oidc.client_secret.clone(),
            redirect_uri: config.auth.oidc.redirect_uri.clone(),
            scopes: config.auth.oidc.scopes.clone().unwrap_or_else(|| {
                vec![
                    "openid".to_string(),
                    "email".to_string(),
                    "profile".to_string(),
                ]
            }),
            allowed_domains: config.auth.oidc.allowed_domains.clone().unwrap_or_default(),
            allowed_emails: config.auth.oidc.allowed_emails.clone().unwrap_or_default(),
            owner_email: None,
        };
        let oidc = Arc::new(OidcClient::new(oidc_config).await.map_err(|e| {
            Error::internal_error(&format!("failed to initialise OIDC client: {e}"))
        })?);

        let oidc_state = Arc::new(oidc_state::OidcStateCache::new());

        Ok(Arc::new(Self {
            config,
            datastore,
            jwt,
            oidc,
            oidc_state,
        }))
    }
}
