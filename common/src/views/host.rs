use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct Host {
    /// The unique identifier for this host.
    pub id: Ulid,

    /// Hostname of the machine. This is a human-readable identifier for the
    /// host, and is not guaranteed to be unique.
    pub hostname: String,

    /// Network interfaces associated with this host. This is a one-to-many
    /// relationship, as a host can have multiple network interfaces.
    pub ifaces: Vec<NetworkInterface>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct NetworkInterface {
    /// The unique identifier for this network interface.
    pub id: Ulid,

    /// The unique identifier for the host that this network interface is
    /// associated with.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_id: Option<Ulid>,

    /// The name of the network interface.
    pub iface: String,

    /// The state of the network interface. This can be "up", "down", or
    /// "unknown".
    pub state: NetworkInterfaceState,

    /// The IP addresses associated with this network interface.
    pub ip_addrs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub enum NetworkInterfaceState {
    /// The network interface is up and operational.
    Up,

    /// The network interface is down and not operational.
    Down,
    #[default]

    /// The state of the network interface is unknown. This can occur if the
    /// state cannot be determined for some reason.
    Unknown,
}
