use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LucidConfig {
    pub database_url: String,
    pub bind_address: String,
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
