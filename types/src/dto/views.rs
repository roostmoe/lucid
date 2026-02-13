use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An organisation the user can log into.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct LoginOrganisation {
    /// The ID of the organisation.
    pub id: Uuid,

    /// The API name of the organisation, which is unique across all
    /// organisations. This is used in API calls to identify the org.
    pub name: String,

    /// The display name of the organisation, which is not necessarily unique.
    /// This is only used for display purposes in the UI.
    pub display_name: String,
}

/// Response for step 1 of the login flow.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct LoginResponse {
    /// The organisations the user can log into. The user will select one of
    /// these to create a session for it.
    pub organisations: Vec<LoginOrganisation>,
}
