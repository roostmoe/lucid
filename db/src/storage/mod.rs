use std::fmt::Debug;

use ::mongodb::bson::oid::ObjectId;
use async_trait::async_trait;
use chrono::Duration;
use lucid_common::{
    caller::Caller,
    params::{CreateLocalUserParams, PaginationParams},
};
use thiserror::Error;

use crate::models::{DbActivationKey, DbHost, DbSession, DbUser};

pub mod mongodb;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("Resource not found")]
    NotFound,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Query Error: {0}")]
    MongoDB(#[from] ::mongodb::error::Error),

    #[error(transparent)]
    Internal(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error(transparent)]
    InternalAnyhow(#[from] anyhow::Error),
}

#[async_trait]
pub trait Storage:
    UserStore + SessionStore + HostStore + ActivationKeyStore + Send + Sync + 'static
{
    async fn ping(&self) -> Result<(), StoreError>;
}

#[derive(Debug, Default)]
pub struct UserFilter {
    pub id: Option<Vec<String>>,
    pub email: Option<Vec<String>>,
}

#[async_trait]
pub trait UserStore {
    async fn get(&self, caller: Caller, id: String) -> Result<Option<DbUser>, StoreError>;
    async fn list(
        &self,
        caller: Caller,
        filter: UserFilter,
        pagination: PaginationParams,
    ) -> Result<Vec<DbUser>, StoreError>;

    async fn create_local(
        &self,
        caller: Caller,
        user: CreateLocalUserParams,
    ) -> Result<DbUser, StoreError>;
    async fn auth_local(
        &self,
        caller: Caller,
        email: String,
        password: String,
    ) -> Result<Caller, StoreError>;
}

#[async_trait]
pub trait SessionStore {
    /// Create a new session for a user
    async fn create_session(
        &self,
        user_id: ObjectId,
        session_id: String,
        csrf_token: String,
        ttl: Duration,
    ) -> Result<DbSession, StoreError>;

    /// Get a session by its session_id
    async fn get_session(&self, session_id: &str) -> Result<Option<DbSession>, StoreError>;

    /// Delete a session by its session_id
    async fn delete_session(&self, session_id: &str) -> Result<(), StoreError>;

    /// Update the last_used_at timestamp
    async fn touch_session(&self, session_id: &str) -> Result<(), StoreError>;

    /// Delete all expired sessions (cleanup job)
    async fn cleanup_expired_sessions(&self) -> Result<u64, StoreError>;

    /// Delete all sessions for a user (logout everywhere)
    async fn delete_user_sessions(&self, user_id: ObjectId) -> Result<u64, StoreError>;
}

#[derive(Debug, Default)]
pub struct HostFilter {
    pub id: Option<Vec<String>>,
    pub arch: Option<Vec<String>>,
    pub os_name: Option<Vec<String>>,
    pub os_version: Option<Vec<String>>,
    pub hostname: Option<Vec<String>>,
}

#[async_trait]
pub trait HostStore {
    async fn get(&self, caller: Caller, id: String) -> Result<Option<DbHost>, StoreError>;
    async fn list(
        &self,
        caller: Caller,
        filter: HostFilter,
        pagination: PaginationParams,
    ) -> Result<Vec<DbHost>, StoreError>;
    async fn create(&self, caller: Caller, host: DbHost) -> Result<DbHost, StoreError>;
    async fn update(&self, caller: Caller, host: DbHost) -> Result<DbHost, StoreError>;
    async fn delete(&self, caller: Caller, id: String) -> Result<(), StoreError>;
}

#[derive(Debug, Default)]
pub struct ActivationKeyFilter {
    pub id: Option<Vec<String>>,
    pub key_id: Option<Vec<String>>,
}

#[async_trait]
pub trait ActivationKeyStore {
    async fn get(&self, caller: Caller, id: String) -> Result<Option<DbActivationKey>, StoreError>;
    async fn list(
        &self,
        caller: Caller,
        filter: ActivationKeyFilter,
        pagination: PaginationParams,
    ) -> Result<Vec<DbActivationKey>, StoreError>;
    async fn create(
        &self,
        caller: Caller,
        key: DbActivationKey,
    ) -> Result<DbActivationKey, StoreError>;
    async fn delete(&self, caller: Caller, id: String) -> Result<(), StoreError>;
}
