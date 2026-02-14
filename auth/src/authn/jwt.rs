use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use lucid_uuid_kinds::UserIdUuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub issuer: String,
    pub audience: String,
    pub expiry_hours: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id as string
    pub email: String,
    pub exp: i64,
    pub iat: i64,
    pub iss: String,
    pub aud: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to create token: {source:#}")]
    TokenCreation {
        #[source]
        source: anyhow::Error,
    },

    #[error("failed to validate token: {source:#}")]
    TokenValidation {
        #[source]
        source: anyhow::Error,
    },
}

pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    config: JwtConfig,
}

impl JwtManager {
    pub fn new(config: JwtConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.secret.as_bytes());
        Self {
            encoding_key,
            decoding_key,
            config,
        }
    }

    pub fn create_token(&self, user_id: UserIdUuid, email: &str) -> Result<String, Error> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.config.expiry_hours);

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| Error::TokenCreation { source: e.into() })
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, Error> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);

        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| Error::TokenValidation { source: e.into() })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use lucid_uuid_kinds::GenericUuid;
    use uuid::Uuid;

    fn test_config() -> JwtConfig {
        JwtConfig {
            secret: "test-secret-key-min-32-chars-long!".to_string(),
            issuer: "lucid-test".to_string(),
            audience: "lucid-api".to_string(),
            expiry_hours: 24,
        }
    }

    fn test_user_id() -> UserIdUuid {
        UserIdUuid::from_untyped_uuid(
            Uuid::parse_str("01234567-89ab-cdef-0123-456789abcdef").unwrap(),
        )
    }

    #[test]
    fn test_create_token() {
        let manager = JwtManager::new(test_config());
        let user_id = test_user_id();
        let email = "test@example.com";

        let token = manager.create_token(user_id, email).unwrap();
        assert!(!token.is_empty());
        assert_eq!(token.matches('.').count(), 2); // JWT has 3 parts separated by dots
    }

    #[test]
    fn test_validate_token_success() {
        let manager = JwtManager::new(test_config());
        let user_id = test_user_id();
        let email = "test@example.com";

        let token = manager.create_token(user_id, email).unwrap();
        let claims = manager.validate_token(&token).unwrap();

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.email, email);
        assert_eq!(claims.iss, "lucid-test");
        assert_eq!(claims.aud, "lucid-api");
    }

    #[test]
    fn test_validate_token_wrong_issuer() {
        let manager = JwtManager::new(test_config());
        let user_id = test_user_id();
        let email = "test@example.com";

        let token = manager.create_token(user_id, email).unwrap();

        // Create a new manager with different issuer
        let mut wrong_config = test_config();
        wrong_config.issuer = "different-issuer".to_string();
        let wrong_manager = JwtManager::new(wrong_config);

        let result = wrong_manager.validate_token(&token);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::TokenValidation { .. }));
    }

    #[test]
    fn test_validate_token_wrong_audience() {
        let manager = JwtManager::new(test_config());
        let user_id = test_user_id();
        let email = "test@example.com";

        let token = manager.create_token(user_id, email).unwrap();

        // Create a new manager with different audience
        let mut wrong_config = test_config();
        wrong_config.audience = "different-audience".to_string();
        let wrong_manager = JwtManager::new(wrong_config);

        let result = wrong_manager.validate_token(&token);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::TokenValidation { .. }));
    }

    #[test]
    fn test_validate_token_expired() {
        let mut config = test_config();
        config.expiry_hours = -1; // Expired 1 hour ago
        let manager = JwtManager::new(config);
        let user_id = test_user_id();
        let email = "test@example.com";

        let token = manager.create_token(user_id, email).unwrap();

        // Re-create manager with positive expiry for validation
        let valid_manager = JwtManager::new(test_config());
        let result = valid_manager.validate_token(&token);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::TokenValidation { .. }));
    }

    #[test]
    fn test_validate_token_wrong_secret() {
        let manager = JwtManager::new(test_config());
        let user_id = test_user_id();
        let email = "test@example.com";

        let token = manager.create_token(user_id, email).unwrap();

        // Create manager with different secret
        let mut wrong_config = test_config();
        wrong_config.secret = "different-secret-key-also-32chars".to_string();
        let wrong_manager = JwtManager::new(wrong_config);

        let result = wrong_manager.validate_token(&token);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::TokenValidation { .. }));
    }

    #[test]
    fn test_validate_token_malformed() {
        let manager = JwtManager::new(test_config());
        let result = manager.validate_token("not.a.valid.jwt");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::TokenValidation { .. }));
    }

    #[test]
    fn test_round_trip() {
        let manager = JwtManager::new(test_config());
        let user_id = test_user_id();
        let email = "roundtrip@example.com";

        // Create token
        let token = manager.create_token(user_id, email).unwrap();

        // Validate and extract claims
        let claims = manager.validate_token(&token).unwrap();

        // Verify all fields round-tripped correctly
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.email, email);
        assert_eq!(claims.iss, manager.config.issuer);
        assert_eq!(claims.aud, manager.config.audience);
        assert!(claims.iat > 0);
        assert!(claims.exp > claims.iat);
    }
}
