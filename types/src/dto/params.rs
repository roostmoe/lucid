use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct OrganisationCreate {
    pub name: String,
    pub display_name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct UserCreate {
    pub email: String,
    pub auth_mode: UserCreateAuthMode,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub enum UserCreateAuthMode {
    Local { password_hash: String },

    External { external_id: String },
}

// ---------------------------------------------------------------------------
// Login
// ---------------------------------------------------------------------------

/// Step 1: validate credentials and list the user's organisations.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct LoginParams {
    pub email: String,
    pub password: String,
}

/// Step 2: validate credentials again and create a session for the chosen org.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct LoginSessionParams {
    pub email: String,
    pub password: String,
    pub organisation_id: Uuid,
}
