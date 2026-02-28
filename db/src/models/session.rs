use bson::serde_helpers::datetime::FromChrono04DateTime;
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

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
