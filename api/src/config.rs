use chrono::Duration;
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LucidConfig {
    pub database_url: String,
    pub bind_address: String,
    pub mode: ServerMode,
    pub session: SessionConfig,
}

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub enum ServerMode {
    /// Production configures the following side effects:
    /// - Cookies set to _secure_
    #[serde(rename = "prod")]
    #[default]
    Production,

    /// Development configures the following side effects:
    /// - Cookies set to _insecure_
    #[serde(rename = "dev")]
    Development
}

#[derive(Debug, Deserialize)]
pub struct SessionConfig {
    pub idle_timeout: Duration,
    pub absolute_timeout: Duration,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            idle_timeout: Duration::hours(24),
            absolute_timeout: Duration::days(7),
        }
    }
}

impl LucidConfig {
    pub fn new(config_sources: Option<Vec<String>>) -> Result<LucidConfig, ConfigError> {
        let mut config = Config::builder()
            .add_source(File::with_name("api/config.toml").required(false))
            .add_source(File::with_name("config.toml").required(false));

        for source in config_sources.unwrap_or_default() {
            config = config.add_source(File::with_name(&source).required(true));
        }

        config
            .add_source(Environment::default().prefix("LUCID"))
            .build()?
            .try_deserialize()
    }
}
