use crate::error::AppError;

pub struct LucidContext {
    pub public_url: String,
}

impl LucidContext {
    pub async fn new(
        public_url: String,
    ) -> Result<Self, AppError> {
        Ok(Self {
            public_url: public_url,
        })
    }
}
