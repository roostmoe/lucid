use axum::Router;

use crate::{config::LucidApiConfig, context::ApiContext};

pub async fn make(cfg: LucidApiConfig) -> Router {
    let context = ApiContext::new(cfg);

    Router::new()
        .with_state(context)
}
