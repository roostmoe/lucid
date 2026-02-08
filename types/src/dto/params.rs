use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    Local {
        password_hash: String,
    },

    External {
        external_id: String,
    },
}
