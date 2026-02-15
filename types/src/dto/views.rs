use chrono::{DateTime, Utc};
use lucid_uuid_kinds::HostUuid;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Pagination metadata for API responses
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct PaginationMeta {
    pub total_items: i64,
    pub total_pages: i64,
    pub current_page: i64,
    pub page_size: i64,
}

/// JWT token response
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// Host view for API responses
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct HostView {
    pub id: HostUuid,

    pub hostname: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Host list response with pagination
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct HostListResponse {
    pub hosts: Vec<HostView>,
    pub pagination: PaginationMeta,
}
