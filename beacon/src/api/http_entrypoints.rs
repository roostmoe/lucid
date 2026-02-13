use crate::context::{Context as Ctx};
use dropshot::{ApiDescription, HttpError, HttpResponseCreated, HttpResponseOk, RequestContext};

use lucid_beacon_api::{BeaconApi, beacon_api_mod};
use lucid_types::dto::views;

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
    ) -> Result<HttpResponseOk<views::LoginResponse>, HttpError> {
        Ok(HttpResponseOk(views::LoginResponse {
            organisations: vec![],
        }))
    }

    async fn console_session_login(
        rqctx: RequestContext<Self::Context>,
    ) -> Result<HttpResponseCreated<()>, HttpError> {
        Ok(HttpResponseCreated(()))
    }

    async fn console_session_logout(
        rqctx: RequestContext<Self::Context>,
    ) -> Result<HttpResponseOk<()>, HttpError> {
        Ok(HttpResponseOk(()))
    }
}
