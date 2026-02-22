use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Missing credentials")]
    MissingCredentials,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Credentials expired")]
    Expired,

    #[error("CSRF validation failed")]
    CsrfFailed,

    #[error(transparent)]
    Storage(#[from] lucid_db::storage::StoreError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
