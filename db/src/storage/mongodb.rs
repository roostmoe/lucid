use std::{str::FromStr, time::Duration};

use anyhow::anyhow;
use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use async_trait::async_trait;
use chrono::Utc;
use futures::TryStreamExt;
use lucid_common::{
    caller::{Caller, Permission},
    params::{CreateLocalUserParams, PaginationParams},
};
use mongodb::{
    Client, Database, IndexModel,
    bson::{DateTime as BsonDateTime, doc, oid::ObjectId},
    options::{ClientOptions, FindOptions, IndexOptions},
};
use tracing::{info, instrument};

use crate::{
    models::{DbHost, DbSession, DbUser},
    storage::{HostFilter, HostStore, SessionStore, Storage, StoreError, UserFilter, UserStore},
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
        // Users collection indexes
        self.get_db()
            .collection::<()>(MONGODB_COLLECTION_USERS)
            .create_index(
                IndexModel::builder()
                    .keys(doc! {"email": 1})
                    .options(IndexOptions::builder().unique(true).build())
                    .build(),
            )
            .await?;

        // Sessions collection indexes
        let sessions_collection = self.get_db().collection::<()>(MONGODB_COLLECTION_SESSIONS);

        // Unique index on session_id
        sessions_collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! {"session_id": 1})
                    .options(IndexOptions::builder().unique(true).build())
                    .build(),
            )
            .await?;

        // Index on user_id for finding user's sessions
        sessions_collection
            .create_index(IndexModel::builder().keys(doc! {"user_id": 1}).build())
            .await?;

        // TTL index on expires_at for automatic cleanup
        sessions_collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! {"expires_at": 1})
                    .options(
                        IndexOptions::builder()
                            .expire_after(Duration::from_secs(0))
                            .build(),
                    )
                    .build(),
            )
            .await?;

        // Hosts collection indexes
        self.get_db()
            .collection::<()>(MONGODB_COLLECTION_INVENTORY_HOSTS)
            .create_index(
                IndexModel::builder()
                    .keys(doc! {"hostname": 1})
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
pub const MONGODB_COLLECTION_SESSIONS: &str = "console_sessions";
pub const MONGODB_COLLECTION_INVENTORY_HOSTS: &str = "inventory_hosts";

#[async_trait]
impl UserStore for MongoDBStorage {
    #[instrument(skip(self), err(Debug))]
    async fn get(&self, caller: Caller, id: String) -> Result<Option<DbUser>, StoreError> {
        caller.require(Permission::UsersRead).map_err(|_| StoreError::PermissionDenied)?;

        let users = UserStore::list(
            self,
            caller,
            UserFilter {
                id: Some(vec![id]),
                email: None,
            },
            PaginationParams {
                limit: Some(1),
                page: Some(0),
            },
        )
        .await?;

        Ok(users.first().cloned())
    }

    #[instrument(skip(self), err(Debug))]
    async fn list(
        &self,
        caller: Caller,
        filter: UserFilter,
        pagination: PaginationParams,
    ) -> Result<Vec<DbUser>, StoreError> {
        caller.require(Permission::UsersRead).map_err(|_| StoreError::PermissionDenied)?;

        let collection = self.get_db().collection::<DbUser>(MONGODB_COLLECTION_USERS);

        let find_options = FindOptions::builder().limit(pagination.limit);

        let mut filter_doc = doc! {};
        if let Some(ids) = filter.id {
            let object_ids: Vec<ObjectId> = ids
                .into_iter()
                .filter_map(|id| ObjectId::from_str(&id).ok())
                .collect();

            filter_doc.insert("_id", doc! { "$in": object_ids });
        }
        if let Some(emails) = filter.email {
            filter_doc.insert("email", doc! { "$in": &emails });
        }

        info!(
            "Finding users with {filter}",
            filter = filter_doc.to_string()
        );

        collection
            .find(filter_doc)
            .with_options(find_options.build())
            .await?
            .try_collect()
            .await
            .map_err(StoreError::MongoDB)
    }

    async fn create_local(&self, caller: Caller, user: CreateLocalUserParams) -> Result<DbUser, StoreError> {
        caller.require(Permission::UsersWrite).map_err(|_| StoreError::PermissionDenied)?;

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

    #[instrument(skip(self), err(Debug))]
    async fn auth_local(&self, caller: Caller, email: String, password: String) -> Result<Caller, StoreError> {
        caller.require(Permission::UsersRead).map_err(|_| StoreError::PermissionDenied)?;

        let users = UserStore::list(
            self,
            caller,
            UserFilter {
                id: None,
                email: Some(vec![email]),
            },
            PaginationParams {
                limit: Some(1),
                page: Some(0),
            },
        )
        .await?;

        let user = users.first().ok_or_else(|| StoreError::NotFound)?;

        if user.password_hash.is_none() {
            return Err(StoreError::NotFound);
        }
        let pw_hash = user.password_hash.clone().unwrap();
        let matches = verify_password(password, pw_hash.clone()).map_err(|e| anyhow!(e))?;
        if matches {
            Ok(user.to_caller())
        } else {
            Err(StoreError::InvalidCredentials)
        }
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

fn verify_password(password: String, hash: String) -> Result<bool, String> {
    let argon2 = Argon2::default();
    let pw_hash = PasswordHash::new(&hash).map_err(|e| e.to_string())?;
    let password_hash = argon2.verify_password(password.as_bytes(), &pw_hash);

    if password_hash.is_err() {
        return Ok(false);
    }

    Ok(true)
}

#[async_trait]
impl SessionStore for MongoDBStorage {
    #[instrument(skip(self), err(Debug))]
    async fn create_session(
        &self,
        user_id: mongodb::bson::oid::ObjectId,
        session_id: String,
        csrf_token: String,
        ttl: chrono::Duration,
    ) -> Result<DbSession, StoreError> {
        let collection = self
            .get_db()
            .collection::<DbSession>(MONGODB_COLLECTION_SESSIONS);

        let now = chrono::Utc::now();
        let expires_at = now + ttl;

        let new_session = DbSession {
            id: None,
            session_id,
            user_id,
            csrf_token,
            created_at: now,
            expires_at,
            last_used_at: now,
        };

        let insert_result = collection.insert_one(new_session.clone()).await?;

        Ok(DbSession {
            id: insert_result.inserted_id.as_object_id(),
            ..new_session
        })
    }

    #[instrument(skip(self), err(Debug))]
    async fn get_session(&self, session_id: &str) -> Result<Option<DbSession>, StoreError> {
        let collection = self
            .get_db()
            .collection::<DbSession>(MONGODB_COLLECTION_SESSIONS);

        let session = collection.find_one(doc! {"session_id": session_id}).await?;

        Ok(session)
    }

    #[instrument(skip(self), err(Debug))]
    async fn delete_session(&self, session_id: &str) -> Result<(), StoreError> {
        let collection = self
            .get_db()
            .collection::<DbSession>(MONGODB_COLLECTION_SESSIONS);

        collection
            .delete_one(doc! {"session_id": session_id})
            .await?;

        Ok(())
    }

    #[instrument(skip(self), err(Debug))]
    async fn touch_session(&self, session_id: &str) -> Result<(), StoreError> {
        let collection = self
            .get_db()
            .collection::<DbSession>(MONGODB_COLLECTION_SESSIONS);

        let bson_now = BsonDateTime::from_chrono(Utc::now());

        collection
            .update_one(
                doc! {"session_id": session_id},
                doc! {"$set": {"last_used_at": bson_now}},
            )
            .await?;

        Ok(())
    }

    #[instrument(skip(self), err(Debug))]
    async fn cleanup_expired_sessions(&self) -> Result<u64, StoreError> {
        let collection = self
            .get_db()
            .collection::<DbSession>(MONGODB_COLLECTION_SESSIONS);

        let bson_now = BsonDateTime::from_chrono(Utc::now());

        let result = collection
            .delete_many(doc! {"expires_at": {"$lt": bson_now}})
            .await?;

        Ok(result.deleted_count)
    }

    #[instrument(skip(self), err(Debug))]
    async fn delete_user_sessions(
        &self,
        user_id: mongodb::bson::oid::ObjectId,
    ) -> Result<u64, StoreError> {
        let collection = self
            .get_db()
            .collection::<DbSession>(MONGODB_COLLECTION_SESSIONS);

        let result = collection.delete_many(doc! {"user_id": user_id}).await?;

        Ok(result.deleted_count)
    }
}

#[async_trait]
impl HostStore for MongoDBStorage {
    #[instrument(skip(self), err(Debug))]
    async fn get(&self, caller: Caller, id: String) -> Result<Option<DbHost>, StoreError> {
        caller.require(Permission::HostsRead).map_err(|_| StoreError::PermissionDenied)?;

        let hosts = HostStore::list(
            self,
            caller,
            HostFilter {
                id: Some(vec![id]),
                ..Default::default()
            },
            PaginationParams {
                limit: Some(1),
                page: Some(0),
            },
        )
        .await?;

        Ok(hosts.first().cloned())
    }

    #[instrument(skip(self), err(Debug))]
    async fn list(
        &self,
        caller: Caller,
        filter: HostFilter,
        pagination: PaginationParams,
    ) -> Result<Vec<DbHost>, StoreError> {
        caller.require(Permission::HostsRead).map_err(|_| StoreError::PermissionDenied)?;

        let collection = self
            .get_db()
            .collection::<DbHost>(MONGODB_COLLECTION_INVENTORY_HOSTS);

        let find_options = FindOptions::builder().limit(pagination.limit);

        let mut filter_doc = doc! {};

        if let Some(ids) = filter.id {
            let object_ids: Vec<ObjectId> = ids
                .into_iter()
                .filter_map(|id| ObjectId::from_str(&id).ok())
                .collect();
            filter_doc.insert("_id", doc! { "$in": object_ids });
        }
        if let Some(hostnames) = filter.hostname {
            filter_doc.insert("hostname", doc! { "$in": &hostnames });
        }
        if let Some(archs) = filter.arch {
            filter_doc.insert("architecture", doc! { "$in": &archs });
        }
        if let Some(os_names) = filter.os_name {
            filter_doc.insert("operating_system.name", doc! { "$in": &os_names });
        }
        if let Some(os_versions) = filter.os_version {
            filter_doc.insert("operating_system.version", doc! { "$in": &os_versions });
        }

        info!(
            "Finding hosts with {filter}",
            filter = filter_doc.to_string()
        );

        collection
            .find(filter_doc)
            .with_options(find_options.build())
            .await?
            .try_collect()
            .await
            .map_err(StoreError::MongoDB)
    }

    #[instrument(skip(self), err(Debug))]
    async fn create(&self, caller: Caller, host: DbHost) -> Result<DbHost, StoreError> {
        caller.require(Permission::HostsWrite).map_err(|_| StoreError::PermissionDenied)?;

        let collection = self
            .get_db()
            .collection::<DbHost>(MONGODB_COLLECTION_INVENTORY_HOSTS);

        let insert_result = collection.insert_one(host.clone()).await?;

        Ok(DbHost {
            id: insert_result.inserted_id.as_object_id(),
            ..host
        })
    }

    #[instrument(skip(self), err(Debug))]
    async fn update(&self, caller: Caller, host: DbHost) -> Result<DbHost, StoreError> {
        caller.require(Permission::HostsWrite).map_err(|_| StoreError::PermissionDenied)?;

        let id = host.id.ok_or_else(|| StoreError::NotFound)?;

        let collection = self
            .get_db()
            .collection::<DbHost>(MONGODB_COLLECTION_INVENTORY_HOSTS);

        let bson_updated_at = BsonDateTime::from_chrono(host.updated_at);
        let bson_last_seen_at = BsonDateTime::from_chrono(host.last_seen_at);

        collection
            .update_one(
                doc! {"_id": id},
                doc! {
                    "$set": {
                        "hostname": &host.hostname,
                        "architecture": &host.architecture,
                        "operating_system": {
                            "name": &host.operating_system.name,
                            "version": &host.operating_system.version,
                        },
                        "updated_at": bson_updated_at,
                        "last_seen_at": bson_last_seen_at,
                    }
                },
            )
            .await?;

        Ok(host)
    }

    #[instrument(skip(self), err(Debug))]
    async fn delete(&self, caller: Caller, id: String) -> Result<(), StoreError> {
        caller.require(Permission::HostsDelete).map_err(|_| StoreError::PermissionDenied)?;

        let object_id = ObjectId::from_str(&id).map_err(|e| StoreError::Internal(Box::new(e)))?;

        let collection = self
            .get_db()
            .collection::<DbHost>(MONGODB_COLLECTION_INVENTORY_HOSTS);

        collection.delete_one(doc! {"_id": object_id}).await?;

        Ok(())
    }
}
