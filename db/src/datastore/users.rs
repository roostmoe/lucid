use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use lucid_auth::{authz::{self, Action}, context::OpContext};
use lucid_common::api::{ResourceType, error::{Error, LookupType}};
use lucid_db_models::{User, UserIdentity};
use lucid_db_schema::schema::users;
use lucid_types::dto::params::{UserCreate, UserCreateAuthMode};
use lucid_uuid_kinds::{GenericUuid, UserIdUuid};

use crate::{datastore::DataStore, errors::public_error_from_diesel_lookup};

impl DataStore {
    /// Get a user by ID
    pub async fn user_get(
        &self,
        opctx: &OpContext,
        user_id: UserIdUuid,
    ) -> Result<Option<User>, Error> {
        opctx.authorize(Action::Get, &authz::User::new(user_id)).await?;

        let user_query = users::table
            .filter(users::id.eq(user_id.into_untyped_uuid()))
            .select(User::as_select());

        let mut conn = self.pool_conn_authorized(opctx).await?;

        let user = user_query
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e|
                public_error_from_diesel_lookup(
                    e,
                    ResourceType::User,
                    &LookupType::ById(user_id.into_untyped_uuid()),
                )
            )?;

        Ok(user)
    }

    /// Get a user by email
    pub async fn user_get_by_email(
        &self,
        opctx: &OpContext,
        email: &str
    ) -> Result<Option<User>, Error> {
        let user_query = users::table
            .filter(users::email.eq(email))
            .select(User::as_select())
            .limit(1);

        let mut conn = self.pool_conn_authorized(opctx).await?;
        let user = user_query
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| Error::internal_anyhow("failed to query user by email".into(), e.into()))?;

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
        opctx: &OpContext,
        user_id: UserIdUuid,
        password: &str,
    ) -> Result<bool, Error> {
        let user = self
            .user_get(&opctx, user_id)
            .await?
            .ok_or_else(|| Error::Unauthenticated {
                internal_message: format!("no user with id: {}", user_id),
            })?;

        let password_hash = user
            .password_hash
            .ok_or_else(|| Error::Unauthenticated {
                internal_message: format!("no user with id: {}", user_id),
            })?;

        Ok(verify_password(password, &password_hash)?)
    }
}

/// Hash a password using bcrypt
fn hash_password(password: &str) -> Result<String, Error> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| Error::internal_error(&format!("failed to hash password: {}", e)))
}

/// Verify a password against a bcrypt hash
fn verify_password(password: &str, hash: &str) -> Result<bool, Error> {
    bcrypt::verify(password, hash)
        .map_err(|e| Error::Unauthenticated {
            internal_message: format!("failed to verify password: {}", e),
        })
}
