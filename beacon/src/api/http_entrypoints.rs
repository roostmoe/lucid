use crate::context::Context as Ctx;
use dropshot::{
    ApiDescription, ClientErrorStatusCode, HttpError, HttpResponseOk, HttpResponseSeeOther, Query,
    RequestContext,
};
use lucid_beacon_api::{AuthCallbackQuery, BeaconApi, beacon_api_mod};
use lucid_types::dto::views;

type BeaconApiDescription = ApiDescription<Ctx>;

pub(crate) fn api() -> BeaconApiDescription {
    beacon_api_mod::api_description::<BeaconApiImpl>().expect("registering entrypoints")
}

enum BeaconApiImpl {}

impl BeaconApi for BeaconApiImpl {
    type Context = Ctx;

    async fn list_hosts(
        rqctx: RequestContext<Self::Context>,
    ) -> Result<HttpResponseOk<views::HostListResponse>, HttpError> {
        let apictx = rqctx.context();
        let opctx = crate::context::op_context_for_external_api(&rqctx).await?;

        // Require authentication
        opctx.authn.actor_required().map_err(HttpError::from)?;

        let hosts = apictx
            .beacon
            .datastore
            .host_list(false)
            .await
            .map_err(|e| HttpError::for_internal_error(format!("db error: {e}")))?;

        Ok(HttpResponseOk(views::HostListResponse {
            hosts: hosts
                .iter()
                .map(|h| views::HostView {
                    id: h.identity.id.into(),
                    hostname: "".into(),
                    created_at: h.identity.created_at,
                    updated_at: h.identity.updated_at,
                    deleted_at: None,
                })
                .collect(),
            pagination: views::PaginationMeta {
                total_items: 0,
                total_pages: 0,
                current_page: 0,
                page_size: 0,
            },
        }))
    }

    async fn auth_login(
        rqctx: RequestContext<Self::Context>,
    ) -> Result<HttpResponseSeeOther, HttpError> {
        let apictx = rqctx.context();

        let (auth_url, csrf_token, nonce) = apictx.beacon.oidc.authorization_url();

        // Store state for callback validation
        apictx.beacon.oidc_state.store(csrf_token, nonce).await;

        dropshot::http_response_see_other(auth_url)
    }

    async fn auth_callback(
        rqctx: RequestContext<Self::Context>,
        query: Query<AuthCallbackQuery>,
    ) -> Result<HttpResponseOk<views::TokenResponse>, HttpError> {
        let apictx = rqctx.context();
        let query = query.into_inner();

        // Validate CSRF token and get nonce
        let nonce = apictx
            .beacon
            .oidc_state
            .get_and_remove(&query.state)
            .await
            .ok_or_else(|| {
                HttpError::for_client_error(
                    None,
                    ClientErrorStatusCode::BAD_REQUEST,
                    "invalid or expired state".to_string(),
                )
            })?;

        // Exchange code for user info
        let user_info = apictx
            .beacon
            .oidc
            .exchange_code(&query.code, &nonce)
            .await
            .map_err(|e| HttpError::for_internal_error(format!("oidc error: {e}")))?;

        // Validate email is allowed
        if !apictx.beacon.oidc.is_email_allowed(&user_info.email) {
            return Err(HttpError::for_client_error(
                None,
                ClientErrorStatusCode::FORBIDDEN,
                "email not allowed".to_string(),
            ));
        }

        // Upsert user in database
        let user = apictx
            .beacon
            .datastore
            .user_upsert_from_oidc(
                &user_info.external_id,
                &user_info.email,
                user_info.display_name.as_deref(),
            )
            .await
            .map_err(|e| HttpError::for_internal_error(format!("db error: {e}")))?;

        // Create JWT
        let token = apictx
            .beacon
            .jwt
            .create_token(user.identity.id.into(), &user.email)
            .map_err(|e| HttpError::for_internal_error(format!("jwt error: {e}")))?;

        Ok(HttpResponseOk(views::TokenResponse {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in: 86400, // 24 hours
        }))
    }
}
