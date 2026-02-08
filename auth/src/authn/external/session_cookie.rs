use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use lucid_uuid_kinds::{ConsoleSessionIdUuid, OrganisationIdUuid, UserIdUuid};

use crate::authn;

pub trait Session {
    fn id(&self) -> ConsoleSessionIdUuid;
    fn user_id(&self) -> UserIdUuid;
    fn organisation_id(&self) -> OrganisationIdUuid;
    fn last_seen_at(&self) -> DateTime<Utc>;
    fn created_at(&self) -> DateTime<Utc>;
}

#[async_trait]
pub trait SessionStore {
    type SessionModel;

    async fn session_fetch(&self, token: String) -> Option<Self::SessionModel>;

    async fn session_update_last_seen(
        &self,
        id: ConsoleSessionIdUuid,
    ) -> Option<Self::SessionModel>;

    async fn session_expire(&self, token: String) -> Option<()>;

    fn session_idle_timeout(&self) -> Duration;

    fn session_absolute_timeout(&self) -> Duration;
}

pub const SESSION_COOKIE_COOKIE_NAME: &str = "session";
pub const SESSION_COOKIE_SCHEME_NAME: authn::SchemeName =
    authn::SchemeName::SessionCookie;
