use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("The request was unauthenticated")]
    Unauthenticated { internal_message: String },

    #[error("There was a problem with the request: {internal_message}")]
    Internal { internal_message: String },
}
