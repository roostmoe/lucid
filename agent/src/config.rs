use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    pub data_dir: PathBuf,
}

impl AgentConfig {
    pub fn from_file(path: PathBuf) -> anyhow::Result<Self> {
        let config_str = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Failed to read config file {}: {}", path.display(), e))?;
        let config = toml::from_str(&config_str).map_err(|e| {
            anyhow::anyhow!("Failed to parse config file {}: {}", path.display(), e)
        })?;
        Ok(config)
    }

    /// The path to the agent's private key file (PEM format)
    pub fn auth_key_path(&self) -> PathBuf {
        self.data_dir.join("auth.key")
    }

    /// The path to the agent's certificate file (PEM format) issued by the
    /// server.
    pub fn auth_cert_path(&self) -> PathBuf {
        self.data_dir.join("auth.crt")
    }

    /// The path to the CA certificate file (PEM format) used to verify the
    /// server's TLS certificate.
    pub fn ca_cert_path(&self) -> PathBuf {
        self.data_dir.join("ca.crt")
    }
}
