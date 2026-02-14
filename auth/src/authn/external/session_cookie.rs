use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use dropshot::HttpError;
use http::HeaderValue;
use lucid_types::authn::cookies::parse_cookies;
use lucid_uuid_kinds::{ConsoleSessionIdUuid, GenericUuid, OrganisationIdUuid, UserIdUuid};

use crate::authn::{
    self, Actor, Details, Reason,
    external::{HttpAuthnScheme, SchemeResult},
};

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
pub const SESSION_COOKIE_SCHEME_NAME: authn::SchemeName = authn::SchemeName::SessionCookie;

/// Generate session cookie header
pub fn session_cookie_header_value(
    token: &str,
    max_age: Duration,
    secure: bool,
) -> Result<HeaderValue, HttpError> {
    let value = format!(
        "{}={}; Path=/; HttpOnly; SameSite=Lax;{} Max-Age={}",
        SESSION_COOKIE_COOKIE_NAME,
        token,
        if secure { " Secure;" } else { "" },
        max_age.num_seconds(),
    );

    // this will only fail if we mess up the formatting here
    http::HeaderValue::from_str(&value).map_err(|_e| {
        HttpError::for_internal_error(format!("unsupported cookie value: {:#}", value,))
    })
}

/// Generate session cookie with empty token and max-age=0 so browser deletes it
pub fn clear_session_cookie_header_value(secure: bool) -> Result<HeaderValue, HttpError> {
    session_cookie_header_value("", Duration::zero(), secure)
}

#[derive(Debug)]
pub struct HttpAuthnSessionCookie;

#[async_trait]
impl<T> HttpAuthnScheme<T> for HttpAuthnSessionCookie
where
    T: Send + Sync + 'static + SessionStore,
    T::SessionModel: Send + Sync + 'static + Session,
{
    fn name(&self) -> authn::SchemeName {
        SESSION_COOKIE_SCHEME_NAME
    }

    async fn authn(&self, ctx: &T, request: &dropshot::RequestInfo) -> SchemeResult {
        let token = match get_token_from_cookie(request.headers()) {
            Some(token) => token,
            None => return SchemeResult::NotRequested,
        };

        let session = match ctx.session_fetch(token.clone()).await {
            Some(session) => session,
            None => {
                return SchemeResult::Failed(Reason::UnknownActor {
                    actor: token.to_owned(),
                });
            }
        };

        let actor = Actor::OrganisationUser {
            user_id: session.user_id(),
            organisation_id: session.organisation_id(),
        };

        // If the session has gone unused for longer than idle_timeout, it is
        // expired.
        let now = Utc::now();
        if session.last_seen_at() + ctx.session_idle_timeout() < now {
            let expired_session = ctx.session_expire(token.clone()).await;
            if expired_session.is_none() {
                // This can fail if the session was already expired by another request.
                tracing::warn!(
                    "failed to expire session for idle timeout: {}",
                    session.id(),
                );
            }

            return SchemeResult::Failed(Reason::BadCredentials {
                actor,
                source: anyhow!(
                    "session expired due to idle timeout. last used: {}.\
                    time checked: {}. TTL: {}",
                    session.last_seen_at(),
                    now,
                    ctx.session_idle_timeout(),
                ),
            });
        }

        // If the user is still within the idle timeout, but has exceeded the
        // absolute_timeout, the session is also expired.
        if session.created_at() + ctx.session_absolute_timeout() < now {
            let expired_session = ctx.session_expire(token.clone()).await;
            if expired_session.is_none() {
                // This can fail if the session was already expired by another request.
                tracing::warn!(
                    "failed to expire session for absolute timeout: {}",
                    session.id(),
                );
            }

            return SchemeResult::Failed(Reason::BadCredentials {
                actor,
                source: anyhow!(
                    "session expired due to absolute timeout. created at: {}.\
                    time checked: {}. TTL: {}",
                    session.created_at(),
                    now,
                    ctx.session_absolute_timeout(),
                ),
            });
        }

        // we don't want to 500 on error here because the user is legitimately
        // authenticated for this request at this point. The next request might
        // be wrongly considered idle, but that's a problem for the next
        // request.
        let updated_session = ctx.session_update_last_seen(session.id()).await;
        if updated_session.is_none() {
            tracing::debug!("failed to extend session");
        }

        SchemeResult::Authenticated(Details {
            actor,
            credential_id: Some(session.id().into_untyped_uuid()),
        })
    }
}

fn get_token_from_cookie(headers: &http::HeaderMap<http::HeaderValue>) -> Option<String> {
    parse_cookies(headers).ok().and_then(|cs| {
        cs.get(SESSION_COOKIE_COOKIE_NAME)
            .map(|c| c.value().to_string())
    })
}
