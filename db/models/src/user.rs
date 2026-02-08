use diesel::{Selectable, Insertable, Queryable};
use lucid_db_macros::Resource;
use lucid_db_schema::schema::{organisation_users, users};
use lucid_types::dto::params::{self, UserCreateAuthMode};
use lucid_uuid_kinds::{OrganisationIdKind, UserIdKind, UserIdUuid};
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

impl User {
    pub fn new(params: params::UserCreate) -> Self {
        Self::new_with_id(UserIdUuid::new_v4(), params)
    }

    pub fn new_with_id(id: UserIdUuid, params: params::UserCreate) -> Self {
        Self {
            identity: UserIdentity::new(id),
            email: params.email,
            external_id: if let UserCreateAuthMode::External { external_id } =  params.auth_mode { Some(external_id) } else { None },
            password_hash: None,
        }
    }
}
