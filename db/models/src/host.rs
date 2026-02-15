use diesel::{Insertable, Queryable, Selectable};
use lucid_db_macros::Resource;
use lucid_db_schema::schema::inventory_hosts;
use lucid_uuid_kinds::HostUuid;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, Queryable, Insertable, Selectable, Resource, Serialize, Deserialize, JsonSchema,
)]
#[diesel(table_name = inventory_hosts)]
#[resource(uuid_kind = HostKind)]
pub struct Host {
    #[diesel(embed)]
    pub identity: HostIdentity,
}

impl Host {
    /// Create a new user from OIDC claims
    pub fn new(id: HostUuid) -> Self {
        Self {
            identity: HostIdentity::new(id),
        }
    }
}
