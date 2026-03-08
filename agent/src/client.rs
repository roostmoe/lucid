use anyhow::{Context, Result};
use lucid_common::{
    params::RegisterAgentRequest,
    views::{ApiErrorResponse, RegisterAgentResponse},
};
use reqwest::{
    Certificate, Client, Identity,
    header::{HeaderMap, HeaderValue},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiClientError {
    #[allow(dead_code)]
    #[error("Missing credentials for API client")]
    MissingCredentials,

    #[error("API error")]
    ApiError(ApiErrorResponse),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    #[error("Failed to load identity from PEM: {0}")]
    IdentityError(reqwest::Error),

    #[error("Request failed: {0}")]
    ReqwestError(#[from] reqwest::Error),
}

#[derive(Default)]
pub struct ApiClient {
    api_url: String,
    client: Client,
    identity: Option<Identity>,
    cert: Option<Certificate>,
}

impl ApiClient {
    pub fn new(
        api_url: String,
        key_pem: Option<String>,
        cert_pem: Option<String>,
        ca_cert_pem: Option<String>,
    ) -> Result<Self, ApiClientError> {
        let mut api_client = ApiClient {
            api_url,
            ..Default::default()
        };

        let mut client_builder =
            Client::builder().user_agent(format!("lucid-agent/{}", env!("CARGO_PKG_VERSION")));

        if let (Some(key_pem), Some(cert_pem), Some(ca_cert_pem)) = (key_pem, cert_pem, ca_cert_pem)
        {
            let identity = Identity::from_pem(
                &(key_pem
                    .into_bytes()
                    .into_iter()
                    .chain(cert_pem.into_bytes())
                    .collect::<Vec<u8>>()),
            )
            .map_err(ApiClientError::IdentityError)?;

            let cert = Certificate::from_pem(&ca_cert_pem.into_bytes())
                .map_err(ApiClientError::IdentityError)?;

            client_builder = client_builder
                .identity(identity.clone())
                .add_root_certificate(cert.clone());

            api_client.identity = Some(identity);
            api_client.cert = Some(cert);
        }

        api_client.client = client_builder
            .build()
            .map_err(ApiClientError::IdentityError)?;

        Ok(api_client)
    }

    #[allow(dead_code)]
    async fn get<TResult>(&self, path: &str) -> Result<TResult, ApiClientError>
    where
        TResult: serde::de::DeserializeOwned,
    {
        let url = format!(
            "{}/{}",
            self.api_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.json::<ApiErrorResponse>().await.map_err(|e| {
                anyhow::anyhow!(
                    "POST {} failed with status {} and invalid error response: {}",
                    url,
                    status,
                    e
                )
            })?;
            return Err(ApiClientError::ApiError(body));
        }

        response
            .json::<TResult>()
            .await
            .map_err(ApiClientError::ReqwestError)
    }

    async fn post<TBody, TResult>(
        &self,
        path: &str,
        body: &TBody,
        headers: Option<HeaderMap<HeaderValue>>,
    ) -> Result<TResult, ApiClientError>
    where
        TBody: serde::ser::Serialize,
        TResult: serde::de::DeserializeOwned,
    {
        let url = format!(
            "{}/{}",
            self.api_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let mut req = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(body);

        if let Some(headers) = headers {
            req = req.headers(headers);
        }

        let response = req.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.json::<ApiErrorResponse>().await.map_err(|e| {
                anyhow::anyhow!(
                    "POST {} failed with status {} and invalid error response: {}",
                    url,
                    status,
                    e
                )
            })?;
            return Err(ApiClientError::ApiError(body));
        }

        Ok(response
            .json::<TResult>()
            .await
            .context("Failed to parse registration response")?)
    }

    pub async fn register(
        &self,
        token: String,
        csr_pem: String,
        hostname: String,
    ) -> Result<RegisterAgentResponse, ApiClientError> {
        let request = RegisterAgentRequest { csr_pem, hostname };
        self.post(
            "/api/v1/agents/register",
            &request,
            Some({
                let mut headers = HeaderMap::new();
                headers.insert(
                    "Authorization",
                    HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
                );
                headers
            }),
        )
        .await
    }
}
