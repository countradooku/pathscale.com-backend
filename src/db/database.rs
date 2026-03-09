use eyre::Result;
use std::sync::Arc;

use worktable::prelude::PersistenceConfig;

use crate::config::DatabaseConfig;
use crate::db::schema::support::{SupportMessageWorkTable, SupportUserWorkTable};
use crate::db::schema::user::UserWorkTable;
use crate::db::schema::waitlist::WaitlistWorkTable;

pub struct Db {
    pub user_table: Arc<UserWorkTable>,
    pub support_user_table: Arc<SupportUserWorkTable>,
    pub support_msg_table: Arc<SupportMessageWorkTable>,
    pub waitlist_table: Arc<WaitlistWorkTable>,
}

impl Db {
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let data_directory = config.path.to_string_lossy().to_string();
        let wt_config = PersistenceConfig::new(&data_directory, &data_directory);

        let user_table = Arc::new(UserWorkTable::load_from_file(wt_config.clone()).await?);
        let support_user_table = Arc::new(SupportUserWorkTable::load_from_file(wt_config.clone()).await?);
        let support_msg_table = Arc::new(SupportMessageWorkTable::load_from_file(wt_config.clone()).await?);
        let waitlist_table = Arc::new(WaitlistWorkTable::new(wt_config.clone()).await?);

        Ok(Self {
            user_table,
            support_user_table,
            support_msg_table,
            waitlist_table,
        })
    }

    pub async fn wait_for_ops(&self) {
        self.user_table.wait_for_ops().await;
        self.support_user_table.wait_for_ops().await;
        self.support_msg_table.wait_for_ops().await;
        self.waitlist_table.wait_for_ops().await;
    }
}
