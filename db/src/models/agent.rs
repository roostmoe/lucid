use bson::serde_helpers::datetime::FromChrono04DateTime;
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbAgent {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// Agent name (typically hostname)
    pub name: String,

    /// Foreign key to DbHost (1:1 relationship)
    pub host_id: ObjectId,

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
