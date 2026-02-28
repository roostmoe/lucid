use chrono::{DateTime, Utc};
use bson::serde_helpers::datetime::FromChrono04DateTime;
use lucid_common::views::Host;
use serde::{Deserialize, Serialize};

use crate::models::DbUlid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbHost {
    #[serde(rename = "_id")]
    pub id: DbUlid,

    /// The hostname or IP address of the host
    pub hostname: String,

    /// The CPU architecture of the host (e.g., "x86_64", "arm64")
    pub architecture: String,

    /// The operating system information of the host
    pub operating_system: OperatingSystem,

    /// Reference to the agent for this host (optional for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<DbUlid>,

    /// When the host was last updated manually
    #[serde(with = "FromChrono04DateTime")]
    pub updated_at: DateTime<Utc>,

    /// Last time the host reported in
    #[serde(with = "FromChrono04DateTime")]
    pub last_seen_at: DateTime<Utc>,
}

impl From<DbHost> for Host {
    fn from(value: DbHost) -> Self {
        Self {
            id: value.id.clone().into(),
            hostname: value.hostname.clone(),
            os_id: value.operating_system.id.clone(),
            os_name: value.operating_system.name.clone(),
            os_version: value.operating_system.version.clone(),
            architecture: value.architecture.clone(),
            created_at: value.id.inner().datetime().into(),
            updated_at: value.updated_at,
            last_seen_at: value.last_seen_at,
            ifaces: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatingSystem {
    /// The ID of the operating system (e.g., "ubuntu", "rocky", "fedora")
    pub id: String,

    /// The name of the operating system (e.g., "Ubuntu", "Windows", "macOS")
    pub name: String,

    /// The version of the operating system (e.g., "20.04", "10", "11.2")
    pub version: String,
}
