use std::path::PathBuf;

use config::{Config, File};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const ENV_PREFIX: &str = "BEACON";

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Default)]
pub struct BeaconConfig {
    pub logging: LoggingConfig,
    pub session: SessionConfig,
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

/// Configuration for logging.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct LoggingConfig {
    /// The URL of the database to connect to.
    ///
    /// Allowed values: trace, debug, info, warn, error
    #[schemars(with = "String", example = "df_log_level")]
    pub level: LogLevel,

    /// The file path to write logs to. If not set, logs will be written to
    /// stdout.
    #[schemars(with = "String", example = "df_log_path")]
    pub file_path: Option<PathBuf>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: df_log_level(),
            file_path: df_log_path(),
        }
    }
}

fn df_log_level() -> LogLevel {
    LogLevel::Info
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

/// The configuration for user sessions, including timeouts and expiration
/// policies.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct SessionConfig {
    /// The idle timeout for a session represented in seconds.
    #[schemars(default = "df_idle_timeout_seconds")]
    pub idle_timeout_seconds: i64,

    /// The absolute maximum duration of a session in seconds. After this time,
    /// the session will expire irrespective of activity.
    #[schemars(default = "df_absolute_timeout_seconds")]
    pub absolute_timeout_seconds: i64,

    /// Whether to set the session cookie as _secure_, which means it will only
    /// be sent over HTTPS connections. This should be `true` in production and
    /// `false` in development.
    #[schemars(default = "df_session_secure")]
    pub secure: bool,
}

fn df_idle_timeout_seconds() -> i64 {
    3600 // 1 hour
}

fn df_absolute_timeout_seconds() -> i64 {
    86400 // 24 hours
}

fn df_session_secure() -> bool {
    true
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            idle_timeout_seconds: df_idle_timeout_seconds(),
            absolute_timeout_seconds: df_absolute_timeout_seconds(),
            secure: df_session_secure(),
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
