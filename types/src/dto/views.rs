use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// JWT token response
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}
