use std::collections::HashSet;

use async_trait::async_trait;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lucid_auth::authz::{self, AuthzStorage};
use lucid_common::api::error::Error;
use lucid_db_models::{IdentityPrincipalType, OrganisationUser, RoleBinding, User};
use lucid_db_schema::schema::role_bindings::dsl as role_bindings_dsl;
use lucid_db_schema::schema::organisation_users::dsl as org_users_dsl;
use lucid_db_schema::schema::users::dsl as users_dsl;
use lucid_uuid_kinds::{GenericUuid, OrganisationIdUuid, UserIdUuid};

#[async_trait]
impl AuthzStorage for super::DataStore {
    async fn permissions_for_user_in_org(
        &self,
        user_id: UserIdUuid,
        organisation_id: OrganisationIdUuid,
    ) ->  Result<HashSet<String>, Error> {
        let mut conn = self.pool.get().await
            .map_err(|e| Error::internal_anyhow(
                "failed to get DB connection".into(),
                e.into()
            ))?;

        let role_bindings = role_bindings_dsl::role_bindings
            .filter(role_bindings_dsl::principal_type.eq(IdentityPrincipalType::User))
            .filter(role_bindings_dsl::principal_id.eq(user_id.into_untyped_uuid()))
            .filter(role_bindings_dsl::organisation_id.eq(organisation_id.into_untyped_uuid()))
            .select(RoleBinding::as_select())
            .get_results(&mut conn)
            .await
            .map_err(|e| Error::internal_anyhow(
                "failed to query user role bindings".into(),
                e.into(),
            ))?;

        let mut permissions = HashSet::new();

        for binding in role_bindings {
            if let Some(role) = authz::get_builtin_role(binding.role_name.as_str()) {
                for permission in role.permissions {
                    permissions.insert(permission.to_string());
                }
            }
        }

        Ok(permissions)
    }

    async fn user_is_member_of_org(
        &self,
        user_id: UserIdUuid,
        organisation_id: OrganisationIdUuid,
    ) -> Result<bool, Error> {
        let mut conn = self.pool.get().await
            .map_err(|e| Error::internal_anyhow(
                "failed to get DB connection".into(),
                e.into()
            ))?;

        let user_org = org_users_dsl::organisation_users
            .filter(org_users_dsl::user_id.eq(user_id.into_untyped_uuid()))
            .filter(org_users_dsl::organisation_id.eq(organisation_id.into_untyped_uuid()))
            .select(OrganisationUser::as_select())
            .get_result(&mut conn)
            .await;

        if let Err(error) = user_org {
            return match error {
                diesel::result::Error::NotFound => Ok(false),
                e => Err(Error::internal_anyhow("failed to query user orgs".into(), e.into()))
            }
        }

        Ok(true)
    }

    async fn user_is_system_admin(
        &self,
        user_id: UserIdUuid,
    ) -> Result<bool, Error> {
        let mut conn = self.pool.get().await
            .map_err(|e| Error::internal_anyhow(
                "failed to get DB connection".into(),
                e.into()
            ))?;

        let user = users_dsl::users
            .filter(users_dsl::id.eq(user_id.into_untyped_uuid()))
            .select(User::as_select())
            .get_result(&mut conn)
            .await
            .map_err(|e| Error::internal_anyhow("failed to query user orgs".into(), e.into()))?;

        Ok(user.system_admin)
    }
}
