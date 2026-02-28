use std::sync::Arc;

use lucid_db::storage::{Storage, mongodb::MongoDBStorage};

use crate::{
    auth::{
        ActivationKeyAuthProvider, AuthManager, CertificateAuthority, MtlsAuthProvider,
        encrypted_ca::EncryptedCa,
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
    pub session_signer: SessionSigner<Ed25519Signer>,
    pub ca: Option<Arc<dyn CertificateAuthority>>,
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
        // mTLS is tried first (for agent connections), then session (for web console)
        let auth_manager = AuthManager::new()
            .with_provider(ActivationKeyAuthProvider::new(
                Arc::clone(&db),
                config.public_url.clone(),
                session_signer.clone(),
            ))
            .with_provider(MtlsAuthProvider::new(Arc::clone(&db)))
            .with_provider(SessionAuthProvider::new(
                session_signer.clone(),
                Arc::clone(&db),
            ));

        // Initialize CA if encryption key is available
        let ca: Option<Arc<dyn CertificateAuthority>> =
            if let Ok(encryption_key) = EncryptedCa::encryption_key_from_env() {
                Some(Arc::new(EncryptedCa::new(Arc::clone(&db), encryption_key)))
            } else {
                None
            };

        Ok(Self {
            _config: config,
            db,
            auth_manager: Arc::new(auth_manager),
            session_signer,
            ca,
        })
    }
}
