use std::fmt::Display;

use bson::serde_helpers::datetime::FromChrono04DateTime;
use chrono::{DateTime, Utc};
use lucid_common::{
    caller::{Caller, Role},
    views::{Host, User},
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbUser {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub display_name: String,
    pub email: String,
    pub password_hash: Option<String>,
    #[serde(with = "FromChrono04DateTime")]
    pub updated_at: DateTime<Utc>,
}

impl Display for DbUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DbUser {{ id: {:?}, display_name: {}, email: {} }}",
            self.id, self.display_name, self.email
        )
    }
}

impl DbUser {
    /// Get the creation time of this user based on the ULID's timestamp.
    pub fn created_at(&self) -> DateTime<Utc> {
        if let Some(id) = self.id {
            id.timestamp().to_system_time().into()
        } else {
            Utc::now()
        }
    }

    /// Convert this database user into a Caller for permission checking.
    ///
    /// TODO: Fetch actual roles from the database instead of hardcoding Admin.
    /// Should query a separate `user_roles` collection or embedded roles array.
    pub fn to_caller(&self) -> Caller {
        Caller::User {
            id: self.id.unwrap().to_string(),
            display_name: self.display_name.clone(),
            email: self.email.clone(),
            roles: vec![Role::Admin], // TODO: get from DB
        }
    }
}

impl From<DbUser> for User {
    fn from(value: DbUser) -> Self {
        Self {
            id: value
                .id
                .map(|oid| oid.to_string())
                .unwrap_or_else(|| "unknown".into()),
            display_name: value.display_name.clone(),
            email: value.email.clone(),
            created_at: value.created_at(),
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbSession {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// The unique session identifier (used in cookies)
    pub session_id: String,

    /// Reference to the authenticated user
    pub user_id: ObjectId,

    /// CSRF token for this session
    pub csrf_token: String,

    /// When the session was created
    #[serde(with = "FromChrono04DateTime")]
    pub created_at: DateTime<Utc>,

    /// When the session expires
    #[serde(with = "FromChrono04DateTime")]
    pub expires_at: DateTime<Utc>,

    /// Last time the session was used (for activity tracking)
    #[serde(with = "FromChrono04DateTime")]
    pub last_used_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbHost {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// The hostname or IP address of the host
    pub hostname: String,

    /// The CPU architecture of the host (e.g., "x86_64", "arm64")
    pub architecture: String,

    /// The operating system information of the host
    pub operating_system: OperatingSystem,

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
            id: value
                .id
                .map(|oid| oid.to_string())
                .unwrap_or_else(|| "unknown".into()),
            hostname: value.hostname.clone(),
            os_id: value.operating_system.id.clone(),
            os_name: value.operating_system.name.clone(),
            os_version: value.operating_system.version.clone(),
            architecture: value.architecture.clone(),
            created_at: value
                .id
                .map(|oid| oid.timestamp().to_system_time().into())
                .unwrap_or_else(|| Utc::now()),
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
