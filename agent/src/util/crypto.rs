use rcgen::{CertificateParams, DnType, KeyPair, PKCS_ED25519};

pub fn generate_keypair() -> Result<KeyPair, anyhow::Error> {
    KeyPair::generate_for(&PKCS_ED25519)
        .map_err(|e| anyhow::anyhow!("Failed to generate keypair: {}", e))
}

pub fn create_csr(key_pair: &KeyPair, hostname: &str) -> Result<String, anyhow::Error> {
    let mut params = CertificateParams::default();
    params.distinguished_name.push(DnType::CommonName, hostname);

    let csr = params
        .serialize_request(key_pair)
        .map_err(|e| anyhow::anyhow!("Failed to create CSR: {}", e))?;

    Ok(csr
        .pem()
        .map_err(|e| anyhow::anyhow!("Failed to encode CSR: {}", e))?)
}
