use chrono::{DateTime, Utc};
use diesel::{Insertable, Selectable, Queryable};
use lucid_db_schema::schema::{user_password_hashes, users};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Selectable, Queryable, Insertable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,

    pub email: String,
    pub display_name: String,
}

#[derive(Selectable, Insertable, PartialEq, Debug)]
#[diesel(table_name = user_password_hashes)]
pub struct UserPasswordHash {
    pub user_id: Uuid,
    pub hash: String,
}
