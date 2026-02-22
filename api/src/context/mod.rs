use std::sync::Arc;

use lucid_db::storage::{Storage, mongodb::MongoDBStorage};

use crate::{auth::{AuthManager, SessionAuthProvider}, config::LucidApiConfig};

#[derive(Clone)]
pub struct ApiContext {
    pub _config: LucidApiConfig,
    pub db: Arc<dyn Storage>,
    pub auth_manager: Arc<AuthManager>,
}

impl ApiContext {
    pub async fn new(config: LucidApiConfig, auth_manager: AuthManager) -> anyhow::Result<Self> {
        let db = Arc::new(MongoDBStorage::new(&config.mongodb_uri).await?);

        Ok(Self {
            _config: config,
            db,
            auth_manager: Arc::new(auth_manager),
        })
    }
}
