use std::fmt::Debug;

use async_trait::async_trait;
use lucid_common::{
    caller::Caller,
    params::{CreateLocalUserParams, PaginationParams},
};
use thiserror::Error;

use crate::models::DbUser;

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
pub trait Storage: UserStore + Send + Sync + 'static {
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
