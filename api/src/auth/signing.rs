use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub struct SessionSigner {
    secret: [u8; 32],
}

impl SessionSigner {
    pub fn new(secret: [u8; 32]) -> Self {
        Self { secret }
    }

    /// Sign a session ID: returns "session_id.signature"
    pub fn sign(&self, session_id: &str) -> String {
        let mut mac =
            HmacSha256::new_from_slice(&self.secret).expect("HMAC can take key of any size");
        mac.update(session_id.as_bytes());
        let signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
        format!("{}.{}", session_id, signature)
    }

    /// Verify a signed session ID, returns the session_id if valid
    pub fn verify(&self, signed: &str) -> Option<String> {
        let (session_id, signature) = signed.rsplit_once('.')?;

        let expected_sig = {
            let mut mac =
                HmacSha256::new_from_slice(&self.secret).expect("HMAC can take key of any size");
            mac.update(session_id.as_bytes());
            URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
        };

        // Constant-time comparison
        if signature.len() != expected_sig.len() {
            return None;
        }

        let matches = signature
            .as_bytes()
            .iter()
            .zip(expected_sig.as_bytes())
            .fold(0u8, |acc, (a, b)| acc | (a ^ b));

        if matches == 0 {
            Some(session_id.to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify_roundtrip() {
        let signer = SessionSigner::new([0u8; 32]);
        let session_id = "test_session_abc123";

        let signed = signer.sign(session_id);

        // Should contain a dot
        assert!(signed.contains('.'));

        // Should verify correctly
        let verified = signer.verify(&signed);
        assert_eq!(verified, Some(session_id.to_string()));
    }

    #[test]
    fn test_verify_rejects_tampered_signature() {
        let signer = SessionSigner::new([0u8; 32]);
        let session_id = "test_session";
        let signed = signer.sign(session_id);

        // Tamper with the signature
        let tampered = format!("{}x", signed);
        assert_eq!(signer.verify(&tampered), None);
    }

    #[test]
    fn test_verify_rejects_tampered_session_id() {
        let signer = SessionSigner::new([0u8; 32]);
        let signed = signer.sign("original_session");

        // Replace session ID but keep signature
        let parts: Vec<&str> = signed.split('.').collect();
        let tampered = format!("different_session.{}", parts[1]);

        assert_eq!(signer.verify(&tampered), None);
    }

    #[test]
    fn test_verify_rejects_missing_signature() {
        let signer = SessionSigner::new([0u8; 32]);
        assert_eq!(signer.verify("no_signature_here"), None);
    }

    #[test]
    fn test_different_secrets_produce_different_signatures() {
        let signer1 = SessionSigner::new([0u8; 32]);
        let signer2 = SessionSigner::new([1u8; 32]);

        let session_id = "same_session";
        let signed1 = signer1.sign(session_id);
        let signed2 = signer2.sign(session_id);

        assert_ne!(signed1, signed2);

        // Cross-verification should fail
        assert_eq!(signer1.verify(&signed2), None);
        assert_eq!(signer2.verify(&signed1), None);
    }

    #[test]
    fn test_verify_rejects_empty_string() {
        let signer = SessionSigner::new([0u8; 32]);
        assert_eq!(signer.verify(""), None);
    }

    #[test]
    fn test_verify_rejects_only_dot() {
        let signer = SessionSigner::new([0u8; 32]);
        assert_eq!(signer.verify("."), None);
    }

    #[test]
    fn test_sign_handles_empty_session_id() {
        let signer = SessionSigner::new([0u8; 32]);
        let signed = signer.sign("");

        // Should still produce a valid format
        assert!(signed.contains('.'));

        // And should verify
        assert_eq!(signer.verify(&signed), Some("".to_string()));
    }

    #[test]
    fn test_sign_handles_special_characters() {
        let signer = SessionSigner::new([0u8; 32]);
        let session_id = "session_with-special.chars_123!@#";

        let signed = signer.sign(session_id);
        let verified = signer.verify(&signed);

        assert_eq!(verified, Some(session_id.to_string()));
    }

    #[test]
    fn test_verify_rejects_multiple_dots() {
        let signer = SessionSigner::new([0u8; 32]);
        let valid = signer.sign("test");

        // Add extra dots
        let invalid = format!("{}.extra", valid);

        // Should fail because rsplit_once takes the LAST dot
        // So it will split incorrectly
        assert_eq!(signer.verify(&invalid), None);
    }
}
