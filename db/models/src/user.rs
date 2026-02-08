use diesel::{Selectable, Insertable, Queryable};
use lucid_db_macros::Resource;
use lucid_db_schema::schema::{organisation_users, users};
use lucid_uuid_kinds::{OrganisationIdKind, UserIdKind};
use serde::{Deserialize, Serialize};

use crate::DbTypedUuid;

#[derive(Clone, Debug, Queryable, Insertable, Selectable, Resource, Serialize, Deserialize)]
#[diesel(table_name = users)]
#[resource(uuid_kind = UserIdKind, deletable = false)]
pub struct User {
    #[diesel(embed)]
    pub identity: UserIdentity,
    pub email: String,
    pub external_id: Option<String>,
    pub password_hash: Option<String>,
}

#[derive(Clone, Debug, Queryable, Insertable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = organisation_users)]
pub struct OrganisationUser {
    pub user_id: DbTypedUuid<UserIdKind>,
    pub organisation_id: DbTypedUuid<OrganisationIdKind>,
}
