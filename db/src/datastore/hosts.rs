use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lucid_db_models::Host;
use lucid_db_schema::schema::inventory_hosts;

use crate::datastore::DataStore;

impl DataStore {
    pub async fn host_list(&self, deleted: bool) -> anyhow::Result<Vec<Host>> {
        let mut conn = self.pool.get().await?;

        let mut host_query = inventory_hosts::table
            .order_by(inventory_hosts::created_at.desc())
            .into_boxed();

        if !deleted {
            host_query = host_query.filter(inventory_hosts::deleted_at.is_null());
        }

        let hosts = host_query
            .select(Host::as_select())
            .get_results(&mut conn)
            .await?;

        Ok(hosts)
    }
}
