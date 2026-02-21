//! Input parameters for the various functions within Lucid.

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

mod auth;
pub use auth::*;

/// Parameters for paginating through a list of records. This is used by the
/// various list endpoints to allow clients to paginate through large sets of
/// records.
#[derive(Debug, Clone, Deserialize, Serialize, IntoParams, ToSchema)]
pub struct PaginationParams {
    /// The next page token, if any. This is acquired by requesting a paginated
    /// set of records and looking at the `next_token` or `prev_token` field.
    pub next_token: Option<String>,

    /// The maximum number of results to return.
    pub limit: Option<u64>,
}
