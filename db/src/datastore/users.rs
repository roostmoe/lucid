use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use lucid_db_models::{User, UserIdentity};
use lucid_db_schema::schema::users;
use lucid_types::dto::params::{UserCreate, UserCreateAuthMode};
use lucid_uuid_kinds::{GenericUuid, UserIdUuid};

use crate::datastore::DataStore;

impl DataStore {
    /// Get a user by ID
    pub async fn user_get(&self, user_id: UserIdUuid) -> anyhow::Result<Option<User>> {
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

    /// List all users
    pub async fn user_list(&self) -> anyhow::Result<Vec<User>> {
        let mut conn = self.pool.get().await?;

        let users_list = users::table
            .select(User::as_select())
            .load(&mut conn)
            .await?;

        Ok(users_list)
    }

    /// List users with pagination
    pub async fn user_list_paginated(
        &self,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<User>> {
        let mut conn = self.pool.get().await?;

        let users_list = users::table
            .select(User::as_select())
            .limit(limit)
            .offset(offset)
            .load(&mut conn)
            .await?;

        Ok(users_list)
    }

    /// Create a new user with password hashing
    pub async fn user_create(&self, params: UserCreate) -> anyhow::Result<User> {
        let mut conn = self.pool.get().await?;

        // hash password if local auth mode
        let password_hash = match &params.auth_mode {
            UserCreateAuthMode::Local { password_hash } => {
                Some(hash_password(password_hash)?)
            }
            UserCreateAuthMode::External { .. } => None,
        };

        let new_user = User {
            identity: UserIdentity::new(UserIdUuid::new_v4()),
            email: params.email,
            external_id: if let UserCreateAuthMode::External { external_id } = params.auth_mode {
                Some(external_id)
            } else {
                None
            },
            password_hash,
            system_admin: false,
        };

        let user = diesel::insert_into(users::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(&mut conn)
            .await?;

        Ok(user)
    }

    /// Verify a user's password
    pub async fn user_verify_password(
        &self,
        user_id: UserIdUuid,
        password: &str,
    ) -> anyhow::Result<bool> {
        let user = self
            .user_get(user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("user not found"))?;

        let password_hash = user
            .password_hash
            .ok_or_else(|| anyhow::anyhow!("user has no password hash"))?;

        Ok(verify_password(password, &password_hash)?)
    }
}

/// Hash a password using bcrypt
fn hash_password(password: &str) -> anyhow::Result<String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| anyhow::anyhow!("failed to hash password: {}", e))
}

/// Verify a password against a bcrypt hash
fn verify_password(password: &str, hash: &str) -> anyhow::Result<bool> {
    bcrypt::verify(password, hash)
        .map_err(|e| anyhow::anyhow!("failed to verify password: {}", e))
}
