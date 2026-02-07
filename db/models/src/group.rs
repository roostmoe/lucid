use chrono::{DateTime, Utc};
use diesel::{Queryable, Selectable, Insertable};
use lucid_db_schema::schema::{group_memberships, groups};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = groups)]
pub struct Group {
    pub id: Uuid,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = group_memberships)]
pub struct GroupMembership {
    pub group_id: Uuid,
    pub user_id: Uuid,
}
