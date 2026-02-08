use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An organisation the user can log into.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct LoginOrganisation {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
}

/// Response for step 1 of the login flow.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct LoginResponse {
    pub organisations: Vec<LoginOrganisation>,
}
