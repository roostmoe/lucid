use chrono::{DateTime, Utc};
use diesel::{Queryable, Selectable, Insertable};
use lucid_db_schema::{schema::sessions, schema_ext::IdentityType};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = sessions)]
pub struct Session {
    pub id: Uuid,
    pub identity_type: IdentityType,
    pub identity_id: Uuid,
    pub refresh_hash: String,
    pub family_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
