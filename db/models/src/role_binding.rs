use diesel::{Selectable, Insertable, Queryable};
use lucid_db_macros::Resource;
use lucid_db_schema::schema::role_bindings;
use lucid_uuid_kinds::OrganisationIdKind;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{DbTypedUuid, IdentityPrincipalType};

#[derive(Clone, Debug, Queryable, Insertable, Selectable, Resource, Serialize, Deserialize)]
#[diesel(table_name = role_bindings)]
#[resource(uuid_kind = RoleBindingIdKind, deletable = false)]
pub struct RoleBinding {
    #[diesel(embed)]
    pub identity: RoleBindingIdentity,
    pub role_name: String,
    pub organisation_id: DbTypedUuid<OrganisationIdKind>,
    pub principal_id: Uuid,
    pub principal_type: IdentityPrincipalType,
    pub resource_id: Uuid,
    pub resource_type: String,
}
