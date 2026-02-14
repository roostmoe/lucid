// OIDC authentication support
//
// TODO: Implement full OIDC client once the rest of the auth system is working.
// For now this is just stubs to get things compiling.

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("OIDC not yet implemented")]
    NotImplemented,
}

#[derive(Debug, Clone)]
pub struct OidcConfig {
    pub discovery_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub allowed_domains: Vec<String>,
    pub allowed_emails: Vec<String>,
    pub owner_email: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OidcUserInfo {
    pub external_id: String, // 'sub' claim
    pub email: String,
    pub display_name: Option<String>,
}

pub struct OidcClient {
    _config: OidcConfig,
}

impl OidcClient {
    pub async fn new(config: OidcConfig) -> Result<Self, Error> {
        Ok(Self { _config: config })
    }

    pub fn authorization_url(&self) -> (String, String, String) {
        ("".to_string(), "".to_string(), "".to_string())
    }

    pub async fn exchange_code(&self, _code: &str, _nonce: &str) -> Result<OidcUserInfo, Error> {
        Err(Error::NotImplemented)
    }

    pub fn is_email_allowed(&self, email: &str) -> bool {
        // Empty lists = allow all
        if self._config.allowed_domains.is_empty() && self._config.allowed_emails.is_empty() {
            return true;
        }

        // Check explicit email list
        if self._config.allowed_emails.contains(&email.to_string()) {
            return true;
        }

        // Check domain list
        if let Some(domain) = email.split('@').nth(1)
            && self._config.allowed_domains.contains(&domain.to_string())
        {
            return true;
        }

        false
    }

    pub fn is_owner_email(&self, email: &str) -> bool {
        self._config
            .owner_email
            .as_ref()
            .map(|owner| owner == email)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_config() -> OidcConfig {
        OidcConfig {
            discovery_url: "https://example.com/.well-known/openid-configuration".to_string(),
            client_id: "test-client".to_string(),
            client_secret: "test-secret".to_string(),
            redirect_uri: "http://localhost:8080/callback".to_string(),
            allowed_domains: vec![],
            allowed_emails: vec![],
            owner_email: None,
        }
    }

    #[tokio::test]
    async fn test_new_client() {
        let config = test_config();
        let client = OidcClient::new(config).await;
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_exchange_code_not_implemented() {
        let client = OidcClient::new(test_config()).await.unwrap();
        let result = client.exchange_code("code", "nonce").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::NotImplemented));
    }

    #[test]
    fn test_is_email_allowed_empty_config_allows_all() {
        let config = test_config();
        let client = OidcClient { _config: config };

        assert!(client.is_email_allowed("anyone@example.com"));
        assert!(client.is_email_allowed("user@different.com"));
    }

    #[test]
    fn test_is_email_allowed_with_allowed_domains() {
        let mut config = test_config();
        config.allowed_domains = vec!["example.com".to_string(), "trusted.org".to_string()];
        let client = OidcClient { _config: config };

        assert!(client.is_email_allowed("user@example.com"));
        assert!(client.is_email_allowed("admin@trusted.org"));
        assert!(!client.is_email_allowed("attacker@evil.com"));
    }

    #[test]
    fn test_is_email_allowed_with_allowed_emails() {
        let mut config = test_config();
        config.allowed_emails = vec![
            "admin@example.com".to_string(),
            "root@system.local".to_string(),
        ];
        let client = OidcClient { _config: config };

        assert!(client.is_email_allowed("admin@example.com"));
        assert!(client.is_email_allowed("root@system.local"));
        assert!(!client.is_email_allowed("user@example.com")); // same domain but different user
    }

    #[test]
    fn test_is_email_allowed_with_both_lists() {
        let mut config = test_config();
        config.allowed_domains = vec!["example.com".to_string()];
        config.allowed_emails = vec!["special@other.com".to_string()];
        let client = OidcClient { _config: config };

        assert!(client.is_email_allowed("user@example.com")); // matches domain
        assert!(client.is_email_allowed("special@other.com")); // matches email
        assert!(!client.is_email_allowed("other@other.com")); // doesn't match either
    }

    #[test]
    fn test_is_owner_email_none() {
        let config = test_config();
        let client = OidcClient { _config: config };

        assert!(!client.is_owner_email("anyone@example.com"));
    }

    #[test]
    fn test_is_owner_email_match() {
        let mut config = test_config();
        config.owner_email = Some("owner@example.com".to_string());
        let client = OidcClient { _config: config };

        assert!(client.is_owner_email("owner@example.com"));
        assert!(!client.is_owner_email("other@example.com"));
    }
}
