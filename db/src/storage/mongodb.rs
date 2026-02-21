use std::time::Duration;

use anyhow::anyhow;
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use async_trait::async_trait;
use futures::TryStreamExt;
use lucid_common::params::{CreateLocalUserParams, PaginationParams};
use mongodb::{
    Client, Database, IndexModel,
    bson::doc,
    options::{ClientOptions, FindOptions, IndexOptions},
};
use tracing::instrument;

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

        let storage = Self(client);

        storage.init().await?;

        Ok(storage)
    }

    fn get_db(&self) -> Database {
        self.0
            .default_database()
            .unwrap_or_else(|| self.0.database("lucid"))
    }

    async fn init(&self) -> Result<(), mongodb::error::Error> {
        self.get_db()
            .collection::<()>(MONGODB_COLLECTION_USERS)
            .create_index(
                IndexModel::builder()
                    .keys(doc! {"email": 1})
                    .options(IndexOptions::builder().unique(true).build())
                    .build(),
            )
            .await?;

        Ok(())
    }
}

#[async_trait]
impl Storage for MongoDBStorage {
    #[instrument(level = "debug", skip(self), err(Debug))]
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
    #[instrument(skip(self), err(Debug))]
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

        Ok(users.first().cloned())
    }

    #[instrument(skip(self), err(Debug))]
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

    async fn create_local(&self, user: CreateLocalUserParams) -> Result<DbUser, StoreError> {
        let collection = self.get_db().collection::<DbUser>(MONGODB_COLLECTION_USERS);

        let new_user = DbUser {
            id: None,
            display_name: user.display_name,
            email: user.email,
            password_hash: Some(hash_password(user.password).map_err(|e| anyhow!(e))?),
            updated_at: chrono::Utc::now(),
        };

        let insert_result = collection.insert_one(new_user.clone()).await?;

        Ok(DbUser {
            id: insert_result.inserted_id.as_object_id(),
            ..new_user
        })
    }
}

fn hash_password(password: String) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| e.to_string())?
        .to_string();

    Ok(password_hash)
}
