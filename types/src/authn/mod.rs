use std::fmt;

use anyhow::anyhow;
use serde_with::{DeserializeFromStr, SerializeDisplay};


/// Identifies a particular external authentication scheme.
///
/// This is a closed set of schemes. Adding a new scheme requires updating this
/// enum, which ensures all consumers (config parsing, authn, audit logging)
/// handle it explicitly.
#[derive(
    Clone, Copy, Debug, DeserializeFromStr, Eq, PartialEq, SerializeDisplay,
)]
pub enum SchemeName {
    /// Session cookie authentication (web console)
    SessionCookie,
}

impl SchemeName {
    /// String representation used in config files and logs.
    pub fn as_str(&self) -> &'static str {
        match self {
            SchemeName::SessionCookie => "session_cookie",
        }
    }
}

impl fmt::Display for SchemeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for SchemeName {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "session_cookie" => Ok(SchemeName::SessionCookie),
            _ => Err(anyhow!("unsupported authn scheme: {:?}", s)),
        }
    }
}
