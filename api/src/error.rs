use axum::{Json, response::IntoResponse};
use lucid_common::views::ApiErrorResponse;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error(transparent)]
    InternalAnyhow(#[from] anyhow::Error)
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("Error returned by handler: {self}");

        let status_code = match &self {
            &Self::InternalAnyhow(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        (
            status_code,
            Json(ApiErrorResponse {
                code: match &self {
                    &Self::InternalAnyhow(_) => Some("InternalError".into()),
                },

                message: match &self {
                    &Self::InternalAnyhow(_) => "Something went wrong on our end. Please try again later.".into(),
                },

                #[cfg(debug_assertions)]
                details: Some(self.to_string()),

                #[cfg(not(debug_assertions))]
                details: None,
            })
        ).into_response()
    }
}
