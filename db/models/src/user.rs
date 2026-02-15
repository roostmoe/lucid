use diesel::{Insertable, Queryable, Selectable};
use lucid_db_macros::Resource;
use lucid_db_schema::schema::users;
use lucid_uuid_kinds::UserUuid;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, Queryable, Insertable, Selectable, Resource, Serialize, Deserialize, JsonSchema,
)]
#[diesel(table_name = users)]
#[resource(uuid_kind = UserKind, deletable = false)]
pub struct User {
    #[diesel(embed)]
    pub identity: UserIdentity,
    pub email: String,
    pub external_id: String,
    pub display_name: Option<String>,
    pub is_owner: bool,
}

impl User {
    /// Create a new user from OIDC claims
    pub fn new(
        id: UserUuid,
        external_id: String,
        email: String,
        display_name: Option<String>,
        is_owner: bool,
    ) -> Self {
        Self {
            identity: UserIdentity::new(id),
            email,
            external_id,
            display_name,
            is_owner,
        }
    }
}
