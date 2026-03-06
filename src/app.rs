mod admin_handlers;
mod auth_handlers;
mod support_handlers;

use std::sync::Arc;

use endpoint_libs::libs::signal::{init_signals, wait_for_signals};
use endpoint_libs::libs::ws::WebsocketServer;
use eyre::{Context, Result, bail};
use honey_id_types::HoneyIdClient;
use honey_id_types::handlers::convenience_utils::token_management::TokenWorkTableStorage;
use tgbot::api::Client;
use tracing::{info, warn};
use uuid::Uuid;

use crate::app::admin_handlers::register_admin_handlers;
use crate::app::auth_handlers::register_auth_handlers;
use crate::app::support_handlers::register_support_handlers;
use crate::config::Config;
use crate::db::database::Db;
use crate::service::support_chat::SupportChatManager;

pub struct AppCtx {
    pub config: Arc<Config>,
    pub db: Arc<Db>,
    pub support_chat_manager: Arc<SupportChatManager>,
}

impl AppCtx {
    pub async fn new(config: Config) -> Result<Self> {
        let db = Arc::new(Db::new(&config.database).await?);

        if let Some(admin_id) = &config.platform_admin_pub_id {
            bootstrap_admin_user(&db, *admin_id).await?;
        }

        let tg_client = Client::new(config.tg_bot.token.clone())?;
        let support_chat_manager = Arc::new(SupportChatManager::new(
            tg_client,
            db.support_user_table.clone(),
            db.support_msg_table.clone(),
        ));

        let app = Self {
            config: Arc::new(config.clone()),
            db,
            support_chat_manager,
        };

        Ok(app)
    }
}

pub struct App {
    ctx: AppCtx,
}

impl App {
    pub async fn new(config: Config) -> Result<Self> {
        Ok(Self {
            ctx: AppCtx::new(config).await?,
        })
    }

    pub fn register_handlers(&self, server: &mut WebsocketServer) {
        let honey_id_client = Arc::new(HoneyIdClient::new(self.ctx.config.honey_id.clone()));
        register_auth_handlers(
            server,
            self.ctx.db.clone(),
            Arc::new(TokenWorkTableStorage::default()),
            honey_id_client,
        );

        register_admin_handlers(server, &self.ctx);
        register_support_handlers(server, &self.ctx);
    }

    pub async fn run(self) -> Result<()> {
        self.init().await?;

        let localset = tokio::task::LocalSet::new();
        let _enter = localset.enter();

        let (mut sigterm, mut sigint) = init_signals()?;
        let mut server = WebsocketServer::new(self.ctx.config.server.clone());

        self.register_handlers(&mut server);

        localset
            .run_until(async {
                tokio::select! {
                    Err(res) = server.listen() => warn!("Server terminated, {res:?}"),
                    _ = self.ctx.support_chat_manager.run() => warn!("Support Bot terminated"),
                    _ = wait_for_signals(&mut sigterm, &mut sigint) => {}
                }
            })
            .await;

        // no matter if it was server issue or thread return signal, go with graceful termination procedure
        tokio::select! {
            _ = self.ctx.db.wait_for_ops() =>{
                warn!("Gracefully terminated all threads");
            },
            _ = tokio::time::sleep(std::time::Duration::from_secs(15)) => {
                std::process::exit(20);
            }
        };

        Ok(())
    }

    async fn init(&self) -> Result<()> {
        let ctx = &self.ctx;
        if ctx.db.user_table.count() == 0 {
            // skip 0 pk for admin user
            ctx.db.user_table.get_next_pk();
        }
        Ok(())
    }
}

async fn bootstrap_admin_user(db: &Db, user_pub_id: Uuid) -> Result<()> {
    tracing::info!(%user_pub_id, "Platform Admin pub ID configured, attempting to assign role to user");
    if let Some(mut user) = db.user_table.select_by_pub_id(user_pub_id) {
        user.role = crate::codegen::model::UserRole::Admin;

        db.user_table
            .update(user)
            .await
            .wrap_err("Failed to update configured platform admin user's role")?;

        info!("Assigned platform administrator role for user {user_pub_id}");

        Ok(())
    } else {
        bail!("Configured platform admin user does not exist. Sign up again and retry with a new ID")
    }
}
