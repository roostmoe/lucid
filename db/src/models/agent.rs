use bson::serde_helpers::datetime::FromChrono04DateTime;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::DbUlid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbAgent {
    #[serde(rename = "_id")]
    pub id: DbUlid,

    /// Agent name (typically hostname)
    pub name: String,

    /// Foreign key to DbHost (1:1 relationship)
    pub host_id: DbUlid,

    /// Ed25519 public key in PEM format
    pub public_key_pem: String,

    /// Current signed certificate in PEM format
    pub certificate_pem: String,

    /// When the certificate was issued
    #[serde(with = "FromChrono04DateTime")]
    pub cert_issued_at: DateTime<Utc>,

    /// When the certificate expires (24h after issued_at)
    #[serde(with = "FromChrono04DateTime")]
    pub cert_expires_at: DateTime<Utc>,

    /// Last time the agent successfully authenticated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen_at: Option<DateTime<Utc>>,

    /// Set when agent is soft-deleted (revoked)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<DateTime<Utc>>,

    /// When this agent was created
    #[serde(with = "FromChrono04DateTime")]
    pub created_at: DateTime<Utc>,

    /// When this agent was last updated
    #[serde(with = "FromChrono04DateTime")]
    pub updated_at: DateTime<Utc>,
}

impl DbAgent {
    pub fn new(name: String, host_id: DbUlid, public_key_pem: String, certificate_pem: String) -> Self {
        Self::new_with_id(DbUlid::new(), name, host_id, public_key_pem, certificate_pem)
    }

    pub fn new_with_id(id: DbUlid, name: String, host_id: DbUlid, public_key_pem: String, certificate_pem: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            host_id,
            public_key_pem,
            certificate_pem,
            cert_issued_at: now,
            cert_expires_at: now + chrono::Duration::hours(24),
            last_seen_at: None,
            revoked_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}
