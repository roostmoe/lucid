use async_trait::async_trait;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lucid_auth::context::OpContext;
use lucid_auth::storage::Storage;
use lucid_common::api::ResourceType;
use lucid_common::api::error::Error;
use lucid_db_models::{IdentityPrincipalType, RoleBinding};
use uuid::Uuid;

use crate::errors::{ErrorHandler, public_error_from_diesel};

#[async_trait]
impl Storage for super::DataStore {
    async fn role_bind_list_for(
        &self,
        opctx: &OpContext,
        identity_type: IdentityPrincipalType,
        identity_id: Uuid,
        resource_type: ResourceType,
        resource_id: Uuid,
    ) -> Result<Vec<RoleBinding>, Error> {
        use lucid_db_schema::schema::role_bindings::dsl as role_dsl;

        let direct_roles_query = role_dsl::role_bindings
            .filter(role_dsl::principal_type.eq(identity_type.clone()))
            .filter(role_dsl::principal_id.eq(identity_id))
            .filter(role_dsl::resource_type.eq(resource_type.to_string()))
            .filter(role_dsl::resource_id.eq(resource_id))
            .select(RoleBinding::as_select());

        let mut conn = self.pool_conn_authorized(opctx).await?;

        direct_roles_query
            .get_results(&mut conn)
            .await
            .map_err(|e| public_error_from_diesel(e, ErrorHandler::Server))
    }
}
