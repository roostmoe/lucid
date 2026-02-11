use dropshot::{
    HttpError, HttpResponseHeaders, HttpResponseOk, HttpResponseUpdatedNoContent,
    RequestContext, TypedBody, endpoint,
};
use http::header;
use lucid_auth::{authn::external::session_cookie::{
    SESSION_COOKIE_COOKIE_NAME, SessionStore, clear_session_cookie_header_value, session_cookie_header_value
}};
use lucid_common::api::error::Error;
use lucid_db_models::User;
use lucid_types::{
    authn::cookies::Cookies,
    dto::{
        params::{LoginParams, LoginSessionParams},
        views::{LoginOrganisation, LoginResponse},
    }, identity::Resource,
};
use lucid_uuid_kinds::{GenericUuid, OrganisationIdUuid};

use crate::{config::ServerMode, context::{Context, op_context_for_external_api}, session::gen_session_id};

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
        .user_get_by_email(
            &ctx.opctx_external_authn,
            &params.email.trim()
        )
        .await?
        .ok_or_else(|| Error::Unauthenticated {
            internal_message: format!("no user with email: {}", params.email),
        })?;

    let user_id = Resource::id(&user);

    let valid = ctx
        .datastore
        .user_verify_password(
            &ctx.opctx_external_authn,
            user_id,
            &params.password
        )
        .await?;

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
        .user_get_by_email(&ctx.opctx_external_authn, &params.email)
        .await?
        .ok_or_else(|| Error::Unauthenticated {
            internal_message: format!("no user with email: {}", params.email),
        })?;

    let user_id = lucid_types::identity::Resource::id(&user);

    let valid = ctx
        .datastore
        .user_verify_password(
            &ctx.opctx_external_authn,
            user_id,
            &params.password
        )
        .await?;

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
        return Err(Error::Forbidden.into());
    }

    // --- create session -------------------------------------------------

    let token = gen_session_id();

    ctx.datastore
        .session_create(user_id, organisation_id, &token)
        .await
        .map_err(|e| {
            Error::internal_anyhow("failed to create session".into(), e)
        })?;

    // --- set cookie -----------------------------------------------------

    let cookie_value = session_cookie_header_value(
        &token,
        ctx.session_config.idle_timeout,
        ctx.mode == ServerMode::Production,
    )?;

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
    let cookie_value = clear_session_cookie_header_value(
        ctx.mode == ServerMode::Production,
    )?;

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
    let opctx = op_context_for_external_api(&rqctx).await?;
    let actor = opctx.authn.actor_required()?;

    if actor.user_id() == None {
        return Err(Error::Unauthenticated {
            internal_message: "actor had no user id".into()
        }.into())
    }

    let user = ctx
        .datastore
        .user_get(&opctx, actor.user_id().unwrap())
        .await?;

    Ok(HttpResponseOk(user.unwrap()))
}
