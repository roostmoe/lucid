#[derive(Debug, Clone)]
pub struct ApiContext {
    pub _config: crate::config::LucidApiConfig,
}

impl ApiContext {
    pub fn new(config: crate::config::LucidApiConfig) -> Self {
        Self { _config: config }
    }
}
