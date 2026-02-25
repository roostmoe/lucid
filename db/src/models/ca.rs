use bson::serde_helpers::datetime::FromChrono04DateTime;
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbCa {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// CA certificate in PEM format (plaintext)
    pub cert_pem: String,

    /// Encrypted private key: nonce (12 bytes) || ciphertext || tag (16 bytes)
    /// Stored as BSON Binary
    pub encrypted_private_key: Vec<u8>,

    /// When this CA was created
    #[serde(with = "FromChrono04DateTime")]
    pub created_at: DateTime<Utc>,
}
