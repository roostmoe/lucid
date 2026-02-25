use axum::{Json, response::IntoResponse};
use lucid_common::{caller::CallerError, views::ApiErrorResponse};
use lucid_db::storage::StoreError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Not found")]
    NotFound,

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error(transparent)]
    Storage(#[from] lucid_db::storage::StoreError),

    #[error(transparent)]
    CallerError(#[from] lucid_common::caller::CallerError),

    #[error(transparent)]
    InternalAnyhow(#[from] anyhow::Error),
}

impl ApiError {
    pub fn not_found() -> Self {
        Self::NotFound
    }

    pub fn service_unavailable(msg: impl Into<String>) -> Self {
        Self::ServiceUnavailable(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::CallerError(CallerError::unauthorized(Some(msg.into())))
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

impl From<ApiError> for ApiErrorResponse {
    fn from(err: ApiError) -> Self {
        ApiErrorResponse {
            code: match &err {
                ApiError::NotFound => Some("NotFound".into()),
                ApiError::ServiceUnavailable(_) => Some("ServiceUnavailable".into()),
                ApiError::Internal(_) => Some("InternalError".into()),
                ApiError::Storage(se) => match se {
                    StoreError::NotFound => Some("NotFound".into()),
                    StoreError::PermissionDenied => Some("Forbidden".into()),
                    _ => Some("InternalError".into()),
                },
                ApiError::InternalAnyhow(_) => Some("InternalError".into()),
                ApiError::CallerError(ce) => match ce {
                    CallerError::Forbidden { .. } => Some("Forbidden".into()),
                    CallerError::Unauthorized { .. } => Some("Unauthorized".into()),
                    CallerError::Anyhow(_) => Some("InternalError".into()),
                },
            },

            message: match &err {
                ApiError::NotFound => "The requested resource was not found.".into(),
                ApiError::ServiceUnavailable(msg) => msg.clone(),
                ApiError::Internal(msg) => msg.clone(),
                ApiError::Storage(se) => match se {
                    StoreError::NotFound => "The requested resource was not found.".into(),
                    StoreError::PermissionDenied => {
                        "You do not have permission to perform this action.".into()
                    }
                    _ => "Something went wrong on our end. Please try again later.".into(),
                },
                ApiError::CallerError(ce) => match ce {
                    CallerError::Forbidden { .. } => {
                        "You do not have permission to perform this action.".into()
                    }
                    CallerError::Unauthorized { .. } => {
                        "You are not authenticated to perform this action.".into()
                    }
                    CallerError::Anyhow(_) => {
                        "Something went wrong on our end. Please try again later.".into()
                    }
                },
                ApiError::InternalAnyhow(_) => {
                    "Something went wrong on our end. Please try again later.".into()
                }
            },

            #[cfg(debug_assertions)]
            details: Some(err.to_string()),

            #[cfg(not(debug_assertions))]
            details: None,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("Error returned by handler: {self}");

        let status_code = match &self {
            Self::NotFound => axum::http::StatusCode::NOT_FOUND,
            Self::ServiceUnavailable(_) => axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Self::Internal(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::Storage(se) => match se {
                StoreError::NotFound => axum::http::StatusCode::NOT_FOUND,
                StoreError::PermissionDenied => axum::http::StatusCode::FORBIDDEN,
                _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            },
            Self::CallerError(ce) => match ce {
                CallerError::Forbidden { .. } => axum::http::StatusCode::FORBIDDEN,
                CallerError::Unauthorized { .. } => axum::http::StatusCode::UNAUTHORIZED,
                CallerError::Anyhow(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            },
            Self::InternalAnyhow(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status_code, Json(Into::<ApiErrorResponse>::into(self))).into_response()
    }
}
