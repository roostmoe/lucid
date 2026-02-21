use axum::{Json, response::IntoResponse};
use lucid_common::{caller::CallerError, views::ApiErrorResponse};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error(transparent)]
    CallerError(#[from] lucid_common::caller::CallerError),

    #[error(transparent)]
    InternalAnyhow(#[from] anyhow::Error)
}

impl Into<ApiErrorResponse> for ApiError {
    fn into(self) -> ApiErrorResponse {
        ApiErrorResponse {
            code: match &self {
                Self::InternalAnyhow(_) => Some("InternalError".into()),
                Self::CallerError(ce) => match ce {
                    CallerError::Forbidden { .. } => Some("Forbidden".into()),
                    CallerError::Unauthorized { .. } => Some("Unauthorized".into()),
                    CallerError::Anyhow(_) => Some("InternalError".into()),
                }
            },

            message: match &self {
                Self::CallerError(ce) => match ce {
                    CallerError::Forbidden { .. } => "You do not have permission to perform this action.".into(),
                    CallerError::Unauthorized { .. } => "You are not authenticated to perform this action.".into(),
                    CallerError::Anyhow(_) => "Something went wrong on our end. Please try again later.".into(),
                }
                Self::InternalAnyhow(_) => "Something went wrong on our end. Please try again later.".into(),
            },

            #[cfg(debug_assertions)]
            details: Some(self.to_string()),

            #[cfg(not(debug_assertions))]
            details: None,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("Error returned by handler: {self}");

        let status_code = match &self {
            Self::CallerError(ce) => match ce {
                CallerError::Forbidden { .. } => axum::http::StatusCode::FORBIDDEN,
                CallerError::Unauthorized { .. } => axum::http::StatusCode::UNAUTHORIZED,
                CallerError::Anyhow(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            }
            Self::InternalAnyhow(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status_code, Json(Into::<ApiErrorResponse>::into(self))).into_response()
    }
}
