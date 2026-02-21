#[derive(Debug, Clone)]
pub struct ApiContext {
    pub config: crate::config::LucidApiConfig,
}

impl ApiContext {
    pub fn new(config: crate::config::LucidApiConfig) -> Self {
        Self { config }
    }
}
