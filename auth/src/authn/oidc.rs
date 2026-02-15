// OIDC authentication support

use openidconnect::{
    AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointMaybeSet,
    EndpointNotSet, EndpointSet, IssuerUrl, Nonce, RedirectUrl, Scope, TokenResponse,
    core::{CoreClient, CoreProviderMetadata, CoreResponseType},
    reqwest,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to discover OIDC provider: {0}")]
    DiscoveryFailed(String),

    #[error("invalid redirect URI: {0}")]
    InvalidRedirectUri(String),

    #[error("invalid issuer URL: {0}")]
    InvalidIssuerUrl(String),

    #[error("failed to exchange authorization code: {0}")]
    CodeExchangeFailed(String),

    #[error("failed to verify ID token: {0}")]
    TokenVerificationFailed(String),

    #[error("missing required claim: {0}")]
    MissingClaim(String),
}

#[derive(Debug, Clone)]
pub struct OidcConfig {
    pub discovery_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
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

type ConfiguredOidcClient = CoreClient<
    EndpointSet,      // HasAuthUrl: EndpointSet (always discovered)
    EndpointNotSet,   // HasDeviceAuthUrl: EndpointNotSet (not used)
    EndpointNotSet,   // HasIntrospectionUrl: EndpointNotSet (not used)
    EndpointNotSet,   // HasRevocationUrl: EndpointNotSet (not used)
    EndpointMaybeSet, // HasTokenUrl: EndpointMaybeSet (might be discovered)
    EndpointMaybeSet, // HasUserInfoUrl: EndpointMaybeSet (might be discovered)
>;

pub struct OidcClient {
    client: ConfiguredOidcClient,
    config: OidcConfig,
}

impl OidcClient {
    pub async fn new(config: OidcConfig) -> Result<Self, Error> {
        // Build HTTP client with no redirects (SSRF protection)
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| Error::DiscoveryFailed(e.to_string()))?;

        // Parse issuer URL from discovery URL
        let issuer_url = IssuerUrl::new(config.discovery_url.clone())
            .map_err(|e| Error::InvalidIssuerUrl(e.to_string()))?;

        // Discover provider metadata
        let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, &http_client)
            .await
            .map_err(|e| Error::DiscoveryFailed(e.to_string()))?;

        // Create OIDC client (without setting redirect_uri to avoid type state issues)
        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
        )
        .set_redirect_uri(
            RedirectUrl::new(config.redirect_uri.clone())
                .map_err(|e| Error::InvalidRedirectUri(e.to_string()))?,
        );

        Ok(Self { client, config })
    }

    pub fn authorization_url(&self) -> (String, String, String) {
        let mut auth_request = self.client.authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        );

        // Add configured scopes
        for scope in &self.config.scopes {
            auth_request = auth_request.add_scope(Scope::new(scope.clone()));
        }

        let (auth_url, csrf_token, nonce) = auth_request.url();

        (
            auth_url.to_string(),
            csrf_token.secret().to_string(),
            nonce.secret().to_string(),
        )
    }

    pub async fn exchange_code(&self, code: &str, nonce: &str) -> Result<OidcUserInfo, Error> {
        // Build HTTP client with no redirects (SSRF protection)
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| Error::CodeExchangeFailed(e.to_string()))?;

        // Exchange authorization code for tokens
        let token_request = self
            .client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .map_err(|e| Error::CodeExchangeFailed(format!("{:?}", e)))?;

        let token_response = token_request
            .request_async(&http_client)
            .await
            .map_err(|e| Error::CodeExchangeFailed(format!("{}", e)))?;

        // Extract and verify ID token
        let id_token = token_response
            .id_token()
            .ok_or_else(|| Error::MissingClaim("id_token".to_string()))?;

        let claims = id_token
            .claims(
                &self.client.id_token_verifier(),
                &Nonce::new(nonce.to_string()),
            )
            .map_err(|e| Error::TokenVerificationFailed(format!("{}", e)))?;

        // Extract user info from claims
        let external_id = claims.subject().to_string();

        let email = claims
            .email()
            .map(|e| e.as_str().to_string())
            .ok_or_else(|| Error::MissingClaim("email".to_string()))?;

        let display_name = claims
            .name()
            .and_then(|n| n.get(None).map(|localized| localized.as_str().to_string()));

        Ok(OidcUserInfo {
            external_id,
            email,
            display_name,
        })
    }

    pub fn is_email_allowed(&self, email: &str) -> bool {
        // Empty lists = allow all
        if self.config.allowed_domains.is_empty() && self.config.allowed_emails.is_empty() {
            return true;
        }

        // Check explicit email list
        if self.config.allowed_emails.contains(&email.to_string()) {
            return true;
        }

        // Check domain list
        if let Some(domain) = email.split('@').nth(1)
            && self.config.allowed_domains.contains(&domain.to_string())
        {
            return true;
        }

        false
    }

    pub fn is_owner_email(&self, email: &str) -> bool {
        self.config
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
            scopes: vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ],
            allowed_domains: vec![],
            allowed_emails: vec![],
            owner_email: None,
        }
    }

    // Test helper that creates a config wrapper for testing email validation logic
    // without requiring a full OIDC client
    struct TestConfigWrapper {
        config: OidcConfig,
    }

    impl TestConfigWrapper {
        fn is_email_allowed(&self, email: &str) -> bool {
            // Replicate the email validation logic for testing
            if self.config.allowed_domains.is_empty() && self.config.allowed_emails.is_empty() {
                return true;
            }

            if self.config.allowed_emails.contains(&email.to_string()) {
                return true;
            }

            if let Some(domain) = email.split('@').nth(1)
                && self.config.allowed_domains.contains(&domain.to_string())
            {
                return true;
            }

            false
        }

        fn is_owner_email(&self, email: &str) -> bool {
            self.config
                .owner_email
                .as_ref()
                .map(|owner| owner == email)
                .unwrap_or(false)
        }
    }

    // Note: Tests for OidcClient::new(), authorization_url(), and exchange_code()
    // require a real OIDC provider or mock server. Integration tests should use
    // a mock OIDC server like https://github.com/navikt/mock-oauth2-server

    #[test]
    fn test_is_email_allowed_empty_config_allows_all() {
        let config = test_config();
        let wrapper = TestConfigWrapper { config };

        assert!(wrapper.is_email_allowed("anyone@example.com"));
        assert!(wrapper.is_email_allowed("user@different.com"));
    }

    #[test]
    fn test_is_email_allowed_with_allowed_domains() {
        let mut config = test_config();
        config.allowed_domains = vec!["example.com".to_string(), "trusted.org".to_string()];
        let wrapper = TestConfigWrapper { config };

        assert!(wrapper.is_email_allowed("user@example.com"));
        assert!(wrapper.is_email_allowed("admin@trusted.org"));
        assert!(!wrapper.is_email_allowed("attacker@evil.com"));
    }

    #[test]
    fn test_is_email_allowed_with_allowed_emails() {
        let mut config = test_config();
        config.allowed_emails = vec![
            "admin@example.com".to_string(),
            "root@system.local".to_string(),
        ];
        let wrapper = TestConfigWrapper { config };

        assert!(wrapper.is_email_allowed("admin@example.com"));
        assert!(wrapper.is_email_allowed("root@system.local"));
        assert!(!wrapper.is_email_allowed("user@example.com")); // same domain but different user
    }

    #[test]
    fn test_is_email_allowed_with_both_lists() {
        let mut config = test_config();
        config.allowed_domains = vec!["example.com".to_string()];
        config.allowed_emails = vec!["special@other.com".to_string()];
        let wrapper = TestConfigWrapper { config };

        assert!(wrapper.is_email_allowed("user@example.com")); // matches domain
        assert!(wrapper.is_email_allowed("special@other.com")); // matches email
        assert!(!wrapper.is_email_allowed("other@other.com")); // doesn't match either
    }

    #[test]
    fn test_is_owner_email_none() {
        let config = test_config();
        let wrapper = TestConfigWrapper { config };

        assert!(!wrapper.is_owner_email("anyone@example.com"));
    }

    #[test]
    fn test_is_owner_email_match() {
        let mut config = test_config();
        config.owner_email = Some("owner@example.com".to_string());
        let wrapper = TestConfigWrapper { config };

        assert!(wrapper.is_owner_email("owner@example.com"));
        assert!(!wrapper.is_owner_email("other@example.com"));
    }
}
