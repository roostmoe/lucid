use diesel::{Selectable, Insertable, Queryable};
use lucid_db_macros::Resource;
use lucid_db_schema::schema::organisation_roles;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Queryable, Insertable, Selectable, Resource, Serialize, Deserialize)]
#[diesel(table_name = organisation_roles)]
#[resource(uuid_kind = RoleIdKind, deletable = false)]
pub struct Role {
    #[diesel(embed)]
    pub identity: RoleIdentity,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub permissions: Vec<String>,
}
