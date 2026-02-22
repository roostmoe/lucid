use std::fmt::Debug;

use ::mongodb::bson::oid::ObjectId;
use async_trait::async_trait;
use chrono::Duration;
use lucid_common::{
    caller::Caller,
    params::{CreateLocalUserParams, PaginationParams},
};
use thiserror::Error;

use crate::models::{DbSession, DbUser};

pub mod mongodb;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("Resource not found")]
    NotFound,

    #[error("Query Error: {0}")]
    MongoDB(#[from] ::mongodb::error::Error),

    #[error(transparent)]
    Internal(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error(transparent)]
    InternalAnyhow(#[from] anyhow::Error),
}

#[async_trait]
pub trait Storage: UserStore + SessionStore + Send + Sync + 'static {
    async fn ping(&self) -> Result<(), StoreError>;
}

#[derive(Debug, Default)]
pub struct UserFilter {
    pub id: Option<Vec<String>>,
    pub email: Option<Vec<String>>,
}

#[async_trait]
pub trait UserStore {
    async fn get(&self, id: String) -> Result<Option<DbUser>, StoreError>;
    async fn list(
        &self,
        filter: UserFilter,
        pagination: PaginationParams,
    ) -> Result<Vec<DbUser>, StoreError>;

    async fn create_local(&self, user: CreateLocalUserParams) -> Result<DbUser, StoreError>;
    async fn auth_local(&self, email: String, password: String) -> Result<Caller, StoreError>;
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
