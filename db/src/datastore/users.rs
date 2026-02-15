use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use lucid_db_models::User;
use lucid_db_schema::schema::users;
use lucid_uuid_kinds::{GenericUuid, UserUuid};

use crate::datastore::DataStore;

impl DataStore {
    /// Get a user by ID
    pub async fn user_get(&self, user_id: UserUuid) -> anyhow::Result<Option<User>> {
        let mut conn = self.pool.get().await?;

        let user = users::table
            .filter(users::id.eq(user_id.into_untyped_uuid()))
            .select(User::as_select())
            .first(&mut conn)
            .await
            .optional()?;

        Ok(user)
    }

    /// Get a user by email
    pub async fn user_get_by_email(&self, email: &str) -> anyhow::Result<Option<User>> {
        let mut conn = self.pool.get().await?;

        let user = users::table
            .filter(users::email.eq(email))
            .select(User::as_select())
            .first(&mut conn)
            .await
            .optional()?;

        Ok(user)
    }

    /// Get a user by external ID (OIDC sub claim)
    pub async fn user_get_by_external_id(&self, external_id: &str) -> anyhow::Result<Option<User>> {
        let mut conn = self.pool.get().await?;

        let user = users::table
            .filter(users::external_id.eq(external_id))
            .select(User::as_select())
            .first(&mut conn)
            .await
            .optional()?;

        Ok(user)
    }

    /// List all users (simple, no pagination for now)
    pub async fn user_list(&self) -> anyhow::Result<Vec<User>> {
        let mut conn = self.pool.get().await?;

        let users_list = users::table
            .select(User::as_select())
            .load(&mut conn)
            .await?;

        Ok(users_list)
    }

    /// Upsert a user from OIDC login.
    /// First user to log in becomes owner.
    pub async fn user_upsert_from_oidc(
        &self,
        external_id: &str,
        email: &str,
        display_name: Option<&str>,
    ) -> anyhow::Result<User> {
        let mut conn = self.pool.get().await?;

        // Check if any users exist
        let user_count: i64 = users::table.count().get_result(&mut conn).await?;
        let is_first_user = user_count == 0;

        // Upsert user
        let user = diesel::insert_into(users::table)
            .values((
                users::external_id.eq(external_id),
                users::email.eq(email),
                users::display_name.eq(display_name),
                users::is_owner.eq(is_first_user),
            ))
            .on_conflict(users::external_id)
            .do_update()
            .set((
                users::email.eq(email),
                users::display_name.eq(display_name),
                users::updated_at.eq(diesel::dsl::now),
            ))
            .returning(User::as_returning())
            .get_result(&mut conn)
            .await?;

        Ok(user)
    }
}
