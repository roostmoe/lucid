use futures::TryStreamExt;
use async_trait::async_trait;
use lucid_common::params::PaginationParams;
use mongodb::{Client, Database, bson::doc, options::FindOptions};

use crate::{models::DbUser, storage::{StoreError, UserFilter, UserStore}};

#[derive(Debug)]
pub struct MongoDBStorage(Client);

impl MongoDBStorage {
    pub async fn new(uri: &str) -> Result<Self, mongodb::error::Error> {
        let client = Client::with_uri_str(uri).await?;
        Ok(Self(client))
    }

    fn get_db(&self) -> Database {
        self
            .0
            .default_database()
            .unwrap_or_else(|| self.0.database("lucid"))
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
        ).await?;

        Ok(users.iter().nth(0).cloned())
    }

    async fn list(
        &self,
        filter: UserFilter,
        pagination: PaginationParams,
    ) -> Result<Vec<DbUser>, StoreError> {
        let collection = self
            .get_db()
            .collection::<DbUser>(MONGODB_COLLECTION_USERS);

        let find_options = FindOptions::builder()
            .limit(pagination.limit);

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
