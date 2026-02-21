use std::sync::Arc;

use lucid_db::storage::{Storage, mongodb::MongoDBStorage};

use crate::config::LucidApiConfig;

#[derive(Clone)]
pub struct ApiContext {
    pub _config: LucidApiConfig,
    pub db: Arc<dyn Storage>,
}

impl ApiContext {
    pub async fn new(config: LucidApiConfig) -> anyhow::Result<Self> {
        let db = Arc::new(MongoDBStorage::new(&config.mongodb_uri).await?);

        Ok(Self {
            _config: config,
            db,
        })
    }
}
