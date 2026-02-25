use std::sync::Arc;

use async_trait::async_trait;
use base64::Engine;
use chrono::{DateTime, Duration, Utc};
use lucid_common::caller::Caller;
use lucid_db::{
    models::DbCa,
    storage::{CaStore, Storage},
};
use mongodb::bson::oid::ObjectId;
use rcgen::{CertificateParams, CertificateSigningRequestParams, DistinguishedName, KeyPair};
use sha2::{Digest, Sha256};
use x509_parser::prelude::*;

use super::ca::{CaError, CaInfo, CertificateAuthority, SignedCertificate};
use crate::crypto::aes;

const AGENT_CERT_VALIDITY_HOURS: i64 = 24;
const CA_CERT_VALIDITY_YEARS: i64 = 10;

pub struct EncryptedCa {
    storage: Arc<dyn Storage>,
    encryption_key: [u8; 32],
}

impl EncryptedCa {
    pub fn new(storage: Arc<dyn Storage>, encryption_key: [u8; 32]) -> Self {
        Self {
            storage,
            encryption_key,
        }
    }

    /// Load encryption key from base64 environment variable.
    pub fn encryption_key_from_env() -> Result<[u8; 32], CaError> {
        let key_b64 = std::env::var("LUCID_CA_ENCRYPTION_KEY")
            .map_err(|_| CaError::Encryption("LUCID_CA_ENCRYPTION_KEY not set".into()))?;

        let key_bytes = base64::engine::general_purpose::STANDARD
            .decode(&key_b64)
            .map_err(|e| CaError::Encryption(format!("Invalid base64: {}", e)))?;

        key_bytes
            .try_into()
            .map_err(|_| CaError::Encryption("Key must be exactly 32 bytes".into()))
    }

    /// Decrypt the CA private key from storage.
    async fn decrypt_private_key(&self, ca: &DbCa) -> Result<KeyPair, CaError> {
        // Use CA ID as AAD to prevent ciphertext transplantation
        let aad = ca
            .id
            .as_ref()
            .ok_or_else(|| CaError::Decryption("CA missing ID".into()))?
            .to_hex()
            .as_bytes()
            .to_vec();

        let private_key_pem = aes::decrypt(&self.encryption_key, &ca.encrypted_private_key, &aad)
            .map_err(|e| CaError::Decryption(e.to_string()))?;

        let private_key_str = std::str::from_utf8(&private_key_pem)
            .map_err(|e| CaError::Decryption(format!("Invalid UTF-8: {}", e)))?;

        KeyPair::from_pem(private_key_str)
            .map_err(|e| CaError::Decryption(format!("Invalid private key PEM: {}", e)))
    }
}

#[async_trait]
impl CertificateAuthority for EncryptedCa {
    async fn sign_csr(
        &self,
        csr_pem: &str,
        agent_id: ObjectId,
    ) -> Result<SignedCertificate, CaError> {
        // Load CA from store
        let ca = CaStore::list(self.storage.as_ref(), Caller::System)
            .await
            .map_err(|e| CaError::Storage(e.to_string()))?
            .into_iter()
            .next()
            .ok_or(CaError::NotInitialized)?;

        // Decrypt CA private key
        let ca_key_pair = self.decrypt_private_key(&ca).await?;

        // Parse CA certificate params to reconstruct Certificate
        let ca_params = CertificateParams::from_ca_cert_pem(&ca.cert_pem)
            .map_err(|e| CaError::Generation(format!("Failed to parse CA cert: {}", e)))?;

        // Self-sign with the CA key to recreate the Certificate object
        let ca_cert = ca_params
            .self_signed(&ca_key_pair)
            .map_err(|e| CaError::Generation(format!("Failed to reconstruct CA cert: {}", e)))?;

        // Parse CSR
        let mut csr = CertificateSigningRequestParams::from_pem(csr_pem)
            .map_err(|e| CaError::InvalidCsr(format!("Failed to parse CSR: {}", e)))?;

        // Override DN with agent_id as CN
        let mut dn = DistinguishedName::new();
        dn.push(rcgen::DnType::CommonName, agent_id.to_hex());
        csr.params.distinguished_name = dn;

        // Set validity period (24 hours)
        let issued_at = Utc::now();
        let expires_at = issued_at + Duration::hours(AGENT_CERT_VALIDITY_HOURS);

        csr.params.not_before = ::time::OffsetDateTime::from_unix_timestamp(issued_at.timestamp())
            .map_err(|e| CaError::Generation(format!("Invalid timestamp: {}", e)))?;
        csr.params.not_after = ::time::OffsetDateTime::from_unix_timestamp(expires_at.timestamp())
            .map_err(|e| CaError::Generation(format!("Invalid timestamp: {}", e)))?;

        // Set key usage for client authentication
        csr.params.key_usages = vec![
            rcgen::KeyUsagePurpose::DigitalSignature,
            rcgen::KeyUsagePurpose::KeyEncipherment,
        ];
        csr.params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ClientAuth];

        // Ensure NOT a CA cert
        csr.params.is_ca = rcgen::IsCa::NoCa;

        // Sign the CSR with the CA
        let cert = csr
            .signed_by(&ca_cert, &ca_key_pair)
            .map_err(|e| CaError::Generation(format!("Failed to sign certificate: {}", e)))?;

        let cert_pem = cert.pem();

        Ok(SignedCertificate {
            cert_pem,
            issued_at,
            expires_at,
        })
    }

    async fn get_ca_cert_pem(&self) -> Result<String, CaError> {
        let ca = CaStore::list(self.storage.as_ref(), Caller::System)
            .await
            .map_err(|e| CaError::Storage(e.to_string()))?
            .into_iter()
            .next()
            .ok_or(CaError::NotInitialized)?;

        Ok(ca.cert_pem)
    }

    async fn get_ca_info(&self) -> Result<CaInfo, CaError> {
        let ca = CaStore::list(self.storage.as_ref(), Caller::System)
            .await
            .map_err(|e| CaError::Storage(e.to_string()))?
            .into_iter()
            .next()
            .ok_or(CaError::NotInitialized)?;

        // Parse certificate to get dates
        let cert_der = pem_rfc7468::decode_vec(ca.cert_pem.as_bytes())
            .map_err(|e| CaError::Generation(format!("Failed to decode CA cert PEM: {}", e)))?
            .1;

        let (_, cert) = X509Certificate::from_der(&cert_der)
            .map_err(|e| CaError::Generation(format!("Failed to parse CA cert: {}", e)))?;

        let issued_at = DateTime::from_timestamp(cert.validity().not_before.timestamp(), 0)
            .unwrap_or(ca.created_at);

        let expires_at = DateTime::from_timestamp(cert.validity().not_after.timestamp(), 0)
            .ok_or_else(|| CaError::Generation("Invalid expiry date".into()))?;

        // Calculate SHA256 fingerprint
        let mut hasher = Sha256::new();
        hasher.update(&cert_der);
        let fingerprint = format!("sha256:{}", hex::encode(hasher.finalize()));

        Ok(CaInfo {
            cert_pem: ca.cert_pem,
            fingerprint,
            issued_at,
            expires_at,
        })
    }
}

/// Generate a new CA certificate and store it encrypted in the database.
pub async fn generate_ca(
    storage: &dyn Storage,
    encryption_key: &[u8; 32],
    force: bool,
) -> Result<CaInfo, CaError> {
    // Check if CA exists
    if let Some(_) = CaStore::list(storage, Caller::System)
        .await
        .map_err(|e| CaError::Storage(e.to_string()))?
        .into_iter()
        .next()
    {
        if !force {
            return Err(CaError::Generation(
                "CA already exists. Use --force to overwrite.".into(),
            ));
        }
    }

    // Generate Ed25519 keypair
    let key_pair = KeyPair::generate_for(&rcgen::PKCS_ED25519)
        .map_err(|e| CaError::Generation(format!("Failed to generate keypair: {}", e)))?;

    // Create CA certificate parameters
    let mut params = CertificateParams::default();

    // Set distinguished name
    let mut dn = DistinguishedName::new();
    dn.push(rcgen::DnType::CommonName, "Lucid CA");
    dn.push(rcgen::DnType::OrganizationName, "Lucid");
    params.distinguished_name = dn;

    // Set as CA
    params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

    // Set validity (10 years)
    let now = Utc::now();
    let expires = now + Duration::days(365 * CA_CERT_VALIDITY_YEARS);
    params.not_before = ::time::OffsetDateTime::from_unix_timestamp(now.timestamp())
        .map_err(|e| CaError::Generation(format!("Invalid timestamp: {}", e)))?;
    params.not_after = ::time::OffsetDateTime::from_unix_timestamp(expires.timestamp())
        .map_err(|e| CaError::Generation(format!("Invalid timestamp: {}", e)))?;

    // Set key usage
    params.key_usages = vec![
        rcgen::KeyUsagePurpose::DigitalSignature,
        rcgen::KeyUsagePurpose::KeyCertSign,
        rcgen::KeyUsagePurpose::CrlSign,
    ];

    // Self-sign
    let cert = params
        .self_signed(&key_pair)
        .map_err(|e| CaError::Generation(format!("Failed to self-sign CA: {}", e)))?;

    let cert_pem = cert.pem();
    let private_key_pem = key_pair.serialize_pem();

    // Pre-generate the ObjectId so we can use it as AAD during encryption.
    // This prevents ciphertext transplantation: the encrypted key is bound to
    // this specific CA record and cannot be decrypted if moved to another.
    let ca_id = ObjectId::new();
    let aad = ca_id.to_hex();

    // Encrypt private key
    let encrypted_private_key =
        aes::encrypt(encryption_key, private_key_pem.as_bytes(), aad.as_bytes())
            .map_err(|e| CaError::Encryption(e.to_string()))?;

    // Create DbCa
    let db_ca = DbCa {
        id: Some(ca_id),
        cert_pem: cert_pem.clone(),
        encrypted_private_key,
        created_at: now,
    };

    // Store in DB
    let stored_ca = CaStore::create(storage, Caller::System, db_ca)
        .await
        .map_err(|e| CaError::Storage(e.to_string()))?;

    // Parse certificate for info
    let cert_der = pem_rfc7468::decode_vec(cert_pem.as_bytes())
        .map_err(|e| CaError::Generation(format!("Failed to decode CA cert PEM: {}", e)))?
        .1;

    let (_, cert_parsed) = X509Certificate::from_der(&cert_der)
        .map_err(|e| CaError::Generation(format!("Failed to parse CA cert: {}", e)))?;

    let issued_at =
        DateTime::from_timestamp(cert_parsed.validity().not_before.timestamp(), 0).unwrap_or(now);

    let expires_at = DateTime::from_timestamp(cert_parsed.validity().not_after.timestamp(), 0)
        .ok_or_else(|| CaError::Generation("Invalid expiry date".into()))?;

    // Calculate SHA256 fingerprint
    let mut hasher = Sha256::new();
    hasher.update(&cert_der);
    let fingerprint = format!("sha256:{}", hex::encode(hasher.finalize()));

    Ok(CaInfo {
        cert_pem: stored_ca.cert_pem,
        fingerprint,
        issued_at,
        expires_at,
    })
}
