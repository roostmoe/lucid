use bb8::PooledConnection;
use diesel_async::{
    AsyncPgConnection, RunQueryDsl, pooled_connection::{AsyncDieselConnectionManager, bb8::Pool}
};
use lucid_auth::{authz, context::OpContext};
use lucid_common::api::error::Error;

pub mod auth;
pub mod sessions;
pub mod tenant;
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

    pub(super) async fn pool_conn_authorized(
        &self,
        opctx: &OpContext
    ) -> Result<
        PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>,
        Error
    > {
        opctx.authorize(authz::Action::Query, &authz::DATABASE).await?;
        self.pool.get().await
            .map_err(|e| Error::internal_anyhow("failed to get database connection".into(), e.into()))
    }
}
