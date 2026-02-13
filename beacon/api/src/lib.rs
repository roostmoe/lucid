use dropshot::{EndpointTagPolicy, HttpError, HttpResponseCreated, HttpResponseHeaders, HttpResponseOk, RequestContext, TypedBody};
use dropshot_api_manager_types::api_versions;
use lucid_types::dto::{params, views};

api_versions!([
    (1, INITIAL_API),
]);

#[dropshot::api_description {
    tag_config = {
        allow_other_tags = false,
        policy = EndpointTagPolicy::ExactlyOne,
        tags = {
            "console-auth" = {
                description = "API for console authentication",
            }
        }
    }
}]
pub trait BeaconApi {
    type Context;

    #[endpoint {
        method = POST,
        path = "/v1/login/start",
        tags = ["console-auth"],
    }]
    async fn console_session_start(
        rqctx: RequestContext<Self::Context>,
        body: TypedBody<params::LoginParams>,
    ) -> Result<HttpResponseOk<views::LoginResponse>, HttpError>;

    #[endpoint {
        method = POST,
        path = "/v1/login/complete",
        tags = ["console-auth"],
    }]
    async fn console_session_login(
        rqctx: RequestContext<Self::Context>,
        body: TypedBody<params::LoginSessionParams>,
    ) -> Result<HttpResponseHeaders<HttpResponseCreated<()>>, HttpError>;

    #[endpoint {
        method = POST,
        path = "/v1/logout",
        tags = ["console-auth"],
    }]
    async fn console_session_logout(rqctx: RequestContext<Self::Context>)
        -> Result<HttpResponseOk<()>, HttpError>;
}
