use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateHostParams {
    /// The hostname of the host to create. This should be a fully qualified
    /// domain name (FQDN) or an IP address.
    pub hostname: String,
}
