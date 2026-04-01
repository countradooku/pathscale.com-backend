use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use endpoint_libs::libs::handler::{RequestHandler, Response};
use endpoint_libs::libs::toolbox::RequestContext;
use tgbot::api::Client;
use worktable::select::SelectQueryExecutor;

use crate::codegen::model::{SetTgBotConfigRequest, SetTgBotConfigResponse};
use crate::db::schema::support::{SupportMessageWorkTable, SupportUserWorkTable};
use crate::db::schema::tg_bot_config::{TgBotConfigRow, TgBotConfigWorkTable};
use crate::service::support_chat::SupportChatManager;

pub struct MethodSetTgBotConfig {
    pub config_table: Arc<TgBotConfigWorkTable>,
    pub support_chat_manager: Arc<Mutex<Option<Arc<SupportChatManager>>>>,
    pub tg_bot_task: Arc<Mutex<Option<tokio::task::AbortHandle>>>,
    pub support_user_table: Arc<SupportUserWorkTable>,
    pub support_msg_table: Arc<SupportMessageWorkTable>,
}

#[async_trait(?Send)]
impl RequestHandler for MethodSetTgBotConfig {
    type Request = SetTgBotConfigRequest;

    async fn handle(&self, _ctx: RequestContext, req: Self::Request) -> Response<Self::Request> {
        let existing = self.config_table.select_all().execute()?.into_iter().next();

        // Determine the effective token: use new one from request, or keep the existing one
        let effective_token = req.token.clone().or_else(|| existing.as_ref().and_then(|r| r.token.clone()));

        // Persist config to DB
        if let Some(mut row) = existing {
            row.enabled = req.enabled;
            if req.token.is_some() {
                row.token = req.token.clone();
            }
            self.config_table.update(row).await?;
        } else {
            self.config_table.insert(TgBotConfigRow {
                id: self.config_table.get_next_pk().into(),
                enabled: req.enabled,
                token: req.token.clone(),
            })?;
        }

        // Abort existing bot task
        {
            let mut task = self.tg_bot_task.lock().unwrap();
            if let Some(handle) = task.take() {
                handle.abort();
            }
        }
        *self.support_chat_manager.lock().unwrap() = None;

        // Start new bot if enabled and token is available
        if req.enabled {
            if let Some(token) = effective_token {
                let tg_client = Client::new(token)?;
                let manager = Arc::new(SupportChatManager::new(
                    tg_client,
                    self.support_user_table.clone(),
                    self.support_msg_table.clone(),
                ));

                let manager_clone = manager.clone();
                let handle = tokio::task::spawn_local(async move {
                    manager_clone.run().await;
                });

                *self.tg_bot_task.lock().unwrap() = Some(handle.abort_handle());
                *self.support_chat_manager.lock().unwrap() = Some(manager);
            }
        }

        Ok(SetTgBotConfigResponse {})
    }
}
