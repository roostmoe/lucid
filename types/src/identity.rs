use chrono::{DateTime, Utc};
use lucid_uuid_kinds::{GenericUuid};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, JsonSchema)]
pub struct ResourceIdentityMetadata {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Resource is an interface providing a default 'standard' set of fields that
/// all API resources will keep in the database.
pub trait Resource {
    type IdType: GenericUuid;

    fn id(&self) -> Self::IdType;
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;
    fn deleted_at(&self) -> Option<DateTime<Utc>>;

    fn identity(&self) -> ResourceIdentityMetadata {
        ResourceIdentityMetadata {
            id: self.id().into_untyped_uuid(),
            created_at: self.created_at(),
            updated_at: self.updated_at(),
            deleted_at: self.deleted_at()
        }
    }
}
