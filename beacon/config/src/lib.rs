use std::{net::SocketAddr, path::PathBuf, str::FromStr};

use config::{Config, File};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const ENV_PREFIX: &str = "BEACON";

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Default)]
pub struct BeaconConfig {
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub auth: AuthConfig,
    pub database: DatabaseConfig,
}

impl BeaconConfig {
    /// Creates a new `BeaconConfig` by loading configuration from the
    /// provided sources in order of precedence:
    ///
    /// 1. Default values defined in the code.
    /// 2. Configuration files: `beacon/config.toml` and `config.toml`
    /// 3. Additional configuration files specified in `config_sources`.
    /// 4. Environment variables prefixed with `BEACON_`
    pub fn new(config_sources: Option<Vec<String>>) -> Result<Self, config::ConfigError> {
        let mut config = Config::builder()
            .add_source(File::with_name("beacon/config.toml").required(false))
            .add_source(File::with_name("config.toml").required(false));

        for source in config_sources.unwrap_or_default() {
            config = config.add_source(config::File::with_name(&source).required(true));
        }

        config
            .add_source(config::Environment::default().prefix(ENV_PREFIX))
            .build()?
            .try_deserialize()
    }
}

/// Configuration for the server.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ServerConfig {
    /// The address to mount the server on
    #[schemars(with = "String", example = "df_bind_addr")]
    pub bind_addr: SocketAddr,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: df_bind_addr(),
        }
    }
}

fn df_bind_addr() -> SocketAddr {
    SocketAddr::from_str("0.0.0.0:8080").expect("failed building default bind addr")
}

/// Configuration for logging.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct LoggingConfig {
    /// The URL of the database to connect to.
    ///
    /// Allowed values: trace, debug, info, warn, error
    #[schemars(with = "String", example = "df_log_level")]
    pub level: LogLevel,

    /// The format to emit logs in.
    ///
    /// Allowed values: json, pretty
    #[schemars(with = "String", example = "df_log_format")]
    pub format: LogFormat,

    /// The file path to write logs to. If not set, logs will be written to
    /// stdout.
    #[schemars(with = "String", example = "df_log_path")]
    pub file_path: Option<PathBuf>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: df_log_level(),
            format: df_log_format(),
            file_path: df_log_path(),
        }
    }
}

fn df_log_level() -> LogLevel {
    LogLevel::Info
}

fn df_log_format() -> LogFormat {
    LogFormat::Json
}

fn df_log_path() -> Option<PathBuf> {
    Some(PathBuf::from("/var/log/lucid/beacon.log"))
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Pretty,
}

/// Configuration for authentication (OIDC + JWT)
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct AuthConfig {
    pub oidc: OidcConfig,
    pub jwt: JwtConfig,
}

/// OIDC provider configuration
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct OidcConfig {
    /// OIDC discovery URL (e.g., https://accounts.google.com/.well-known/openid-configuration)
    pub discovery_url: String,
    /// OAuth2 client ID
    pub client_id: String,
    /// OAuth2 client secret
    pub client_secret: String,
    /// Redirect URI for OAuth2 callback
    pub redirect_uri: String,
    /// Optional list of allowed email domains (e.g., ["example.com"])
    pub allowed_domains: Option<Vec<String>>,
    /// Optional list of allowed email addresses
    pub allowed_emails: Option<Vec<String>>,
}

impl Default for OidcConfig {
    fn default() -> Self {
        Self {
            discovery_url: String::new(),
            client_id: String::new(),
            client_secret: String::new(),
            redirect_uri: "http://localhost:8080/auth/callback".to_string(),
            allowed_domains: None,
            allowed_emails: None,
        }
    }
}

/// JWT configuration
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct JwtConfig {
    /// JWT signing secret
    pub secret: String,
    /// JWT issuer claim
    pub issuer: String,
    /// JWT audience claim
    pub audience: String,
    /// Token expiry in hours
    pub expiry_hours: i64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: String::new(),
            issuer: "lucid".to_string(),
            audience: "lucid-api".to_string(),
            expiry_hours: 24,
        }
    }
}

/// Configuration for the database connection.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DatabaseConfig {
    /// The URL of the database to connect to.
    #[schemars(with = "String", example = "ex_db_url")]
    pub url: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self { url: "".into() }
    }
}

fn ex_db_url() -> String {
    "postgres://<user>:<password>@<host>:<port>/<database>".into()
}
