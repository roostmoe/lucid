use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit},
};
use thiserror::Error;

const NONCE_SIZE: usize = 12; // 96 bits for GCM
const TAG_SIZE: usize = 16; // 128 bits for GCM

#[derive(Debug, Error)]
pub enum AesError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Invalid ciphertext: expected at least {expected} bytes, got {actual}")]
    InvalidCiphertext { expected: usize, actual: usize },

    #[error("Invalid key: expected {expected} bytes, got {actual}")]
    InvalidKey { expected: usize, actual: usize },
}

/// Encrypt plaintext using AES-256-GCM with Additional Authenticated Data (AAD).
///
/// # Format
/// The output is: `nonce (12 bytes) || ciphertext || tag (16 bytes)`
///
/// # Arguments
/// * `key` - 32-byte encryption key
/// * `plaintext` - Data to encrypt
/// * `aad` - Additional authenticated data (e.g., record ID to prevent ciphertext transplantation)
///
/// # Returns
/// Combined nonce + ciphertext + tag as a single Vec<u8>
pub fn encrypt(key: &[u8], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, AesError> {
    if key.len() != 32 {
        return Err(AesError::InvalidKey {
            expected: 32,
            actual: key.len(),
        });
    }

    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    // Generate random nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    getrandom::getrandom(&mut nonce_bytes).map_err(|e| {
        AesError::EncryptionFailed(format!("Failed to generate random nonce: {}", e))
    })?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt with AAD
    let ciphertext = cipher
        .encrypt(
            nonce,
            aes_gcm::aead::Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|e| AesError::EncryptionFailed(e.to_string()))?;

    // Combine: nonce || ciphertext+tag
    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypt ciphertext using AES-256-GCM with Additional Authenticated Data (AAD).
///
/// # Format
/// Input is expected to be: `nonce (12 bytes) || ciphertext || tag (16 bytes)`
///
/// # Arguments
/// * `key` - 32-byte encryption key
/// * `ciphertext` - Combined nonce + encrypted data + tag
/// * `aad` - Additional authenticated data (must match what was used during encryption)
///
/// # Returns
/// Decrypted plaintext
pub fn decrypt(key: &[u8], ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>, AesError> {
    if key.len() != 32 {
        return Err(AesError::InvalidKey {
            expected: 32,
            actual: key.len(),
        });
    }

    let min_size = NONCE_SIZE + TAG_SIZE;
    if ciphertext.len() < min_size {
        return Err(AesError::InvalidCiphertext {
            expected: min_size,
            actual: ciphertext.len(),
        });
    }

    // Extract nonce and ciphertext+tag
    let nonce = Nonce::from_slice(&ciphertext[..NONCE_SIZE]);
    let encrypted_data = &ciphertext[NONCE_SIZE..];

    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    // Decrypt with AAD
    let plaintext = cipher
        .decrypt(
            nonce,
            aes_gcm::aead::Payload {
                msg: encrypted_data,
                aad,
            },
        )
        .map_err(|e| AesError::DecryptionFailed(e.to_string()))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        [0x42; 32] // Simple test key
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = test_key();
        let plaintext = b"super secret data";
        let aad = b"record-id-12345";

        let ciphertext = encrypt(&key, plaintext, aad).expect("encryption failed");
        let decrypted = decrypt(&key, &ciphertext, aad).expect("decryption failed");

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_nonce_uniqueness() {
        let key = test_key();
        let plaintext = b"same data";
        let aad = b"record-id";

        let ct1 = encrypt(&key, plaintext, aad).unwrap();
        let ct2 = encrypt(&key, plaintext, aad).unwrap();

        // Ciphertexts should differ due to different nonces
        assert_ne!(ct1, ct2);

        // But both should decrypt to same plaintext
        assert_eq!(
            decrypt(&key, &ct1, aad).unwrap(),
            decrypt(&key, &ct2, aad).unwrap()
        );
    }

    #[test]
    fn test_aad_binding() {
        let key = test_key();
        let plaintext = b"data";
        let aad1 = b"record-id-1";
        let aad2 = b"record-id-2";

        let ciphertext = encrypt(&key, plaintext, aad1).unwrap();

        // Should decrypt with correct AAD
        assert!(decrypt(&key, &ciphertext, aad1).is_ok());

        // Should fail with wrong AAD (prevents ciphertext transplantation)
        assert!(decrypt(&key, &ciphertext, aad2).is_err());
    }

    #[test]
    fn test_invalid_key_size() {
        let short_key = [0u8; 16]; // Too short
        let plaintext = b"data";
        let aad = b"aad";

        assert!(matches!(
            encrypt(&short_key, plaintext, aad),
            Err(AesError::InvalidKey { .. })
        ));
    }

    #[test]
    fn test_invalid_ciphertext_size() {
        let key = test_key();
        let short_ciphertext = [0u8; 10]; // Too short
        let aad = b"aad";

        assert!(matches!(
            decrypt(&key, &short_ciphertext, aad),
            Err(AesError::InvalidCiphertext { .. })
        ));
    }

    #[test]
    fn test_tampered_ciphertext() {
        let key = test_key();
        let plaintext = b"data";
        let aad = b"aad";

        let mut ciphertext = encrypt(&key, plaintext, aad).unwrap();

        // Tamper with a byte in the encrypted portion
        if ciphertext.len() > NONCE_SIZE {
            ciphertext[NONCE_SIZE] ^= 0xFF;
        }

        // Decryption should fail
        assert!(decrypt(&key, &ciphertext, aad).is_err());
    }
}
