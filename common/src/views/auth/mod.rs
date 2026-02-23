use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Response for the login endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "token_type", rename_all = "PascalCase")]
pub enum AuthLoginResponse {
    /// The session cookie for the authenticated user. This cookie should be
    /// included in subsequent requests to authenticate the user.
    Session {
        /// CSRF token that must be included in X-CSRF-Token header for mutating requests
        csrf_token: String,
    },

    /// The access token for the authenticated user. This token should be
    /// included in the `Authorization` header of subsequent requests to
    /// authenticate the user.
    Bearer {
        /// The access token for the authenticated user.
        access_token: String,

        /// The refresh token for the user's new session.
        refresh_token: String,

        /// How long the token is valid for, in seconds. After this time has
        /// elapsed, the user will need to use the refresh token to obtain a
        /// new access token.
        expires_in: i64,
    },
}
