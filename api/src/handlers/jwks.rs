//! JWKS (JSON Web Key Set) endpoint handler.
//!
//! Exposes the server's Ed25519 public signing key as a JWKS document, enabling
//! external consumers to verify JWTs issued by this service.
//!
//! The endpoint follows [RFC 7517](https://www.rfc-editor.org/rfc/rfc7517) and
//! [RFC 8037](https://www.rfc-editor.org/rfc/rfc8037) for OKP key representation.

use axum::{Json, extract::State};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::Serialize;

use crate::{context::ApiContext, error::ApiError};

/// A single JSON Web Key representing an OKP (Octet Key Pair) Ed25519 public key.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct Jwk {
    /// Key type — always `"OKP"` for Ed25519 keys (RFC 8037).
    kty: &'static str,

    /// Curve — always `"Ed25519"`.
    crv: &'static str,

    /// Base64url-encoded public key bytes (32 bytes for Ed25519).
    x: String,

    /// Key ID — a base64url-encoded prefix of the public key bytes,
    /// used to identify which key was used to sign a token.
    kid: String,

    /// Intended use of the key. `"sig"` indicates this key is for signing.
    #[serde(rename = "use")]
    key_use: &'static str,

    /// Algorithms this key supports — `"EdDSA"` for Ed25519.
    #[serde(rename = "alg")]
    algorithm: &'static str,
}

/// The JSON Web Key Set response, containing one or more public keys.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct JwkSet {
    keys: Vec<Jwk>,
}

/// Retrieve the server's public JSON Web Key Set.
///
/// Returns the Ed25519 public key(s) used by this server to sign tokens.
/// External services can use this endpoint to verify JWTs without needing
/// a shared secret.
///
/// The key is represented as an OKP (Octet Key Pair) JWK per RFC 8037.
///
/// # Example
///
/// ```bash
/// curl http://localhost:4000/.well-known/jwks.json
/// ```
///
/// ```json
/// {
///   "keys": [
///     {
///       "kty": "OKP",
///       "crv": "Ed25519",
///       "x": "11qYAYKxCrfVS_7TyWQHOg7hcvPapiMlrwIaaPcHURo",
///       "kid": "11qYAYKxCrfV",
///       "use": "sig",
///       "alg": "EdDSA"
///     }
///   ]
/// }
/// ```
#[utoipa::path(
    get,
    path = "/.well-known/jwks.json",
    tags = ["auth"],
    responses((status = 200, description = "JSON Web Key Set", body = JwkSet))
)]
pub async fn get_jwks(State(ctx): State<ApiContext>) -> Result<Json<JwkSet>, ApiError> {
    let pub_bytes = ctx.session_signer.inner().public_key_bytes();

    let x = URL_SAFE_NO_PAD.encode(pub_bytes);
    // Use the first 8 bytes as a short key ID — deterministic, no extra deps needed.
    let kid = URL_SAFE_NO_PAD.encode(&pub_bytes[..8]);

    let key = Jwk {
        kty: "OKP",
        crv: "Ed25519",
        x,
        kid,
        key_use: "sig",
        algorithm: "EdDSA",
    };

    Ok(Json(JwkSet { keys: vec![key] }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::signing::Ed25519Signer;

    const TEST_PRIVATE_KEY_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEIJ+DYvh6SEqVTm50DFtMDoQikTmiCqirVv9mWG9qfSnF
-----END PRIVATE KEY-----"#;

    #[test]
    fn test_public_key_bytes_roundtrip() {
        let signer = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();
        let bytes = signer.public_key_bytes();

        // Should be exactly 32 bytes for Ed25519
        assert_eq!(bytes.len(), 32);

        // Base64url encode should be 43 chars (32 bytes, no padding)
        let encoded = URL_SAFE_NO_PAD.encode(bytes);
        assert_eq!(encoded.len(), 43);
    }

    #[test]
    fn test_kid_derived_from_public_key() {
        let signer = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();
        let bytes = signer.public_key_bytes();
        let kid = URL_SAFE_NO_PAD.encode(&bytes[..8]);

        // Kid should be 11 chars (8 bytes base64url no-pad)
        assert_eq!(kid.len(), 11);
    }

    #[test]
    fn test_same_key_same_kid() {
        let signer1 = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();
        let signer2 = Ed25519Signer::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();

        let kid1 = URL_SAFE_NO_PAD.encode(&signer1.public_key_bytes()[..8]);
        let kid2 = URL_SAFE_NO_PAD.encode(&signer2.public_key_bytes()[..8]);

        assert_eq!(kid1, kid2, "same key should always produce same kid");
    }
}
