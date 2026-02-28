//! JWT generation for activation keys.

use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::auth::signing::Signer;

use super::signing::SigningError;

/// Claims for activation key JWTs.
#[derive(Debug, Serialize, Deserialize)]
pub struct ActivationKeyClaims {
    /// Issuer - the public URL of this Lucid instance
    pub iss: String,
    /// Subject - the user-provided key_id
    pub sub: String,
    /// Activation key internal ID for DB lookup
    pub ak: Ulid,
    /// Issued at timestamp
    pub iat: i64,
}

/// Generate a JWT for an activation key.
pub fn generate_activation_key_jwt(
    signer: impl Signer,
    pem_key: &str,
    public_url: &str,
    key_id: &str,
    internal_id: Ulid,
) -> Result<String, SigningError> {
    let claims = ActivationKeyClaims {
        iss: public_url.to_string(),
        sub: key_id.to_string(),
        ak: internal_id,
        iat: chrono::Utc::now().timestamp(),
    };

    let mut header = Header::new(Algorithm::EdDSA);
    header.kid = Some(signer.key_id());
    header.jku = Some(format!("{}/.well-known/jwks.json", public_url));
    let encoding_key = EncodingKey::from_ed_pem(pem_key.as_bytes())
        .map_err(|e| SigningError::InvalidPem(e.to_string()))?;

    encode(&header, &claims, &encoding_key).map_err(|e| SigningError::SigningFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use crate::auth::signing::Ed25519Signer;

    use super::*;

    // Test keypair from signing.rs tests
    const TEST_PRIVATE_KEY_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEIJ+DYvh6SEqVTm50DFtMDoQikTmiCqirVv9mWG9qfSnF
-----END PRIVATE KEY-----"#;

    #[test]
    fn test_jwt_has_three_parts() {
        let signer = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();

        let jwt = generate_activation_key_jwt(
            signer,
            TEST_PRIVATE_KEY_PEM,
            "https://lucid.example.com",
            "test-key-id",
            Ulid::new(),
        )
        .unwrap();

        // JWT should be in format: header.payload.signature
        let parts: Vec<&str> = jwt.split('.').collect();
        assert_eq!(parts.len(), 3, "JWT should have exactly 3 parts");

        // Each part should be non-empty
        assert!(!parts[0].is_empty(), "header should not be empty");
        assert!(!parts[1].is_empty(), "payload should not be empty");
        assert!(!parts[2].is_empty(), "signature should not be empty");
    }

    #[test]
    fn test_jwt_claims_are_correctly_encoded() {
        use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
        use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};

        let signer = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();
        let public_url = "https://lucid.example.com";
        let key_id = "test-key-id";
        let internal_id = Ulid::new();

        let jwt = generate_activation_key_jwt(
            signer,
            TEST_PRIVATE_KEY_PEM,
            public_url,
            key_id,
            internal_id,
        )
        .unwrap();

        // Extract and decode the payload manually to verify structure
        let parts: Vec<&str> = jwt.split('.').collect();
        let payload_b64 = parts[1];

        // Decode the payload
        let payload_json = URL_SAFE_NO_PAD.decode(payload_b64).unwrap();
        let payload_str = String::from_utf8(payload_json).unwrap();

        // Verify the claims are present in JSON
        assert!(
            payload_str.contains(r#""iss":"https://lucid.example.com""#),
            "iss claim should match"
        );
        assert!(
            payload_str.contains(r#""sub":"test-key-id""#),
            "sub claim should match"
        );
        assert!(
            payload_str.contains(r#""ak":"internal-abc123""#),
            "ak claim should match"
        );
        assert!(
            payload_str.contains(r#""iat":"#),
            "iat claim should be present"
        );

        // Now decode properly with jsonwebtoken to verify full structure
        // Extract public key from the PEM
        use crate::auth::signing::Ed25519Signer;
        let signer = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();
        let public_key_bytes = signer.public_key_bytes();

        let decoding_key = DecodingKey::from_ed_der(public_key_bytes);
        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.validate_exp = false; // No expiration in our tokens
        validation.required_spec_claims.clear(); // Don't require exp claim
        validation.set_issuer(&[public_url]);

        let decoded = decode::<ActivationKeyClaims>(&jwt, &decoding_key, &validation).unwrap();

        assert_eq!(decoded.claims.iss, public_url);
        assert_eq!(decoded.claims.sub, key_id);
        assert_eq!(decoded.claims.ak, internal_id);
        assert!(decoded.claims.iat > 0, "iat should be a valid timestamp");
    }

    #[test]
    fn test_jwt_invalid_pem_returns_error() {
        let signer = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();

        let result = generate_activation_key_jwt(
            signer,
            "not a valid pem",
            "https://lucid.example.com",
            "test-key",
            Ulid::new(),
        );

        assert!(result.is_err(), "should reject invalid PEM");
        match result {
            Err(SigningError::InvalidPem(_)) => {} // expected
            _ => panic!("expected InvalidPem error"),
        }
    }

    #[test]
    fn test_jwt_different_keys_produce_different_signatures() {
        // Second test key
        const TEST_PRIVATE_KEY_PEM_2: &str = r#"-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEIBcUIT7KhLMKX9R1oJf+dFUDux98dVbI5mB3HuhMglFF
-----END PRIVATE KEY-----"#;

        let signer = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();
        let signer_2 = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM_2).unwrap();

        let id = Ulid::new();

        let jwt1 = generate_activation_key_jwt(
            signer,
            TEST_PRIVATE_KEY_PEM,
            "https://lucid.example.com",
            "same-key-id",
            id.clone(),
        )
        .unwrap();

        let jwt2 = generate_activation_key_jwt(
            signer_2,
            TEST_PRIVATE_KEY_PEM_2,
            "https://lucid.example.com",
            "same-key-id",
            id.clone(),
        )
        .unwrap();

        // Headers and payloads might be the same, but signatures MUST differ
        assert_ne!(jwt1, jwt2, "different keys should produce different JWTs");

        let parts1: Vec<&str> = jwt1.split('.').collect();
        let parts2: Vec<&str> = jwt2.split('.').collect();

        // Signatures (third part) must be different
        assert_ne!(parts1[2], parts2[2], "signatures must differ");
    }

    #[test]
    fn test_jwt_same_inputs_produce_different_tokens_due_to_timestamp() {
        use std::{thread, time::Duration};

        let signer = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();

        let id = Ulid::new();
        let jwt1 = generate_activation_key_jwt(
            signer.clone(),
            TEST_PRIVATE_KEY_PEM,
            "https://lucid.example.com",
            "test-key",
            id.clone(),
        )
        .unwrap();

        // Sleep to ensure different timestamp
        thread::sleep(Duration::from_millis(1001));

        let jwt2 = generate_activation_key_jwt(
            signer,
            TEST_PRIVATE_KEY_PEM,
            "https://lucid.example.com",
            "test-key",
            id,
        )
        .unwrap();

        // Should be different due to different iat timestamps
        assert_ne!(
            jwt1, jwt2,
            "JWTs generated at different times should differ"
        );
    }

    #[test]
    fn test_jwt_header_algorithm_is_eddsa() {
        use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

        let signer = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();

        let jwt = generate_activation_key_jwt(
            signer,
            TEST_PRIVATE_KEY_PEM,
            "https://lucid.example.com",
            "test-key",
            Ulid::new(),
        )
        .unwrap();

        let parts: Vec<&str> = jwt.split('.').collect();
        let header_b64 = parts[0];
        let header_json = URL_SAFE_NO_PAD.decode(header_b64).unwrap();
        let header_str = String::from_utf8(header_json).unwrap();

        assert!(
            header_str.contains(r#""alg":"EdDSA""#),
            "algorithm should be EdDSA"
        );
        assert!(header_str.contains(r#""typ":"JWT""#), "type should be JWT");
    }
}
