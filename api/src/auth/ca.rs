use async_trait::async_trait;
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::Serialize;
use utoipa::ToSchema;

#[async_trait]
pub trait CertificateAuthority: Send + Sync {
    /// Sign a CSR and return a PEM-encoded certificate valid for 24 hours.
    /// The CN is set to the agent_id.
    async fn sign_csr(
        &self,
        csr_pem: &str,
        agent_id: ObjectId,
    ) -> Result<SignedCertificate, CaError>;

    /// Get the CA certificate in PEM format.
    async fn get_ca_cert_pem(&self) -> Result<String, CaError>;

    /// Get CA certificate metadata for the well-known endpoint.
    async fn get_ca_info(&self) -> Result<CaInfo, CaError>;
}

#[derive(Debug, Clone)]
pub struct SignedCertificate {
    pub cert_pem: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CaInfo {
    pub cert_pem: String,
    pub fingerprint: String, // sha256:hex
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum CaError {
    #[error("CA not initialized - run `lucid-api generate-ca` first")]
    NotInitialized,
    #[error("Invalid CSR: {0}")]
    InvalidCsr(String),
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("Decryption error: {0}")]
    Decryption(String),
    #[error("Certificate generation error: {0}")]
    Generation(String),
    #[error("Storage error: {0}")]
    Storage(String),
}
