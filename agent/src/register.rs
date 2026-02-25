use crate::config::{AGENT_DATA_DIR, auth_cert_path, auth_key_path, ca_cert_path};
use crate::crypto::{create_csr, generate_keypair};
use anyhow::{Context, Result, bail};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

#[derive(Deserialize)]
struct JwtClaims {
    iss: String,
    // other fields we don't need
}

#[derive(Serialize)]
struct RegisterRequest {
    csr_pem: String,
    hostname: String,
}

#[derive(Deserialize)]
struct RegisterResponse {
    agent_id: String,
    certificate_pem: String,
    ca_certificate_pem: String,
    expires_at: String,
    api_base_url: String,
}

pub async fn register(token: &str) -> Result<()> {
    // 1. Check if already registered
    if auth_key_path().exists() {
        bail!(
            "Agent already registered. Delete {} to re-register.",
            auth_key_path().display()
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
    let client = Client::new();
    let response = client
        .post(format!("{}/api/v1/agents/register", api_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&RegisterRequest { csr_pem, hostname })
        .send()
        .await
        .context("Failed to send registration request")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        bail!("Registration failed ({}): {}", status, body);
    }

    let reg_response: RegisterResponse = response
        .json()
        .await
        .context("Failed to parse registration response")?;

    // 7. Create directory if needed
    fs::create_dir_all(AGENT_DATA_DIR).context(format!("Failed to create {}", AGENT_DATA_DIR))?;

    // 8. Write files atomically
    write_file_atomic(&auth_key_path(), &private_key_pem, 0o600)?;
    write_file_atomic(&auth_cert_path(), &reg_response.certificate_pem, 0o644)?;
    write_file_atomic(&ca_cert_path(), &reg_response.ca_certificate_pem, 0o644)?;

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
