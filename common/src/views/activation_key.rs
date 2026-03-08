use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use utoipa::ToSchema;

/// An activation key used to bootstrap host agent configuration.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ActivationKey {
    /// Internal database ID
    pub id: Ulid,
    /// User-provided key identifier
    pub key_id: String,
    /// Human-readable description
    pub description: String,
    /// Whether or not the key has been used to register an agent
    pub used: bool,
    /// When the key was created
    pub created_at: DateTime<Utc>,
}
