use std::sync::Arc;

use lucid_db::storage::{Storage, mongodb::MongoDBStorage};

use crate::{
    auth::{
        AuthManager,
        providers::session::SessionAuthProvider,
        signing::{Ed25519Signer, SessionSigner},
    },
    config::LucidApiConfig,
};

#[derive(Clone)]
pub struct ApiContext {
    pub _config: LucidApiConfig,
    pub db: Arc<dyn Storage>,
    pub auth_manager: Arc<AuthManager>,
}

impl ApiContext {
    pub async fn new(config: LucidApiConfig, _auth_manager: AuthManager) -> anyhow::Result<Self> {
        let db: Arc<dyn Storage> = Arc::new(MongoDBStorage::new(&config.mongodb_uri).await?);

        // Initialize Ed25519 session signing
        // This loads the private key from config and creates a session token signer
        let signing_key_pem = config.get_signing_key_pem()?;
        let ed25519_signer = Ed25519Signer::from_pem(&signing_key_pem)?;
        let session_signer = SessionSigner::new(ed25519_signer);

        // Wire up auth providers
        let auth_manager = AuthManager::new()
            .with_provider(SessionAuthProvider::new(session_signer, Arc::clone(&db)));

        Ok(Self {
            _config: config,
            db,
            auth_manager: Arc::new(auth_manager),
        })
    }
}
