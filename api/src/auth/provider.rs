use async_trait::async_trait;
use axum::http::request::Parts;
use lucid_common::caller::Caller;

use super::error::AuthError;

#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Attempt to authenticate from request parts
    /// Returns Ok(Caller) on success, Err on failure
    async fn authenticate(&self, parts: &Parts) -> Result<Caller, AuthError>;

    /// Name of this auth scheme (for debugging/logging)
    fn scheme(&self) -> &'static str;
}
