//! Activation key JWT authentication provider.
//!
//! This provider validates Bearer tokens that are activation key JWTs, issued
//! when an activation key is created. These JWTs are single-use tokens for
//! agent registration.

use std::sync::Arc;

use async_trait::async_trait;
use axum::http::{header, request::Parts};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use lucid_common::caller::Caller;
use lucid_db::storage::{ActivationKeyStore, Storage};
use tracing::{debug, instrument};

use crate::auth::{
    error::AuthError,
    jwt::ActivationKeyClaims,
    provider::AuthProvider,
    signing::{Ed25519Signer, SessionSigner},
};

/// Authentication provider for activation key JWTs.
///
/// Validates Bearer tokens that contain activation key JWTs and returns
/// a System caller for registration. The activation key ID is stored in
/// request extensions for the handler to consume.
pub struct ActivationKeyAuthProvider {
    db: Arc<dyn Storage>,
    public_url: String,
    session_signer: SessionSigner<Ed25519Signer>,
}

impl ActivationKeyAuthProvider {
    pub fn new(
        db: Arc<dyn Storage>,
        public_url: String,
        session_signer: SessionSigner<Ed25519Signer>,
    ) -> Self {
        Self {
            db,
            public_url,
            session_signer,
        }
    }

    /// Extract Bearer token from Authorization header
    fn extract_bearer_token(headers: &header::HeaderMap) -> Option<String> {
        headers
            .get(header::AUTHORIZATION)?
            .to_str()
            .ok()?
            .strip_prefix("Bearer ")
            .map(|s| s.to_string())
    }
}

#[async_trait]
impl AuthProvider for ActivationKeyAuthProvider {
    #[instrument(skip(self, parts), fields(scheme = "activation-key"))]
    async fn authenticate(&self, parts: &Parts) -> Result<Caller, AuthError> {
        // 1. Extract Bearer token
        let token =
            Self::extract_bearer_token(&parts.headers).ok_or(AuthError::MissingCredentials)?;

        debug!("Found Bearer token, decoding JWT...");

        // 2. Decode and verify JWT
        let public_key_bytes = self.session_signer.inner().public_key_bytes();
        let decoding_key = DecodingKey::from_ed_der(public_key_bytes);
        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.validate_exp = false; // No expiration in activation key tokens
        validation.required_spec_claims.clear();
        validation.set_issuer(&[&self.public_url]);

        let token_data = decode::<ActivationKeyClaims>(&token, &decoding_key, &validation)
            .map_err(|e| {
                debug!("JWT decode failed: {}", e);
                AuthError::InvalidCredentials
            })?;

        let claims = token_data.claims;
        debug!(ak = %claims.ak, "JWT decoded successfully");

        // 3. Look up activation key in DB
        let activation_key = ActivationKeyStore::get_by_internal_id(&*self.db, &claims.ak)
            .await?
            .ok_or_else(|| {
                debug!("Activation key not found");
                AuthError::InvalidCredentials
            })?;

        debug!(key_id = %activation_key.key_id, "Found activation key");

        // 4. Check if already used
        if activation_key.used_by_agent_id.is_some() {
            debug!("Activation key already used");
            return Err(AuthError::InvalidCredentials);
        }

        debug!("Activation key valid and unused");

        // 5. Store activation key ID in extensions for handler to retrieve
        // We can't modify parts here, so we'll return System caller
        // The handler will need to re-decode the JWT to get the activation key ID
        // (This is a limitation of the current auth architecture)
        Ok(Caller::System)
    }

    fn scheme(&self) -> &'static str {
        "activation-key"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::header::HeaderMap;

    #[test]
    fn test_extract_bearer_token_valid() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            "Bearer test_token_123".parse().unwrap(),
        );

        let result = ActivationKeyAuthProvider::extract_bearer_token(&headers);
        assert_eq!(result, Some("test_token_123".to_string()));
    }

    #[test]
    fn test_extract_bearer_token_missing_header() {
        let headers = HeaderMap::new();
        let result = ActivationKeyAuthProvider::extract_bearer_token(&headers);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_bearer_token_wrong_scheme() {
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, "Basic dXNlcjpwYXNz".parse().unwrap());

        let result = ActivationKeyAuthProvider::extract_bearer_token(&headers);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_bearer_token_no_token() {
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, "Bearer ".parse().unwrap());

        let result = ActivationKeyAuthProvider::extract_bearer_token(&headers);
        assert_eq!(result, Some("".to_string()));
    }

    #[test]
    fn test_extract_bearer_token_with_spaces() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            "Bearer   token_with_leading_spaces".parse().unwrap(),
        );

        // strip_prefix only removes "Bearer ", so extra spaces remain
        let result = ActivationKeyAuthProvider::extract_bearer_token(&headers);
        assert_eq!(result, Some("  token_with_leading_spaces".to_string()));
    }
}
