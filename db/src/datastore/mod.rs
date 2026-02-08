use diesel_async::{
    pooled_connection::{AsyncDieselConnectionManager, bb8::Pool},
    AsyncPgConnection,
};

pub mod tenant;
pub mod authz;

pub struct DataStore {
    pool: Pool<AsyncPgConnection>,
}

impl DataStore {
    pub async fn open(database_url: String) -> anyhow::Result<Self> {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
        let pool = Pool::builder().build(config).await?;

        Ok(Self { pool })
    }
}
