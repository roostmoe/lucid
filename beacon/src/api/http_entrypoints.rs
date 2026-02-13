use crate::context::{Context as Ctx};
use dropshot::{ApiDescription, HttpError, HttpResponseCreated, HttpResponseHeaders, HttpResponseOk, RequestContext, TypedBody};

use lucid_beacon_api::{BeaconApi, beacon_api_mod};
use lucid_types::dto::{params, views};

type BeaconApiDescription = ApiDescription<Ctx>;

pub(crate) fn api() -> BeaconApiDescription {
    beacon_api_mod::api_description::<BeaconApiImpl>()
        .expect("registering entrypoints")
}

enum BeaconApiImpl {}

impl BeaconApi for BeaconApiImpl {
    type Context = Ctx;

    async fn console_session_start(
        rqctx: RequestContext<Self::Context>,
        body: TypedBody<params::LoginParams>,
    ) -> Result<HttpResponseOk<views::LoginResponse>, HttpError> {
        Ok(rqctx
            .context()
            .beacon
            .console_session_start(body.into_inner())
            .await?)
    }

    async fn console_session_login(
        rqctx: RequestContext<Self::Context>,
        body: TypedBody<params::LoginSessionParams>,
    ) -> Result<HttpResponseHeaders<HttpResponseCreated<()>>, HttpError> {
        Ok(rqctx
            .context()
            .beacon
            .console_session_login(body.into_inner())
            .await?)
    }

    async fn console_session_logout(
        rqctx: RequestContext<Self::Context>,
    ) -> Result<HttpResponseOk<()>, HttpError> {
        Ok(HttpResponseOk(()))
    }
}
