use axum::http::request::Parts;
use lucid_common::caller::Caller;
use tracing::{debug, trace};

use super::{error::AuthError, provider::AuthProvider};

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
    pub async fn authenticate(&self, parts: &Parts) -> Caller {
        for provider in &self.providers {
            trace!(scheme = provider.scheme(), "Trying auth provider");

            match provider.authenticate(parts).await {
                Ok(caller) => {
                    debug!(scheme = provider.scheme(), "Auth succeeded");
                    return caller;
                }
                Err(AuthError::MissingCredentials) => {
                    // This provider doesn't apply, try next
                    trace!(scheme = provider.scheme(), "No credentials for this scheme");
                    continue;
                }
                Err(e) => {
                    // Auth was attempted but failed
                    debug!(scheme = provider.scheme(), error = %e, "Auth failed");
                    return Caller::Unauthenticated;
                }
            }
        }

        // No provider had credentials
        Caller::Unauthenticated
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}
