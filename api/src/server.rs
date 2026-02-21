use axum::{Router, extract::MatchedPath, http::{HeaderName, HeaderValue, Request}};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer}, trace::TraceLayer};
use tracing::{error, info, info_span};
use utoipa::openapi::{Contact, Info, License, OpenApi};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{config::LucidApiConfig, context::ApiContext, handlers};

const REQUEST_ID_HEADER: &str = "x-request-id";

pub async fn make(cfg: LucidApiConfig) -> (Router, OpenApi) {
    let context = ApiContext::new(cfg.clone());

    let x_request_id = HeaderName::from_static(REQUEST_ID_HEADER);
    let middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            MakeRequestUuid,
        ))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &Request<_>| {
                    // Log the request ID as generated
                    let request_id = req.headers().get(REQUEST_ID_HEADER);
                    let span = info_span!(
                        "http_request",
                        method = req.method().to_string(),
                        request_id = Option::<&str>::None,
                        path = Option::<&str>::None,
                    );

                    if let Some(request_id) = request_id {
                        span.record("request_id", request_id.to_str().unwrap());
                    };

                    if let Some(path) = req.extensions().get::<MatchedPath>() {
                        span.record("path", path.as_str())
                    } else {
                        span.record("path", req.uri().path())
                    };

                    span
                }),
        )
        .layer(
            CorsLayer::new()
                .allow_credentials(true)
                .allow_origin(cfg.public_url.parse::<HeaderValue>().unwrap())
        )
        .layer(PropagateRequestIdLayer::new(x_request_id));

    let openapi = OpenApi::builder()
        .info(
            Info::builder()
                .title("Lucid API Reference")
                .version(env!("CARGO_PKG_VERSION"))
                .license(Some(
                    License::builder()
                        .name("Apache 2.0 License")
                        .identifier(Some(env!("CARGO_PKG_LICENSE")))
                        .url("https://github.com/roostmoe/lucid/blob/main/LICENSE".into())
                        .build()
                ))
                .contact(Some(
                    Contact::builder()
                        .name(Some("Roost team"))
                        .email("hello@roost.moe".into())
                        .url("https://github.com/roostmoe/lucid".into())
                        .build()
                ))
        )
        .build();

    OpenApiRouter::with_openapi(openapi)
        .routes(routes!(handlers::auth::auth_login))
        .routes(routes!(handlers::auth::auth_logout))
        .routes(routes!(handlers::auth::auth_whoami))
        .layer(middleware)
        .with_state(context)
        .split_for_parts()
}
