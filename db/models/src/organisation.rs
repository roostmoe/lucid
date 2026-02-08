use diesel::{Selectable, Insertable, Queryable};
use lucid_db_macros::Resource;
use lucid_db_schema::schema::organisations;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Queryable, Insertable, Selectable, Resource, Serialize, Deserialize)]
#[diesel(table_name = organisations)]
#[resource(uuid_kind = OrganisationIdKind, deletable = false)]
pub struct Organisation {
    #[diesel(embed)]
    pub identity: OrganisationIdentity,
    pub name: String,
}
