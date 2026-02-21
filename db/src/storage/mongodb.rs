use std::time::Duration;

use async_trait::async_trait;
use futures::TryStreamExt;
use lucid_common::params::PaginationParams;
use mongodb::{
    Client, Database,
    bson::doc,
    options::{ClientOptions, FindOptions},
};

use crate::{
    models::DbUser,
    storage::{Storage, StoreError, UserFilter, UserStore},
};

#[derive(Debug)]
pub struct MongoDBStorage(Client);

impl MongoDBStorage {
    pub async fn new(uri: &str) -> Result<Self, mongodb::error::Error> {
        let mut client_opts = ClientOptions::parse(uri).await?;
        if client_opts.app_name.is_none() {
            client_opts.app_name = Some("Lucid".to_string());
        }
        if client_opts.connect_timeout.is_none() {
            client_opts.connect_timeout = Some(Duration::from_secs(3));
        }
        if client_opts.server_selection_timeout.is_none() {
            client_opts.server_selection_timeout = Some(Duration::from_secs(3));
        }

        let client = Client::with_options(client_opts)?;
        Ok(Self(client))
    }

    fn get_db(&self) -> Database {
        self.0
            .default_database()
            .unwrap_or_else(|| self.0.database("lucid"))
    }
}

#[async_trait]
impl Storage for MongoDBStorage {
    async fn ping(&self) -> Result<(), StoreError> {
        self.0
            .database("admin")
            .run_command(doc! {"ping": 1})
            .await?;

        Ok(())
    }
}

pub const MONGODB_COLLECTION_USERS: &str = "users";

#[async_trait]
impl UserStore for MongoDBStorage {
    async fn get(&self, id: String) -> Result<Option<DbUser>, StoreError> {
        let users = UserStore::list(
            self,
            UserFilter {
                id: Some(vec![id]),
                email: None,
            },
            PaginationParams { limit: 1, page: 0 },
        )
        .await?;

        Ok(users.get(0).cloned())
    }

    async fn list(
        &self,
        filter: UserFilter,
        pagination: PaginationParams,
    ) -> Result<Vec<DbUser>, StoreError> {
        let collection = self.get_db().collection::<DbUser>(MONGODB_COLLECTION_USERS);

        let find_options = FindOptions::builder().limit(pagination.limit);

        let mut filter_doc = doc! {};
        if let Some(ids) = filter.id {
            filter_doc.insert("_id", doc! { "$in": &ids });
        }
        if let Some(emails) = filter.email {
            filter_doc.insert("email", doc! { "$in": &emails });
        }

        collection
            .find(filter_doc)
            .with_options(find_options.build())
            .await?
            .try_collect()
            .await
            .map_err(StoreError::MongoDB)
    }
}
