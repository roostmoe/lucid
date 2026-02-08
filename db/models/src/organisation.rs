use diesel::{Selectable, Insertable, Queryable};
use lucid_db_macros::Resource;
use lucid_db_schema::schema::organisations;
use lucid_types::dto::params;
use lucid_uuid_kinds::OrganisationIdUuid;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Queryable, Insertable, Selectable, Resource, Serialize, Deserialize)]
#[diesel(table_name = organisations)]
#[resource(uuid_kind = OrganisationIdKind, deletable = false)]
pub struct Organisation {
    #[diesel(embed)]
    pub identity: OrganisationIdentity,
    pub name: String,
    pub display_name: String,
}

impl Organisation {
    pub fn new(params: params::OrganisationCreate) -> Self {
        Self::new_with_id(OrganisationIdUuid::new_v4(), params)
    }

    pub fn new_with_id(
        id: OrganisationIdUuid,
        params: params::OrganisationCreate
    ) -> Self {
        Self {
            identity: OrganisationIdentity::new(id),
            display_name: params.display_name,
            name: params.name,
        }
    }
}
