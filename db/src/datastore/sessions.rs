use async_trait::async_trait;
use chrono::Duration;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use lucid_auth::authn;
use lucid_auth::authn::external::session_cookie::SessionStore;
use lucid_db_models::ConsoleSession;
use lucid_db_schema::schema::console_sessions;
use lucid_uuid_kinds::{ConsoleSessionIdUuid, GenericUuid, OrganisationIdUuid};

use crate::datastore::DataStore;

/// Session idle timeout: 1 hour of inactivity expires the session.
const SESSION_IDLE_TIMEOUT_HOURS: i64 = 1;

/// Session absolute timeout: 24 hours from creation, regardless of activity.
const SESSION_ABSOLUTE_TIMEOUT_HOURS: i64 = 24;

#[async_trait]
impl SessionStore for DataStore {
    type SessionModel = authn::ConsoleSession;

    async fn session_fetch(&self, token: String) -> Option<Self::SessionModel> {
        let mut conn = self.pool.get().await.ok()?;

        let session: ConsoleSession = console_sessions::table
            .filter(console_sessions::token.eq(&token))
            .select(ConsoleSession::as_select())
            .first(&mut conn)
            .await
            .ok()?;

        let organisation_id: OrganisationIdUuid =
            OrganisationIdUuid::from_untyped_uuid(
                session.organisation_id.into_untyped_uuid(),
            );

        Some(authn::ConsoleSession {
            console_session: session,
            organisation_id,
        })
    }

    async fn session_update_last_seen(
        &self,
        id: ConsoleSessionIdUuid,
    ) -> Option<Self::SessionModel> {
        let mut conn = self.pool.get().await.ok()?;

        let now = chrono::Utc::now();

        let session: ConsoleSession = diesel::update(
            console_sessions::table
                .filter(console_sessions::id.eq(id.into_untyped_uuid())),
        )
        .set(console_sessions::last_seen_at.eq(now))
        .returning(ConsoleSession::as_returning())
        .get_result(&mut conn)
        .await
        .ok()?;

        let organisation_id: OrganisationIdUuid =
            OrganisationIdUuid::from_untyped_uuid(
                session.organisation_id.into_untyped_uuid(),
            );

        Some(authn::ConsoleSession {
            console_session: session,
            organisation_id,
        })
    }

    async fn session_expire(&self, token: String) -> Option<()> {
        let mut conn = self.pool.get().await.ok()?;

        let rows_deleted = diesel::delete(
            console_sessions::table
                .filter(console_sessions::token.eq(&token)),
        )
        .execute(&mut conn)
        .await
        .ok()?;

        if rows_deleted > 0 {
            Some(())
        } else {
            None
        }
    }

    fn session_idle_timeout(&self) -> Duration {
        Duration::hours(SESSION_IDLE_TIMEOUT_HOURS)
    }

    fn session_absolute_timeout(&self) -> Duration {
        Duration::hours(SESSION_ABSOLUTE_TIMEOUT_HOURS)
    }
}

impl DataStore {
    /// Create a new console session for a user scoped to an organisation.
    pub async fn session_create(
        &self,
        user_id: lucid_uuid_kinds::UserIdUuid,
        organisation_id: OrganisationIdUuid,
        token: &str,
    ) -> anyhow::Result<ConsoleSession> {
        let mut conn = self.pool.get().await?;

        let new_session = ConsoleSession {
            identity: lucid_db_models::ConsoleSessionIdentity::new(
                ConsoleSessionIdUuid::new_v4(),
            ),
            user_id: lucid_db_models::to_db_typed_uuid(user_id),
            token: token.to_string(),
            last_seen_at: chrono::Utc::now(),
            organisation_id: lucid_db_models::to_db_typed_uuid(organisation_id),
        };

        let session = diesel::insert_into(console_sessions::table)
            .values(&new_session)
            .returning(ConsoleSession::as_returning())
            .get_result(&mut conn)
            .await?;

        Ok(session)
    }

    /// Delete all sessions for a user (e.g. on password change).
    pub async fn sessions_delete_for_user(
        &self,
        user_id: lucid_uuid_kinds::UserIdUuid,
    ) -> anyhow::Result<usize> {
        let mut conn = self.pool.get().await?;

        let rows_deleted = diesel::delete(
            console_sessions::table
                .filter(console_sessions::user_id.eq(user_id.into_untyped_uuid())),
        )
        .execute(&mut conn)
        .await?;

        Ok(rows_deleted)
    }
}
