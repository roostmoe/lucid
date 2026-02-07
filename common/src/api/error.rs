use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("There was a problem with the request: {0}")]
    Internal(String)
}
