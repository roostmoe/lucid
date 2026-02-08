pub mod login;

use dropshot::{HttpError, HttpResponseOk, RequestContext, endpoint};

use crate::context::Context;

#[endpoint {
    method = GET,
    path = "/healthz"
}]
pub async fn health_check(
    _rqctx: RequestContext<Context>,
) -> Result<HttpResponseOk<()>, HttpError> {
    Ok(HttpResponseOk(()))
}
