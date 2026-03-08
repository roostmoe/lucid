use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Response body for a successful agent registration.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RegisterAgentResponse {
    /// Agent UUID (ObjectId as hex string)
    pub agent_id: String,
    /// Signed certificate in PEM format
    pub certificate_pem: String,
    /// CA certificate in PEM format
    pub ca_certificate_pem: String,
    /// Certificate expiration time
    #[serde(with = "chrono::serde::ts_seconds")]
    pub expires_at: DateTime<Utc>,
    /// API base URL for future requests
    pub api_base_url: String,
}
