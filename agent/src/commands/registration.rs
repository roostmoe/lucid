use crate::client::ApiClient;
use crate::config::AgentConfig;
use crate::util::crypto::{create_csr, generate_keypair};
use anyhow::{Context, Result, bail};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

#[derive(Deserialize)]
struct JwtClaims {
    iss: String,
    // other fields we don't need
}

pub async fn register(token: &str, config: AgentConfig) -> Result<()> {
    // 1. Check if already registered
    if config.auth_key_path().exists() {
        bail!(
            "Agent already registered. Delete {} to re-register.",
            config.auth_key_path().display()
        );
    }

    // 2. Decode JWT to get API URL
    let api_url = extract_issuer_from_jwt(token)?;
    println!("Registering with API at: {}", api_url);

    // 3. Generate keypair
    let key_pair = generate_keypair()?;
    let private_key_pem = key_pair.serialize_pem();

    // 4. Get hostname
    let hostname = hostname::get()
        .context("Failed to get hostname")?
        .to_string_lossy()
        .to_string();
    println!("Hostname: {}", hostname);

    // 5. Create CSR
    let csr_pem = create_csr(&key_pair, &hostname)?;

    // 6. Make registration request
    let client = ApiClient::new(api_url, None, None, None)
        .context("Failed to create API client")?;

    let reg_response = client.register(
        token.to_string(),
        csr_pem,
        hostname,
    )
        .await
        .context("Failed to register agent")?;

    // 7. Create directory if needed
    fs::create_dir_all(config.data_dir.clone())
        .context(format!("Failed to create {:?}", config.data_dir.clone()))?;

    // 8. Write files atomically
    write_file_atomic(&config.auth_key_path(), &private_key_pem, 0o600)?;
    write_file_atomic(&config.auth_cert_path(), &reg_response.certificate_pem, 0o644)?;
    write_file_atomic(&config.ca_cert_path(), &reg_response.ca_certificate_pem, 0o644)?;

    println!("✓ Registered as agent {}", reg_response.agent_id);
    println!("  Certificate expires: {}", reg_response.expires_at);
    println!("  API URL: {}", reg_response.api_base_url);

    Ok(())
}

fn extract_issuer_from_jwt(token: &str) -> Result<String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        bail!("Invalid JWT format");
    }

    let claims_b64 = parts[1];
    let claims_json = URL_SAFE_NO_PAD
        .decode(claims_b64)
        .context("Failed to decode JWT claims")?;

    let claims: JwtClaims =
        serde_json::from_slice(&claims_json).context("Failed to parse JWT claims")?;

    Ok(claims.iss)
}

fn write_file_atomic(path: &Path, content: &str, mode: u32) -> Result<()> {
    // Write to temp file first, then rename for atomicity
    let temp_path = path.with_extension("tmp");

    fs::write(&temp_path, content).context(format!("Failed to write {}", temp_path.display()))?;

    // Set permissions
    let mut perms = fs::metadata(&temp_path)?.permissions();
    perms.set_mode(mode);
    fs::set_permissions(&temp_path, perms)?;

    // Atomic rename
    fs::rename(&temp_path, path).context(format!("Failed to rename to {}", path.display()))?;

    Ok(())
}

pub fn unregister(config: AgentConfig) -> anyhow::Result<()> {
    let mut removed = false;

    for path in [config.auth_key_path(), config.auth_cert_path(), config.ca_cert_path()] {
        if path.exists() {
            std::fs::remove_file(&path)?;
            println!("Removed: {}", path.display());
            removed = true;
        }
    }

    if removed {
        println!("✓ Local credentials removed");
        println!("  Note: The agent is still registered on the server.");
        println!("  An admin must revoke it via the API.");
    } else {
        println!("No credentials found - agent was not registered.");
    }

    Ok(())
}
