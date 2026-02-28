use chrono::Utc;
use lucid_common::views::ActivationKey;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::models::DbUlid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbActivationKey {
    #[serde(rename = "_id")]
    pub id: DbUlid,

    /// The unique identifier for this activation key (e.g., a random string or ULID)
    pub key_id: String,

    /// The description of this activation key (e.g., "Key for activating new hosts")
    pub description: String,

    /// The agent that used this activation key (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used_by_agent_id: Option<ObjectId>,
}

impl DbActivationKey {
    pub fn new(key_id: String, description: String) -> Self {
        Self::new_with_id(
            DbUlid::new(),
            key_id,
            description,
        )
    }

    pub fn new_with_id(id: DbUlid, key_id: String, description: String) -> Self {
        Self {
            id,
            key_id,
            description,
            used_by_agent_id: None,
        }
    }

    /// Get the creation time of this activation key based on the ObjectId's timestamp.
    pub fn created_at(&self) -> chrono::DateTime<Utc> {
        self.id.inner().datetime().into()
    }
}

impl From<DbActivationKey> for ActivationKey {
    fn from(value: DbActivationKey) -> Self {
        Self {
            id: value.id.clone().into(),
            key_id: value.key_id.clone(),
            description: value.description.clone(),
            used: value.used_by_agent_id.is_some(),
            created_at: value.created_at(),
        }
    }
}
