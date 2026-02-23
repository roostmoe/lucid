use clap::Parser;
use std::{net::SocketAddr, path::PathBuf};

#[derive(Clone, Debug, Parser)]
pub struct LucidApiConfig {
    #[clap(
        short,
        long,
        env = "LUCID_API_BIND_ADDR",
        default_value = "0.0.0.0:4000"
    )]
    pub bind_addr: SocketAddr,

    #[clap(
        long,
        env = "LUCID_API_PUBLIC_URL",
        default_value = "http://localhost:4000"
    )]
    pub public_url: String,

    #[clap(long, default_value_t = false)]
    pub dump_openapi: bool,

    #[clap(
        long,
        env = "LUCID_API_MONGODB_URI",
        default_value = "mongodb://localhost:27017/lucid"
    )]
    pub mongodb_uri: String,

    /// Ed25519 private key for signing session tokens (PEM format).
    ///
    /// Provide the key as an inline PEM string. For security, prefer using
    /// `signing_key_file` instead of embedding the key directly.
    ///
    /// Example PEM format:
    /// ```text
    /// -----BEGIN PRIVATE KEY-----
    /// MC4CAQAwBQYDK2VwBCIEI...
    /// -----END PRIVATE KEY-----
    /// ```
    ///
    /// Mutually exclusive with `signing_key_file`.
    #[clap(long, env = "LUCID_API_SIGNING_KEY")]
    pub signing_key: Option<String>,

    /// Path to Ed25519 private key file (PEM format).
    ///
    /// The file should contain a PEM-encoded PKCS#8 Ed25519 private key.
    /// Generate one using:
    /// ```bash
    /// openssl genpkey -algorithm ED25519 -out signing_key.pem
    /// ```
    ///
    /// Mutually exclusive with `signing_key`.
    #[clap(long, env = "LUCID_API_SIGNING_KEY_FILE")]
    pub signing_key_file: Option<PathBuf>,
}

impl LucidApiConfig {
    /// Get the signing key PEM data from either inline config or file.
    ///
    /// Checks `signing_key` first (inline PEM), then falls back to reading
    /// from `signing_key_file`. Returns an error if neither is configured.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Neither `signing_key` nor `signing_key_file` is configured
    /// - `signing_key_file` path doesn't exist or can't be read
    pub fn get_signing_key_pem(&self) -> anyhow::Result<String> {
        if let Some(ref key) = self.signing_key {
            return Ok(key.clone());
        }

        if let Some(ref path) = self.signing_key_file {
            return std::fs::read_to_string(path)
                .map_err(|e| anyhow::anyhow!("failed to read signing key file: {}", e));
        }

        Err(anyhow::anyhow!(
            "no signing key configured (set LUCID_API_SIGNING_KEY or LUCID_API_SIGNING_KEY_FILE)"
        ))
    }
}
