use bson::serde_helpers::datetime::FromChrono04DateTime;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::DbUlid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbCa {
    #[serde(rename = "_id")]
    pub id: DbUlid,

    /// CA certificate in PEM format (plaintext)
    pub cert_pem: String,

    /// Encrypted private key: nonce (12 bytes) || ciphertext || tag (16 bytes)
    /// Stored as BSON Binary
    pub encrypted_private_key: Vec<u8>,

    /// When this CA was created
    #[serde(with = "FromChrono04DateTime")]
    pub created_at: DateTime<Utc>,
}
