//! mTLS client certificate authentication provider.
//!
//! Authenticates agents via client certificates presented during TLS handshake.

use std::{str::FromStr, sync::Arc};

use async_trait::async_trait;
use axum::http::request::Parts;
use chrono::{Duration, Utc};
use lucid_common::caller::{Caller, Role};
use lucid_db::storage::{AgentStore, Storage};
use tracing::{debug, instrument, warn};
use ulid::Ulid;
use x509_parser::prelude::*;

use crate::auth::{error::AuthError, provider::AuthProvider};

/// Authentication provider for mTLS client certificates.
///
/// Validates agent client certificates and extracts agent identity from the CN.
pub struct MtlsAuthProvider {
    db: Arc<dyn Storage>,
}

impl MtlsAuthProvider {
    pub fn new(db: Arc<dyn Storage>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AuthProvider for MtlsAuthProvider {
    fn scheme(&self) -> &'static str {
        "mtls"
    }

    #[instrument(skip(self, parts), fields(scheme = "mtls"))]
    async fn authenticate(&self, parts: &Parts) -> Result<Caller, AuthError> {
        // 1. Extract client certificate from request extensions
        // The certificate is inserted by rustls/axum-server as Vec<rustls::pki_types::CertificateDer>
        let certs: &Vec<rustls::pki_types::CertificateDer> = parts
            .extensions
            .get()
            .ok_or(AuthError::MissingCredentials)?;

        if certs.is_empty() {
            return Err(AuthError::MissingCredentials);
        }

        let cert_der = &certs[0];
        debug!("Client certificate found, parsing...");

        // 2. Parse the certificate
        let (_, cert) = X509Certificate::from_der(cert_der.as_ref()).map_err(|e| {
            warn!("Failed to parse client certificate: {}", e);
            AuthError::InvalidCredentials
        })?;

        // 3. Check validity period with 5-minute clock skew grace
        let now = Utc::now();
        let grace = Duration::minutes(5);

        // Convert time::OffsetDateTime to chrono::DateTime
        let not_before_unix = cert.validity().not_before.to_datetime().unix_timestamp();
        let not_after_unix = cert.validity().not_after.to_datetime().unix_timestamp();

        let not_before = chrono::DateTime::from_timestamp(not_before_unix, 0).ok_or_else(|| {
            warn!("Invalid not_before timestamp");
            AuthError::InvalidCredentials
        })?;
        let not_after = chrono::DateTime::from_timestamp(not_after_unix, 0).ok_or_else(|| {
            warn!("Invalid not_after timestamp");
            AuthError::InvalidCredentials
        })?;

        if now + grace < not_before {
            warn!("Certificate not yet valid");
            return Err(AuthError::InvalidCredentials);
        }
        if now - grace > not_after {
            warn!("Certificate expired");
            return Err(AuthError::Expired);
        }

        // 4. Extract agent ID from CN
        let cn = cert
            .subject()
            .iter_common_name()
            .next()
            .and_then(|attr| attr.as_str().ok())
            .ok_or_else(|| {
                warn!("Certificate missing CN");
                AuthError::InvalidCredentials
            })?;

        debug!(cn = %cn, "Extracted CN from certificate");

        // 5. Parse CN as ObjectId (agent ID)
        let agent_id = Ulid::from_str(cn).map_err(|e| {
            warn!("Invalid agent ID in CN: {}", e);
            AuthError::InvalidCredentials
        })?;

        // 6. Look up agent in database
        let agent = AgentStore::get(&*self.db, agent_id.into())
            .await
            .map_err(|e| {
                warn!("Database error looking up agent: {}", e);
                AuthError::Internal(e.to_string())
            })?
            .ok_or_else(|| {
                warn!("Agent not found: {}", agent_id);
                AuthError::InvalidCredentials
            })?;

        // 7. Check not revoked
        if agent.revoked_at.is_some() {
            warn!(agent_id = %agent_id, "Agent is revoked");
            return Err(AuthError::InvalidCredentials);
        }

        // 8. Verify certificate matches stored certificate
        // Convert presented cert to PEM for comparison
        let presented_pem = pem_rfc7468::encode_string(
            "CERTIFICATE",
            pem_rfc7468::LineEnding::LF,
            cert_der.as_ref(),
        )
        .map_err(|e| {
            warn!("Failed to encode certificate as PEM: {}", e);
            AuthError::Internal(e.to_string())
        })?;

        // Normalize whitespace for comparison
        let stored_normalized: String = agent
            .certificate_pem
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        let presented_normalized: String = presented_pem
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();

        if stored_normalized != presented_normalized {
            warn!(agent_id = %agent_id, "Certificate mismatch");
            return Err(AuthError::InvalidCredentials);
        }

        // 9. Update last_seen_at
        if let Err(e) = AgentStore::update_last_seen(&*self.db, agent.id).await {
            warn!("Failed to update last_seen_at: {}", e);
            // Don't fail auth for this
        }

        debug!(agent_id = %agent_id, agent_name = %agent.name, "Agent authenticated successfully");

        // 10. Return Caller::Agent
        Ok(Caller::Agent {
            id: agent_id.to_string(),
            name: agent.name,
            roles: vec![Role::Agent],
        })
    }
}
