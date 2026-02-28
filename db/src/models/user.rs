use std::fmt::Display;

use bson::serde_helpers::datetime::FromChrono04DateTime;
use chrono::{DateTime, Utc};
use lucid_common::{caller::{Caller, Role}, views::User};
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


