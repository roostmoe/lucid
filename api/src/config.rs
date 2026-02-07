use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use tracing::level_filters::LevelFilter;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub log_format: ServerLogFormat,
    pub log_level: ServerLogLevel,
    pub public_url: String,
    pub server_port: u16,
    pub database_url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServerLogFormat {
    Json,
    Pretty,
    Logfmt,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServerLogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error
}

impl From<ServerLogLevel> for LevelFilter {
    fn from(value: ServerLogLevel) -> Self {
        match value {
            ServerLogLevel::Trace => LevelFilter::TRACE,
            ServerLogLevel::Debug => LevelFilter::DEBUG,
            ServerLogLevel::Info => LevelFilter::INFO,
            ServerLogLevel::Warn => LevelFilter::WARN,
            ServerLogLevel::Error => LevelFilter::ERROR,
        }        
    }
}

impl AppConfig {
    pub fn new(config_sources: Option<Vec<String>>) -> Result<Self, ConfigError> {
        let mut config = Config::builder()
            .add_source(File::with_name("config.toml").required(false))
            .add_source(File::with_name("lucid-api/config.toml").required(false));

        for source in config_sources.unwrap_or_default() {
            config = config.add_source(File::with_name(&source).required(false));
        }

        config
            .add_source(Environment::default())
            .build()?
            .try_deserialize()
    }
}
