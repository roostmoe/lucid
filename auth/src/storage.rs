use lucid_common::api::error::Error;
use lucid_db_models::{IdentityPrincipalType, RoleBinding};
use uuid::Uuid;

use crate::context::OpContext;

/// Storage operations that require an [`OpContext`] (and therefore authz
/// checks).
pub trait Storage: Send + Sync {
    async fn role_bind_list_for(
        &self,
        opctx: &OpContext,
        identity_type: IdentityPrincipalType,
        identity_id: Uuid,
    ) -> Result<Vec<RoleBinding>, Error>;
}
