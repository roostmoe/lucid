use dropshot::HttpError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::api::ResourceType;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Object of type {lookup_type:?} not found: {resource_type}")]
    ObjectNotFound {
        resource_type: ResourceType,
        lookup_type: LookupType,
    },

    #[error("Object of type {type_name} already exists")]
    ObjectAlreadyExists {
        type_name: ResourceType,
        object_name: String,
    },

    #[error("The request was unauthenticated")]
    Unauthenticated { internal_message: String },

    #[error("Forbidden")]
    Forbidden,

    #[error("The requested resource was not found: {internal_message} ({error_code:?})")]
    NotFound {
        error_code: Option<String>,
        internal_message: String,
    },

    #[error("There was a problem with the request: {internal_message}")]
    Internal { internal_message: String },

    #[error("There was a problem with the request: {internal_message}")]
    InternalAnyhow {
        internal_message: String,
        #[source]
        source: anyhow::Error,
    },
}

impl Error {
    pub fn internal_error(internal_message: &str) -> Error {
        Error::Internal {
            internal_message: internal_message.to_owned(),
        }
    }

    pub fn internal_anyhow(message: String, source: anyhow::Error) -> Self {
        Self::InternalAnyhow {
            internal_message: message,
            source,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum LookupType {
    /// a specific id was requested
    ById(Uuid),

    /// object selected by criteria that would be confusing to call an id
    ByOther(String),
}

impl LookupType {
    pub fn into_not_found(&self, resource_type: ResourceType) -> Error {
        Error::ObjectNotFound {
            resource_type,
            lookup_type: self.clone(),
        }
    }
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

            Error::Forbidden => HttpError {
                status_code: dropshot::ErrorStatusCode::FORBIDDEN,
                error_code: Some(String::from("Forbidden")),
                external_message: String::from("insufficient permissions"),
                internal_message: "Insufficient permissions".into(),
                headers: None,
            },

            Error::ObjectNotFound {
                resource_type: t,
                lookup_type: lt,
            } => {
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

            Error::ObjectAlreadyExists {
                type_name,
                object_name,
            } => HttpError::for_client_error(
                Some(String::from("ObjectAlreadyExists")),
                dropshot::ClientErrorStatusCode::CONFLICT,
                format!("{} with name \"{}\" already exists", type_name, object_name),
            ),

            Error::NotFound {
                error_code,
                internal_message,
            } => HttpError::for_not_found(error_code, internal_message),

            Error::Internal { internal_message } => HttpError::for_internal_error(internal_message),
            Error::InternalAnyhow {
                internal_message, ..
            } => HttpError::for_internal_error(internal_message),
        }
    }
}
