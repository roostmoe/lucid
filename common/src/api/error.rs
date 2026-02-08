use dropshot::HttpError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::api::ResourceType;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Object of type {lookup_type:?} not found: {resource_type}")]
    ObjectNotFound { resource_type: ResourceType, lookup_type: LookupType },

    #[error("The request was unauthenticated")]
    Unauthenticated { internal_message: String },

    #[error("The requested resource was not found: {internal_message} ({error_code:?})")]
    NotFound { error_code: Option<String>, internal_message: String },

    #[error("There was a problem with the request: {internal_message}")]
    Internal { internal_message: String },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum LookupType {
    /// a specific id was requested
    ById(Uuid),

    /// object selected by criteria that would be confusing to call an id
    ByOther(String),
}

impl From<Error> for HttpError {
    fn from(error: Error) -> Self {
        match error {
            Error::Unauthenticated { internal_message } => HttpError {
                status_code: dropshot::ErrorStatusCode::UNAUTHORIZED,
                error_code: Some(String::from("Unauthorized")),
                external_message: String::from("credentials missing or invalid"),
                internal_message,
                headers: None,
            },

            Error::ObjectNotFound { resource_type: t, lookup_type: lt } => {
                let message = match lt {
                    LookupType::ById(id) => {
                        format!("{} with id \"{}\"", t, id)
                    }
                    LookupType::ByOther(msg) => msg,
                };

                HttpError::for_client_error(
                    Some(String::from("ObjectNotFound")),
                    dropshot::ClientErrorStatusCode::NOT_FOUND,
                    format!("not found: {}", message),
                )
            }

            Error::NotFound { error_code, internal_message } =>
                HttpError::for_not_found(error_code, internal_message),

            Error::Internal { internal_message } =>
                HttpError::for_internal_error(internal_message),
        }
    }
}
