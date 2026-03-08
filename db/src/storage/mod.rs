use std::fmt::Debug;

use ::mongodb::bson::oid::ObjectId;
use async_trait::async_trait;
use chrono::Duration;
use lucid_common::{
    caller::Caller,
    params::{CreateLocalUserParams, PaginationParams},
};
use thiserror::Error;

use crate::models::{DbActivationKey, DbAgent, DbCa, DbHost, DbSession, DbUlid, DbUser};

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
    UserStore
    + SessionStore
    + HostStore
    + ActivationKeyStore
    + AgentStore
    + CaStore
    + Send
    + Sync
    + 'static
{
    async fn ping(&self) -> Result<(), StoreError>;
}

#[derive(Debug, Default)]
pub struct UserFilter {
    pub id: Option<Vec<DbUlid>>,
    pub email: Option<Vec<String>>,
}

#[async_trait]
pub trait UserStore {
    async fn get(&self, caller: Caller, id: DbUlid) -> Result<Option<DbUser>, StoreError>;
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
        user_id: DbUlid,
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
    pub id: Option<Vec<DbUlid>>,
    pub arch: Option<Vec<String>>,
    pub os_name: Option<Vec<String>>,
    pub os_version: Option<Vec<String>>,
    pub hostname: Option<Vec<String>>,
}

#[async_trait]
pub trait HostStore {
    async fn get(&self, caller: Caller, id: DbUlid) -> Result<Option<DbHost>, StoreError>;
    async fn list(
        &self,
        caller: Caller,
        filter: HostFilter,
        pagination: PaginationParams,
    ) -> Result<Vec<DbHost>, StoreError>;
    async fn create(&self, caller: Caller, host: DbHost) -> Result<DbHost, StoreError>;
    async fn update(&self, caller: Caller, host: DbHost) -> Result<DbHost, StoreError>;
    async fn delete(&self, caller: Caller, id: DbUlid) -> Result<(), StoreError>;
}

#[derive(Debug, Default)]
pub struct ActivationKeyFilter {
    pub id: Option<Vec<DbUlid>>,
    pub key_id: Option<Vec<String>>,
}

#[async_trait]
pub trait ActivationKeyStore {
    async fn get(&self, caller: Caller, id: DbUlid) -> Result<Option<DbActivationKey>, StoreError>;
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
    async fn delete(&self, caller: Caller, id: DbUlid) -> Result<(), StoreError>;
    async fn mark_as_used(&self, key_id: DbUlid, agent_id: DbUlid) -> Result<(), StoreError>;
    async fn is_used(&self, key_id: DbUlid) -> Result<bool, StoreError>;
    async fn get_by_internal_id(
        &self,
        internal_id: &str,
    ) -> Result<Option<DbActivationKey>, StoreError>;
}

#[async_trait]
pub trait AgentStore {
    async fn create(&self, agent: DbAgent) -> Result<DbAgent, StoreError>;
    async fn get(&self, id: DbUlid) -> Result<Option<DbAgent>, StoreError>;
    async fn get_by_public_key(&self, public_key_pem: &str) -> Result<Option<DbAgent>, StoreError>;
    async fn update(&self, agent: DbAgent) -> Result<DbAgent, StoreError>;
    async fn update_last_seen(&self, id: DbUlid) -> Result<(), StoreError>;
    async fn soft_delete(&self, id: DbUlid) -> Result<(), StoreError>;
    async fn hard_delete(&self, id: DbUlid) -> Result<(), StoreError>;
}

#[async_trait]
pub trait CaStore: Send + Sync {
    async fn get(&self, caller: Caller, id: DbUlid) -> Result<Option<DbCa>, StoreError>;
    async fn list(&self, caller: Caller) -> Result<Vec<DbCa>, StoreError>;
    async fn create(&self, caller: Caller, ca: DbCa) -> Result<DbCa, StoreError>;
    async fn delete(&self, caller: Caller, id: DbUlid) -> Result<(), StoreError>;
}
