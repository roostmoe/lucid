use async_trait::async_trait;
use axum::http::request::Parts;
use lucid_common::caller::Caller;

use super::error::AuthError;

/// Authentication provider that can extract and verify credentials.
///
/// Implement this trait to add new authentication methods to Lucid.
/// Providers are registered with `AuthManager` and tried in order.
///
/// # Implementing a Provider
///
/// Your provider should:
///
/// 1. Check if the request contains credentials for this auth scheme
/// 2. If not, return `Err(AuthError::MissingCredentials)` so other providers can try
/// 3. If yes, verify the credentials
/// 4. On success, return `Ok(Caller)`
/// 5. On failure, return an appropriate `AuthError`
///
/// # Examples
///
/// ```rust,ignore
/// use async_trait::async_trait;
/// use axum::http::request::Parts;
/// use lucid_common::caller::{Caller, Role};
/// use crate::auth::{AuthProvider, AuthError};
///
/// pub struct ApiKeyProvider {
///     db: Database,
/// }
///
/// #[async_trait]
/// impl AuthProvider for ApiKeyProvider {
///     fn scheme(&self) -> &'static str {
///         "api-key"
///     }
///
///     async fn authenticate(&self, parts: &Parts) -> Result<Caller, AuthError> {
///         // 1. Check for API key in Authorization header
///         let api_key = parts
///             .headers
///             .get("authorization")
///             .and_then(|v| v.to_str().ok())
///             .and_then(|v| v.strip_prefix("Bearer "))
///             .ok_or(AuthError::MissingCredentials)?;  // ← no API key, try next provider
///
///         // 2. Look up service account
///         let service_account = self.db
///             .find_service_account_by_key(api_key)
///             .await
///             .map_err(|_| AuthError::InvalidCredentials)?;  // ← bad key, fail
///
///         // 3. Return caller
///         Ok(Caller::ServiceAccount {
///             id: service_account.id,
///             name: service_account.name,
///             description: service_account.description,
///             roles: service_account.roles,
///         })
///     }
/// }
/// ```
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Attempt to authenticate from request parts.
    ///
    /// Return `Err(AuthError::MissingCredentials)` if this request doesn't
    /// contain credentials for your scheme. Return other errors if credentials
    /// are present but invalid.
    async fn authenticate(&self, parts: &Parts) -> Result<Caller, AuthError>;

    /// Name of this auth scheme for debugging/logging.
    ///
    /// Examples: "session", "api-key", "mtls"
    fn scheme(&self) -> &'static str;
}
