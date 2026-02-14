use diesel_async::{
    AsyncPgConnection, RunQueryDsl,
    pooled_connection::{AsyncDieselConnectionManager, bb8::Pool},
};

pub mod users;

pub struct DataStore {
    pool: Pool<AsyncPgConnection>,
}

impl DataStore {
    pub async fn open(database_url: String) -> anyhow::Result<Self> {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
        let pool = Pool::builder().build(config).await?;

        Ok(Self { pool })
    }

    pub async fn ping(&self) -> anyhow::Result<()> {
        let mut conn = self.pool.get().await?;
        diesel::sql_query("SELECT 1").execute(&mut conn).await?;
        Ok(())
    }
}
