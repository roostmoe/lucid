use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AuthLoginParams {
    /// The username or email of the user to authenticate as.
    pub username: String,

    /// The password of the user to authenticate as.
    pub password: String,
}
