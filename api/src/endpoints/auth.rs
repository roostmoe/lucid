use dropshot::{HttpError, HttpResponseCreated, HttpResponseError, RequestContext, TypedBody, endpoint};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{context::LucidContext, error::AppError};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SignupBody {
    pub email: String,
    pub display_name: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SignupResponse {}

#[endpoint {
    method = POST,
    path = "/auth/signup",
}]
#[instrument(skip(rqctx), fields(request_id = rqctx.request_id), err(Debug))]
pub async fn sign_up(
    rqctx: RequestContext<LucidContext>,
    body: TypedBody<SignupBody>,
) -> Result<HttpResponseCreated<SignupResponse>, HttpError> {
    let resp = SignupResponse {};

    Ok(HttpResponseCreated(resp))
}
