use std::path::PathBuf;

pub const AGENT_DATA_DIR: &str = "/var/lib/lucid/agent";

pub fn auth_key_path() -> PathBuf {
    PathBuf::from(AGENT_DATA_DIR).join("auth.key")
}

pub fn auth_cert_path() -> PathBuf {
    PathBuf::from(AGENT_DATA_DIR).join("auth.crt")
}

pub fn ca_cert_path() -> PathBuf {
    PathBuf::from(AGENT_DATA_DIR).join("ca.crt")
}
