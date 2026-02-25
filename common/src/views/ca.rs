use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A certificate authority managed by Lucid.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Ca {
    /// Internal database ID
    pub id: String,
    /// CA certificate in PEM format
    pub cert_pem: String,
    /// SHA-256 fingerprint of the certificate (format: `sha256:<hex>`)
    pub fingerprint: String,
    /// When this CA was created
    pub created_at: DateTime<Utc>,
}
