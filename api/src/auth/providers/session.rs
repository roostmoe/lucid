//! Session-based authentication provider.
//!
//! This module implements authentication via signed session cookies. It's designed for
//! web console users and provides CSRF protection for state-changing requests.
//!
//! # Cookie-Based Sessions
//!
//! Session tokens are stored in HttpOnly cookies named `lucid_session`. The cookie value
//! is a signed token in the format: `{session_id}.{ed25519_signature}`
//!
//! See `docs/API_SESSIONS.adoc` for complete API documentation.
//!
//! # CSRF Protection
//!
//! For mutating requests (POST, PUT, PATCH, DELETE), the provider requires a CSRF token
//! in the `X-CSRF-Token` header. This token is returned by the login endpoint and must
//! be stored by the client.
//!
//! Read-only requests (GET, HEAD, OPTIONS) do not require the CSRF token.
//!
//! # Authentication Flow
//!
//! 1. Extract `lucid_session` cookie from request
//! 2. Verify Ed25519 signature on cookie value
//! 3. Extract session ID from signed token
//! 4. Fetch session from database, check expiry
//! 5. For mutating requests: validate CSRF token against session's stored token
//! 6. Fetch user from database
//! 7. Update session's `last_used_at` timestamp (sliding expiry)
//! 8. Return authenticated `Caller::User`
//!
//! # Example
//!
//! ```no_run
//! use std::sync::Arc;
//! use lucid_api::auth::{SessionSigner, signing::Ed25519Signer};
//! use lucid_api::auth::providers::session::SessionAuthProvider;
//! use lucid_db::storage::Storage;
//!
//! # async fn example(db: Arc<dyn Storage>) -> Result<(), Box<dyn std::error::Error>> {
//! // Load signing key
//! let pem = std::fs::read_to_string("signing_key.pem")?;
//! let ed25519 = Ed25519Signer::from_pem(&pem)?;
//! let signer = SessionSigner::new(ed25519);
//!
//! // Create provider
//! let provider = SessionAuthProvider::new(signer, db);
//!
//! // Provider is used by AuthManager to authenticate requests
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;

use async_trait::async_trait;
use axum::http::{Method, header, request::Parts};
use lucid_common::caller::Caller;
use lucid_db::storage::{SessionStore, Storage, UserStore};
use tracing::{info, instrument};

use crate::auth::{
    error::AuthError,
    provider::AuthProvider,
    signing::{SessionSigner, Signer},
};

const COOKIE_NAME: &str = "lucid_session";
const CSRF_HEADER: &str = "X-CSRF-Token";

/// Authentication provider for session-based auth.
///
/// Authenticates users via signed session cookies with optional CSRF protection.
///
/// # Cookie Security
///
/// - HttpOnly: JavaScript cannot access the cookie (XSS protection)
/// - SameSite=Lax: Cookie not sent on cross-site POST (baseline CSRF protection)
/// - Secure: Cookie only sent over HTTPS in production
/// - Max-Age: 30 days (sliding expiry via `touch_session`)
///
/// # CSRF Protection
///
/// Mutating requests (POST/PUT/PATCH/DELETE) require the `X-CSRF-Token` header.
/// The CSRF token is returned by the login endpoint and stored in the session.
///
/// Read-only requests (GET/HEAD/OPTIONS) do not require CSRF validation.
pub struct SessionAuthProvider<S: Signer> {
    signer: SessionSigner<S>,
    db: Arc<dyn Storage>,
}

impl<S: Signer> SessionAuthProvider<S> {
    pub fn new(signer: SessionSigner<S>, db: Arc<dyn Storage>) -> Self {
        Self { signer, db }
    }

    /// Sign a session ID: returns "session_id.signature"
    pub fn sign(&self, session_id: &str) -> Result<String, crate::auth::signing::SigningError> {
        self.signer.sign(session_id)
    }

    /// Verify a signed session ID, returns the session_id if valid
    pub fn verify(&self, signed: &str) -> Option<String> {
        self.signer.verify(signed)
    }

    /// Extract cookie value from Cookie header
    fn extract_cookie(headers: &header::HeaderMap, name: &str) -> Option<String> {
        headers
            .get(header::COOKIE)?
            .to_str()
            .ok()?
            .split(';')
            .map(|s| s.trim())
            .find(|s| s.starts_with(&format!("{}=", name)))?
            .strip_prefix(&format!("{}=", name))
            .map(|s| s.to_string())
    }

    /// Check if this method requires CSRF validation
    fn requires_csrf(method: &Method) -> bool {
        !matches!(method, &Method::GET | &Method::HEAD | &Method::OPTIONS)
    }
}

#[async_trait]
impl<S: Signer> AuthProvider for SessionAuthProvider<S> {
    #[instrument(skip(self, parts), fields(scheme = "session"))]
    async fn authenticate(&self, parts: &Parts) -> Result<Caller, AuthError> {
        // 1. Extract session cookie
        let signed_cookie = Self::extract_cookie(&parts.headers, COOKIE_NAME)
            .ok_or(AuthError::MissingCredentials)?;

        info!(?signed_cookie, "Found session cookie, verifying...");

        // 2. Verify signature
        let session_id = self
            .verify(&signed_cookie)
            .ok_or(AuthError::InvalidCredentials)?;

        info!(?session_id, "Found session ID, loading from DB...");

        // 3. Fetch session from DB
        let session = SessionStore::get_session(&*self.db, &session_id)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        info!(?session, "Found session, checking expiry...");

        // 4. Check expiry
        if session.expires_at < chrono::Utc::now() {
            return Err(AuthError::Expired);
        }

        info!("Session valid, checking CSRF...");

        // 5. Validate CSRF for mutating requests
        if Self::requires_csrf(&parts.method) {
            let csrf_token = parts
                .headers
                .get(CSRF_HEADER)
                .and_then(|v| v.to_str().ok())
                .ok_or(AuthError::CsrfFailed)?;

            if csrf_token != session.csrf_token {
                return Err(AuthError::CsrfFailed);
            }
        }

        info!(?session.user_id, "CSRF check passed, loading user...");

        // 6. Fetch user
        let user = UserStore::get(&*self.db, Caller::System, session.user_id.to_string())
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        info!(user_id = ?user.id, "User authenticated successfully");

        // 7. Touch session (update last_used_at) - fire and forget
        let _ = SessionStore::touch_session(&*self.db, &session_id).await;

        // 8. Return authenticated caller
        Ok(user.to_caller())
    }

    fn scheme(&self) -> &'static str {
        "session"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::signing::Ed25519Signer;

    // Helper type for tests
    type TestSessionAuthProvider = SessionAuthProvider<Ed25519Signer>;

    #[test]
    fn test_extract_cookie_basic() {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::COOKIE,
            "foo=bar; lucid_session=abc123; other=value"
                .parse()
                .unwrap(),
        );

        let result = TestSessionAuthProvider::extract_cookie(&headers, "lucid_session");
        assert_eq!(result, Some("abc123".to_string()));

        let result = TestSessionAuthProvider::extract_cookie(&headers, "foo");
        assert_eq!(result, Some("bar".to_string()));

        let result = TestSessionAuthProvider::extract_cookie(&headers, "other");
        assert_eq!(result, Some("value".to_string()));
    }

    #[test]
    fn test_extract_cookie_not_found() {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::COOKIE, "foo=bar".parse().unwrap());

        let result = TestSessionAuthProvider::extract_cookie(&headers, "nonexistent");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_cookie_no_cookie_header() {
        let headers = header::HeaderMap::new();
        let result = TestSessionAuthProvider::extract_cookie(&headers, "anything");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_cookie_with_spaces() {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::COOKIE,
            "cookie1=value1;  cookie2=value2  ;cookie3=value3"
                .parse()
                .unwrap(),
        );

        let result = TestSessionAuthProvider::extract_cookie(&headers, "cookie2");
        assert_eq!(result, Some("value2".to_string()));
    }

    #[test]
    fn test_extract_cookie_empty_value() {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::COOKIE, "empty=; other=value".parse().unwrap());

        let result = TestSessionAuthProvider::extract_cookie(&headers, "empty");
        assert_eq!(result, Some("".to_string()));
    }

    #[test]
    fn test_extract_cookie_similar_names() {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::COOKIE,
            "session=old; lucid_session=new".parse().unwrap(),
        );

        let result = TestSessionAuthProvider::extract_cookie(&headers, "session");
        assert_eq!(result, Some("old".to_string()));

        let result = TestSessionAuthProvider::extract_cookie(&headers, "lucid_session");
        assert_eq!(result, Some("new".to_string()));
    }

    #[test]
    fn test_requires_csrf_safe_methods() {
        assert!(!TestSessionAuthProvider::requires_csrf(&Method::GET));
        assert!(!TestSessionAuthProvider::requires_csrf(&Method::HEAD));
        assert!(!TestSessionAuthProvider::requires_csrf(&Method::OPTIONS));
    }

    #[test]
    fn test_requires_csrf_mutating_methods() {
        assert!(TestSessionAuthProvider::requires_csrf(&Method::POST));
        assert!(TestSessionAuthProvider::requires_csrf(&Method::PUT));
        assert!(TestSessionAuthProvider::requires_csrf(&Method::PATCH));
        assert!(TestSessionAuthProvider::requires_csrf(&Method::DELETE));
    }

    #[test]
    fn test_requires_csrf_trace_method() {
        // TRACE should also require CSRF
        assert!(TestSessionAuthProvider::requires_csrf(&Method::TRACE));
    }
}
