use chrono::{DateTime, Utc};
use diesel::{
    prelude::{Insertable, Queryable},
    Selectable,
};
use lucid_db_macros::Resource;
use lucid_db_schema::schema::console_sessions;
use lucid_uuid_kinds::{OrganisationIdKind, UserIdKind};
use serde::{Deserialize, Serialize};

use crate::DbTypedUuid;

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Resource, Serialize, Deserialize)]
#[diesel(table_name = console_sessions)]
#[resource(uuid_kind = ConsoleSessionIdKind, deletable = false)]
pub struct ConsoleSession {
    #[diesel(embed)]
    pub identity: ConsoleSessionIdentity,

    pub user_id: DbTypedUuid<UserIdKind>,
    pub token: String,
    pub last_seen_at: DateTime<Utc>,
    pub organisation_id: DbTypedUuid<OrganisationIdKind>,
}
