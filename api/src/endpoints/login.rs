use dropshot::{
    HttpError, HttpResponseHeaders, HttpResponseOk, HttpResponseUpdatedNoContent,
    RequestContext, TypedBody, endpoint,
};
use http::header;
use lucid_auth::authn::external::session_cookie::{
    SESSION_COOKIE_COOKIE_NAME, clear_session_cookie_header_value,
    session_cookie_header_value, SessionStore,
};
use lucid_common::api::error::Error;
use lucid_db_models::User;
use lucid_types::{
    authn::cookies::Cookies,
    dto::{
        params::{LoginParams, LoginSessionParams},
        views::{LoginOrganisation, LoginResponse},
    },
};
use lucid_uuid_kinds::{GenericUuid, OrganisationIdUuid};

use crate::context::Context;

/// Step 1: validate email + password, return the user's organisations.
#[endpoint {
    method = POST,
    path = "/login"
}]
pub async fn login(
    rqctx: RequestContext<Context>,
    body: TypedBody<LoginParams>,
) -> Result<HttpResponseOk<LoginResponse>, HttpError> {
    let ctx = rqctx.context();
    let params = body.into_inner();

    let user = ctx
        .datastore
        .user_get_by_email(&params.email.trim())
        .await
        .map_err(|e| {
            tracing::error!(?e, "failed to look up user");
            Error::internal_anyhow("failed to look up user".into(), e)
        })?
        .ok_or_else(|| Error::Unauthenticated {
            internal_message: format!("no user with email: {}", params.email),
        })?;

    let user_id = lucid_types::identity::Resource::id(&user);

    let valid = ctx
        .datastore
        .user_verify_password(user_id, &params.password)
        .await
        .map_err(|e| Error::internal_anyhow("password verification failed".into(), e))?;

    if !valid {
        return Err(Error::Unauthenticated {
            internal_message: "invalid password".into(),
        }
        .into());
    }

    let orgs = ctx
        .datastore
        .organisations_for_user(user_id)
        .await
        .map_err(|e| {
            Error::internal_anyhow("failed to list organisations".into(), e)
        })?;

    let organisations = orgs
        .into_iter()
        .map(|org| {
            let org_id = lucid_types::identity::Resource::id(&org);
            LoginOrganisation {
                id: org_id.into_untyped_uuid(),
                name: org.name,
                display_name: org.display_name,
            }
        })
        .collect();

    Ok(HttpResponseOk(LoginResponse { organisations }))
}

/// Step 2: validate credentials again, create a session for the chosen org,
/// and set the session cookie.
#[endpoint {
    method = POST,
    path = "/login/session"
}]
pub async fn login_session(
    rqctx: RequestContext<Context>,
    body: TypedBody<LoginSessionParams>,
) -> Result<HttpResponseHeaders<HttpResponseOk<()>>, HttpError> {
    let ctx = rqctx.context();
    let params = body.into_inner();

    // --- re-validate credentials ----------------------------------------

    let user = ctx
        .datastore
        .user_get_by_email(&params.email)
        .await
        .map_err(|e| Error::internal_anyhow("failed to look up user".into(), e))?
        .ok_or_else(|| Error::Unauthenticated {
            internal_message: format!("no user with email: {}", params.email),
        })?;

    let user_id = lucid_types::identity::Resource::id(&user);

    let valid = ctx
        .datastore
        .user_verify_password(user_id, &params.password)
        .await
        .map_err(|e| Error::internal_anyhow("password verification failed".into(), e))?;

    if !valid {
        return Err(Error::Unauthenticated {
            internal_message: "invalid password".into(),
        }
        .into());
    }

    // --- verify org membership ------------------------------------------

    let organisation_id =
        OrganisationIdUuid::from_untyped_uuid(params.organisation_id);

    let orgs = ctx
        .datastore
        .organisations_for_user(user_id)
        .await
        .map_err(|e| {
            Error::internal_anyhow("failed to list organisations".into(), e)
        })?;

    let is_member = orgs.iter().any(|org| {
        let org_id: OrganisationIdUuid = lucid_types::identity::Resource::id(org);
        org_id == organisation_id
    });

    if !is_member {
        return Err(Error::Forbidden {
            internal_message: format!(
                "user {} is not a member of organisation {}",
                user_id, organisation_id,
            ),
            required_permission: None,
        }
        .into());
    }

    // --- create session -------------------------------------------------

    let token = uuid::Uuid::new_v4().to_string();

    ctx.datastore
        .session_create(user_id, organisation_id, &token)
        .await
        .map_err(|e| {
            Error::internal_anyhow("failed to create session".into(), e)
        })?;

    // --- set cookie -----------------------------------------------------

    // Use the absolute timeout as the cookie max-age so the browser drops it
    // at the same time the server would expire it.
    let max_age = chrono::Duration::hours(24);

    // TODO: make `secure` configurable (should be true in production)
    let cookie_value = session_cookie_header_value(&token, max_age, false)?;

    let mut response = HttpResponseHeaders::new_unnamed(HttpResponseOk(()));
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie_value);

    Ok(response)
}

/// Expire the current session and clear the session cookie.
#[endpoint {
    method = POST,
    path = "/logout"
}]
pub async fn logout(
    rqctx: RequestContext<Context>,
    cookies: Cookies,
) -> Result<HttpResponseHeaders<HttpResponseUpdatedNoContent>, HttpError> {
    let ctx = rqctx.context();

    // If the caller sent a session cookie, expire the session server-side.
    // We intentionally don't fail if the session is already gone — the user's
    // intent is to be logged out, and they are.
    if let Some(cookie) = cookies.get(SESSION_COOKIE_COOKIE_NAME) {
        let token = cookie.value().to_string();
        let _ = ctx.datastore.session_expire(token).await;
    }

    // Always clear the cookie in the browser regardless of whether a
    // server-side session existed.
    // TODO: make `secure` configurable (should be true in production)
    let cookie_value = clear_session_cookie_header_value(false)?;

    let mut response =
        HttpResponseHeaders::new_unnamed(HttpResponseUpdatedNoContent());
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie_value);

    Ok(response)
}

#[endpoint {
    method = GET,
    path = "/whoami"
}]
pub async fn whoami(
    rqctx: RequestContext<Context>,
) -> Result<HttpResponseOk<User>, HttpError> {
    let ctx = rqctx.context();
    let user_id = ctx
        .authenticator
        .authn_request(&rqctx)
        .await?
        .actor_required()?
        .user_id()
        .unwrap();
    let user = ctx
        .datastore
        .user_get(user_id)
        .await
        .map_err(|e|
            Error::internal_anyhow("failed to lookup user".into(), e)
        )?;

    Ok(HttpResponseOk(user.unwrap()))
}
