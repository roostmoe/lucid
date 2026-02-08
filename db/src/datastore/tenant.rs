use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use lucid_db_models::Organisation;
use lucid_db_schema::schema::{organisation_users, organisations};
use lucid_uuid_kinds::{GenericUuid, UserIdUuid};

use crate::datastore::DataStore;

impl DataStore {
    /// List all organisations a user belongs to.
    pub async fn organisations_for_user(
        &self,
        user_id: UserIdUuid,
    ) -> anyhow::Result<Vec<Organisation>> {
        let mut conn = self.pool.get().await?;

        let orgs = organisation_users::table
            .inner_join(organisations::table)
            .filter(organisation_users::user_id.eq(user_id.into_untyped_uuid()))
            .select(Organisation::as_select())
            .load(&mut conn)
            .await?;

        Ok(orgs)
    }
}
