//! Output views for the various functions within Lucid.

use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

mod auth;
pub use auth::*;

mod host;
pub use host::*;

mod user;
pub use user::*;

/// Parameters for paginating through a list of records. This is used by the
/// various list endpoints to allow clients to paginate through large sets of
/// records.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedList<T> {
    pub items: Vec<T>,

    /// The next page token, if any. This is acquired by requesting a paginated
    /// set of records and looking at the `next_token` or `prev_token` field.
    pub next_token: Option<String>,

    /// The maximum number of results to return.
    pub limit: Option<u64>,
}

/// An error response for an API endpoint. This is used to return errors to the
/// client in a consistent format.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ApiErrorResponse {
    /// An optional error code that can be used to identify the type of error
    /// that occurred.
    pub code: Option<String>,

    /// A human-readable message describing the error that occurred.
    pub message: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}
