//! Cryptographic signing and verification for session tokens and future JWT support.
//!
//! This module provides a generic [`Signer`] trait and concrete implementations for
//! creating and verifying digital signatures. The primary use case is session token
//! authentication, but the design supports future JWT signing needs.
//!
//! # Architecture
//!
//! - [`Signer`]: Generic trait for any signing implementation
//! - [`Ed25519Signer`]: Ed25519 implementation using PEM-formatted PKCS#8 keys
//! - [`SessionSigner`]: Wrapper that applies session-specific token formatting
//!
//! # Example
//!
//! ```no_run
//! use lucid_api::auth::signing::{Ed25519Signer, SessionSigner, Signer};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Load key from PEM
//! let pem = std::fs::read_to_string("signing_key.pem")?;
//! let ed25519 = Ed25519Signer::from_pem(&pem)?;
//!
//! // Wrap for session token format
//! let session_signer = SessionSigner::new(ed25519);
//!
//! // Sign a session ID
//! let token = session_signer.sign("user_session_123")?;
//! // Returns: "user_session_123.{base64_signature}"
//!
//! // Verify and extract session ID
//! if let Some(session_id) = session_signer.verify(&token) {
//!     println!("Valid session: {}", session_id);
//! }
//! # Ok(())
//! # }
//! ```

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{
    SECRET_KEY_LENGTH, Signature, Signer as DalekSigner, SigningKey, Verifier, VerifyingKey,
};
use thiserror::Error;

/// Generic signing trait for any payload.
///
/// This trait abstracts over different signing algorithms, allowing the same
/// session/JWT logic to work with different cryptographic backends.
///
/// Implementations must be thread-safe (`Send + Sync`) for use in async contexts.
pub trait Signer: Send + Sync {
    /// Sign a payload, returning the raw signature bytes.
    ///
    /// # Errors
    ///
    /// Returns [`SigningError`] if signing fails (e.g., invalid key state).
    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, SigningError>;

    /// Verify a payload against a signature.
    ///
    /// Returns `true` if the signature is valid for the given payload.
    /// Returns `false` for any verification failure (invalid signature, wrong key, etc.).
    fn verify(&self, payload: &[u8], signature: &[u8]) -> bool;
}

/// Errors that can occur during signing operations.
#[derive(Debug, Error)]
pub enum SigningError {
    /// PEM parsing or key format errors.
    #[error("invalid PEM format: {0}")]
    InvalidPem(String),

    /// Signing operation failures.
    #[error("signing failed: {0}")]
    SigningFailed(String),
}

/// Ed25519 digital signature implementation of [`Signer`].
///
/// Uses the Ed25519 signature scheme with SHA-512 hashing. Keys must be provided
/// in PEM-encoded PKCS#8 format.
///
/// # Key Format
///
/// Expected PEM structure:
/// ```text
/// -----BEGIN PRIVATE KEY-----
/// MC4CAQAwBQYDK2VwBCIEI...
/// -----END PRIVATE KEY-----
/// ```
///
/// Generate a compatible key using OpenSSL:
/// ```bash
/// openssl genpkey -algorithm ED25519 -out signing_key.pem
/// ```
///
/// # Example
///
/// ```no_run
/// use lucid_api::auth::signing::Ed25519Signer;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pem = std::fs::read_to_string("signing_key.pem")?;
/// let signer = Ed25519Signer::from_pem(&pem)?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Ed25519Signer {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl Ed25519Signer {
    /// Create a new Ed25519 signer from PEM-formatted PKCS#8 private key data.
    ///
    /// # Errors
    ///
    /// Returns [`SigningError::InvalidPem`] if:
    /// - PEM format is invalid
    /// - PKCS#8 structure is malformed
    /// - Key is not exactly 32 bytes (Ed25519 requirement)
    /// - PEM label is not "PRIVATE KEY"
    pub fn from_pem(pem_data: &str) -> Result<Self, SigningError> {
        // Parse PEM using pem-rfc7468
        use pkcs8::der::Decode;
        let (label, der_bytes) = pem_rfc7468::decode_vec(pem_data.as_bytes())
            .map_err(|e| SigningError::InvalidPem(format!("PEM decode failed: {}", e)))?;

        if label != "PRIVATE KEY" {
            return Err(SigningError::InvalidPem(format!(
                "expected PRIVATE KEY label, got {}",
                label
            )));
        }

        // Extract the raw secret key bytes from PKCS#8
        let private_key_info = pkcs8::PrivateKeyInfo::from_der(&der_bytes)
            .map_err(|e| SigningError::InvalidPem(format!("invalid PKCS#8 structure: {}", e)))?;

        // The private key is wrapped in an OCTET STRING, decode it
        let secret_octet_string: &[u8] =
            pkcs8::der::asn1::OctetStringRef::from_der(private_key_info.private_key)
                .map_err(|e| SigningError::InvalidPem(format!("invalid octet string: {}", e)))?
                .as_bytes();

        if secret_octet_string.len() != SECRET_KEY_LENGTH {
            return Err(SigningError::InvalidPem(format!(
                "expected {} byte key, got {}",
                SECRET_KEY_LENGTH,
                secret_octet_string.len()
            )));
        }

        let mut key_array = [0u8; SECRET_KEY_LENGTH];
        key_array.copy_from_slice(secret_octet_string);

        let signing_key = SigningKey::from_bytes(&key_array);
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            signing_key,
            verifying_key,
        })
    }

    /// Get the public verifying key.
    ///
    /// Useful for debugging, logging, or distributing the public key for
    /// external verification.
    pub fn public_key(&self) -> &VerifyingKey {
        &self.verifying_key
    }
}

impl Signer for Ed25519Signer {
    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, SigningError> {
        let signature = self.signing_key.sign(payload);
        Ok(signature.to_bytes().to_vec())
    }

    fn verify(&self, payload: &[u8], signature: &[u8]) -> bool {
        let Ok(sig) = Signature::from_slice(signature) else {
            return false;
        };
        self.verifying_key.verify(payload, &sig).is_ok()
    }
}

/// Session token signing wrapper.
///
/// Wraps any [`Signer`] implementation to produce and verify session tokens in the
/// format: `{session_id}.{base64_signature}`.
///
/// The signature covers only the session ID portion. The base64 encoding uses
/// URL-safe characters without padding for compatibility with HTTP headers.
///
/// # Token Format
///
/// ```text
/// session_abc123.dGVzdHNpZ25hdHVyZQ
///         ^               ^
///    session ID      base64(signature)
/// ```
///
/// The session ID and signature are separated by a single dot (`.`). The signature
/// is base64-encoded using URL-safe characters (no padding).
///
/// # Example
///
/// ```no_run
/// use lucid_api::auth::signing::{Ed25519Signer, SessionSigner};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pem = std::fs::read_to_string("signing_key.pem")?;
/// let ed25519 = Ed25519Signer::from_pem(&pem)?;
/// let session_signer = SessionSigner::new(ed25519);
///
/// // Create signed token
/// let token = session_signer.sign("user_session_123")?;
///
/// // Verify token and extract session ID
/// match session_signer.verify(&token) {
///     Some(session_id) => println!("Valid: {}", session_id),
///     None => println!("Invalid or tampered token"),
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct SessionSigner<S: Signer> {
    signer: S,
}

impl<S: Signer> SessionSigner<S> {
    /// Create a new session signer with the given underlying signer.
    pub fn new(signer: S) -> Self {
        Self { signer }
    }

    /// Sign a session ID, returning a token in the format `{session_id}.{signature}`.
    ///
    /// # Errors
    ///
    /// Returns [`SigningError`] if the underlying signer fails.
    pub fn sign(&self, session_id: &str) -> Result<String, SigningError> {
        let signature = self.signer.sign(session_id.as_bytes())?;
        let encoded = URL_SAFE_NO_PAD.encode(&signature);
        Ok(format!("{}.{}", session_id, encoded))
    }

    /// Verify a signed session token and extract the session ID.
    ///
    /// Returns `Some(session_id)` if the signature is valid, `None` otherwise.
    ///
    /// # Validation
    ///
    /// Returns `None` if:
    /// - Token format is invalid (no dot separator)
    /// - Signature portion is not valid base64
    /// - Signature verification fails
    /// - Session ID has been tampered with
    pub fn verify(&self, signed: &str) -> Option<String> {
        let (session_id, signature_b64) = signed.rsplit_once('.')?;

        let signature = URL_SAFE_NO_PAD.decode(signature_b64).ok()?;

        if self.signer.verify(session_id.as_bytes(), &signature) {
            Some(session_id.to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test keypair generated for reproducible tests
    const TEST_PRIVATE_KEY_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEIJ+DYvh6SEqVTm50DFtMDoQikTmiCqirVv9mWG9qfSnF
-----END PRIVATE KEY-----"#;

    // Second test key for different-key tests
    const TEST_PRIVATE_KEY_PEM_2: &str = r#"-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEIBcUIT7KhLMKX9R1oJf+dFUDux98dVbI5mB3HuhMglFF
-----END PRIVATE KEY-----"#;

    #[test]
    fn test_pem_key_loading() {
        let signer = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM);
        assert!(signer.is_ok(), "should load valid PEM key");
    }

    #[test]
    fn test_invalid_pem_handling() {
        let invalid_pem = "not a valid pem";
        let result = Ed25519Signer::from_pem(invalid_pem);
        assert!(result.is_err(), "should reject invalid PEM");

        let empty_pem = "";
        let result = Ed25519Signer::from_pem(empty_pem);
        assert!(result.is_err(), "should reject empty PEM");
    }

    #[test]
    fn test_sign_verify_roundtrip() {
        let signer = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();

        let payload = b"test payload";
        let signature = signer.sign(payload).unwrap();

        assert!(
            signer.verify(payload, &signature),
            "should verify own signature"
        );
    }

    #[test]
    fn test_signature_rejection_wrong_key() {
        let signer1 = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();
        let signer2 = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM_2).unwrap();

        let payload = b"test payload";
        let signature1 = signer1.sign(payload).unwrap();

        assert!(
            !signer2.verify(payload, &signature1),
            "should reject signature from different key"
        );
    }

    #[test]
    fn test_signature_rejection_tampered_payload() {
        let signer = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();

        let payload = b"original payload";
        let signature = signer.sign(payload).unwrap();

        let tampered = b"tampered payload";
        assert!(
            !signer.verify(tampered, &signature),
            "should reject tampered payload"
        );
    }

    #[test]
    fn test_signature_rejection_tampered_signature() {
        let signer = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();

        let payload = b"test payload";
        let mut signature = signer.sign(payload).unwrap();

        // Flip a bit
        signature[0] ^= 0x01;

        assert!(
            !signer.verify(payload, &signature),
            "should reject tampered signature"
        );
    }

    #[test]
    fn test_session_signer_roundtrip() {
        let ed25519 = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();
        let session_signer = SessionSigner::new(ed25519);

        let session_id = "test_session_abc123";
        let signed = session_signer.sign(session_id).unwrap();

        // Should contain a dot
        assert!(signed.contains('.'), "should have dot separator");

        // Should verify correctly
        let verified = session_signer.verify(&signed);
        assert_eq!(verified, Some(session_id.to_string()));
    }

    #[test]
    fn test_session_signer_rejects_tampered_signature() {
        let ed25519 = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();
        let session_signer = SessionSigner::new(ed25519);

        let session_id = "test_session";
        let signed = session_signer.sign(session_id).unwrap();

        // Tamper with the signature
        let tampered = format!("{}x", signed);
        assert_eq!(session_signer.verify(&tampered), None);
    }

    #[test]
    fn test_session_signer_rejects_tampered_session_id() {
        let ed25519 = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();
        let session_signer = SessionSigner::new(ed25519);

        let signed = session_signer.sign("original_session").unwrap();

        // Replace session ID but keep signature
        let parts: Vec<&str> = signed.split('.').collect();
        let tampered = format!("different_session.{}", parts[1]);

        assert_eq!(session_signer.verify(&tampered), None);
    }

    #[test]
    fn test_session_signer_rejects_missing_signature() {
        let ed25519 = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();
        let session_signer = SessionSigner::new(ed25519);

        assert_eq!(session_signer.verify("no_signature_here"), None);
    }

    #[test]
    fn test_session_signer_different_keys_different_signatures() {
        let ed25519_1 = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();
        let session_signer1 = SessionSigner::new(ed25519_1);

        let ed25519_2 = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM_2).unwrap();
        let session_signer2 = SessionSigner::new(ed25519_2);

        let session_id = "same_session";
        let signed1 = session_signer1.sign(session_id).unwrap();
        let signed2 = session_signer2.sign(session_id).unwrap();

        assert_ne!(
            signed1, signed2,
            "different keys should produce different signatures"
        );

        // Cross-verification should fail
        assert_eq!(session_signer1.verify(&signed2), None);
        assert_eq!(session_signer2.verify(&signed1), None);
    }

    #[test]
    fn test_session_signer_empty_session_id() {
        let ed25519 = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();
        let session_signer = SessionSigner::new(ed25519);

        let signed = session_signer.sign("").unwrap();

        // Should still produce valid format
        assert!(signed.contains('.'));

        // And should verify
        assert_eq!(session_signer.verify(&signed), Some("".to_string()));
    }

    #[test]
    fn test_session_signer_special_characters() {
        let ed25519 = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();
        let session_signer = SessionSigner::new(ed25519);

        let session_id = "session_with-special.chars_123!@#";
        let signed = session_signer.sign(session_id).unwrap();
        let verified = session_signer.verify(&signed);

        assert_eq!(verified, Some(session_id.to_string()));
    }

    #[test]
    fn test_session_signer_rejects_multiple_dots() {
        let ed25519 = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();
        let session_signer = SessionSigner::new(ed25519);

        let valid = session_signer.sign("test").unwrap();

        // Add extra dots
        let invalid = format!("{}.extra", valid);

        // Should fail because rsplit_once takes the LAST dot
        assert_eq!(session_signer.verify(&invalid), None);
    }

    #[test]
    fn test_session_signer_rejects_only_dot() {
        let ed25519 = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();
        let session_signer = SessionSigner::new(ed25519);

        assert_eq!(session_signer.verify("."), None);
    }

    #[test]
    fn test_session_signer_rejects_empty_string() {
        let ed25519 = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();
        let session_signer = SessionSigner::new(ed25519);

        assert_eq!(session_signer.verify(""), None);
    }
}
