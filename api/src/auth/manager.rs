use axum::http::request::Parts;
use lucid_common::caller::Caller;
use tracing::{debug, instrument, trace};

use super::{error::AuthError, provider::AuthProvider};

/// Coordinates multiple authentication providers in priority order.
///
/// The `AuthManager` tries each registered provider until one successfully
/// authenticates the request. Providers are tried in registration order.
///
/// # Flow
///
/// 1. Request comes in with some credentials (cookie, API key, etc.)
/// 2. AuthManager asks each provider if it can authenticate
/// 3. If provider returns `MissingCredentials`, try next provider
/// 4. If provider returns success, return the `Caller`
/// 5. If provider returns other error, fail immediately (stop trying)
///
/// # Examples
///
/// ```rust,ignore
/// let auth_manager = AuthManager::new()
///     .with_provider(SessionProvider::new(db.clone(), signing_key))
///     .with_provider(ApiKeyProvider::new(db.clone()));
///
/// // In extractor:
/// let caller = auth_manager.authenticate(&request_parts).await?;
/// ```
pub struct AuthManager {
    providers: Vec<Box<dyn AuthProvider>>,
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn with_provider<P: AuthProvider + 'static>(mut self, provider: P) -> Self {
        self.providers.push(Box::new(provider));
        self
    }

    /// Try each provider in order until one succeeds
    #[instrument(skip(self))]
    pub async fn authenticate(&self, parts: &Parts) -> Result<Caller, AuthError> {
        for provider in &self.providers {
            trace!(scheme = provider.scheme(), "Trying auth provider");

            match provider.authenticate(parts).await {
                Ok(caller) => {
                    debug!(scheme = provider.scheme(), "Auth succeeded");
                    return Ok(caller);
                }
                Err(AuthError::MissingCredentials) => {
                    trace!(scheme = provider.scheme(), "No credentials for this scheme");
                    continue;
                }
                Err(e) => {
                    debug!(scheme = provider.scheme(), error = %e, "Auth failed");
                    return Err(e);
                }
            }
        }

        Err(AuthError::MissingCredentials)
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}
