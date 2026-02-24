use axum::{
    Router,
    extract::MatchedPath,
    http::{HeaderName, Request},
    routing::get,
};
use lucid_common::views::ApiErrorResponse;
use tower::ServiceBuilder;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::info_span;
use utoipa::{
    PartialSchema, ToSchema,
    openapi::{Contact, Info, License, OpenApi, RefOr, Response, path::Operation},
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    auth::AuthManager, config::LucidApiConfig, context::ApiContext, error::ApiError, handlers,
};

const REQUEST_ID_HEADER: &str = "x-request-id";

pub async fn make(cfg: LucidApiConfig) -> (Router, OpenApi) {
    // TODO: Wire up auth providers properly
    let auth_manager = AuthManager::new();

    let context = ApiContext::new(cfg.clone(), auth_manager)
        .await
        .expect("Failed to initialize API context");

    let cors_public_url = cfg.public_url.clone();
    let x_request_id = HeaderName::from_static(REQUEST_ID_HEADER);
    let middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            MakeRequestUuid,
        ))
        .layer(
            TraceLayer::new_for_http().make_span_with(|req: &Request<_>| {
                // Log the request ID as generated
                let request_id = req.headers().get(REQUEST_ID_HEADER);
                let span = info_span!(
                    "http_request",
                    http.request.method = req.method().to_string(),
                    http.request.id = Option::<&str>::None,
                    http.route = Option::<&str>::None,
                    url.full = Option::<&str>::None,
                );

                span.record("url.full", req.uri().path());

                if let Some(request_id) = request_id {
                    span.record("http.request.id", request_id.to_str().unwrap());
                }

                if let Some(path) = req.extensions().get::<MatchedPath>() {
                    span.record("http.route", path.as_str());
                }

                span
            }),
        )
        .layer(
            CorsLayer::new()
                .allow_credentials(true)
                .allow_origin(
                    AllowOrigin::predicate(move |origin, request_parts| {
                        if request_parts.uri.path().starts_with("/.well-known") {
                            return true
                        }
                        origin.as_bytes().starts_with(cors_public_url.as_bytes())
                    })
                ),
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
                        .build(),
                ))
                .contact(Some(
                    Contact::builder()
                        .name(Some("Roost team"))
                        .email("hello@roost.moe".into())
                        .url("https://github.com/roostmoe/lucid".into())
                        .build(),
                )),
        )
        .build();

    let (r, mut a) = OpenApiRouter::with_openapi(openapi)
        .routes(routes!(handlers::activation_keys::create_activation_key))
        .routes(routes!(handlers::activation_keys::list_activation_keys))
        .routes(routes!(handlers::activation_keys::get_activation_key))
        .routes(routes!(handlers::activation_keys::delete_activation_key))
        .routes(routes!(handlers::auth::auth_login))
        .routes(routes!(handlers::auth::auth_logout))
        .routes(routes!(handlers::auth::auth_whoami))
        .routes(routes!(handlers::hosts::list_hosts))
        .routes(routes!(handlers::hosts::get_host))
        .routes(routes!(handlers::jwks::get_jwks))
        .routes(routes!(handlers::jwks::get_openid_configuration))
        .route("/healthz", get(handlers::health_check))
        .fallback(not_found_handler)
        .layer(middleware)
        .with_state(context)
        .split_for_parts();

    a.components.as_mut().unwrap().schemas.insert(
        ApiErrorResponse::name().to_string(),
        ApiErrorResponse::schema(),
    );
    a.paths.paths.iter_mut().for_each(|(_path, item)| {
        apply_default_errors(&mut item.get);
        apply_default_errors(&mut item.post);
        apply_default_errors(&mut item.patch);
        apply_default_errors(&mut item.put);
        apply_default_errors(&mut item.delete);
        apply_default_errors(&mut item.trace);
        apply_default_errors(&mut item.head);
        apply_default_errors(&mut item.options);
    });

    (r, a)
}

async fn not_found_handler() -> ApiError {
    ApiError::not_found()
}

fn apply_default_errors(item: &mut Option<Operation>) {
    if let Some(item) = item {
        item.responses
            .responses
            .insert("400".into(), error_resp("Client or validation error"));
        item.responses
            .responses
            .insert("401".into(), error_resp("Unauthorized"));
        item.responses
            .responses
            .insert("403".into(), error_resp("Forbidden"));
        item.responses
            .responses
            .insert("404".into(), error_resp("Not found"));
        item.responses
            .responses
            .insert("500".into(), error_resp("Internal server error"));
    }
}

fn error_resp(summary: &str) -> RefOr<Response> {
    RefOr::Ref(
        utoipa::openapi::Ref::builder()
            .summary(summary)
            .ref_location_from_schema_name(ApiErrorResponse::name())
            .build(),
    )
}
