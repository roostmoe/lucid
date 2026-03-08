use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request body for agent registration.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct RegisterAgentRequest {
    /// CSR in PEM format
    pub csr_pem: String,
    /// Hostname of the agent
    pub hostname: String,
}
