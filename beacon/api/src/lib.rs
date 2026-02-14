use dropshot::{
    EndpointTagPolicy, HttpError, HttpResponseOk, HttpResponseSeeOther, Query, RequestContext,
};
use dropshot_api_manager_types::api_versions;
use lucid_types::dto::views;
use schemars::JsonSchema;
use serde::Deserialize;

api_versions!([(1, INITIAL_API),]);

#[derive(Deserialize, JsonSchema)]
pub struct AuthCallbackQuery {
    pub code: String,
    pub state: String, // CSRF token
}

#[dropshot::api_description {
    tag_config = {
        allow_other_tags = false,
        policy = EndpointTagPolicy::ExactlyOne,
        tags = {
            "auth" = {
                description = "Authentication endpoints (OIDC)",
            }
        }
    }
}]
pub trait BeaconApi {
    type Context;

    /// Initiate OIDC login flow
    #[endpoint {
        method = GET,
        path = "/auth/login",
        tags = ["auth"],
    }]
    async fn auth_login(
        rqctx: RequestContext<Self::Context>,
    ) -> Result<HttpResponseSeeOther, HttpError>;

    /// Handle OIDC callback
    #[endpoint {
        method = GET,
        path = "/auth/callback",
        tags = ["auth"],
    }]
    async fn auth_callback(
        rqctx: RequestContext<Self::Context>,
        query: Query<AuthCallbackQuery>,
    ) -> Result<HttpResponseOk<views::TokenResponse>, HttpError>;
}
