use std::sync::Arc;

use eyre::{Result, eyre};
use parking_lot::Mutex;
use tgbot::api::Client;
use uuid::Uuid;

use crate::db::schema::support::{SupportMessageWorkTable, SupportUserWorkTable};
use crate::db::schema::tg_bot_config::{TgBotConfigRow, TgBotConfigWorkTable};
use crate::service::support_chat::SupportChatManager;

// There is always at most one config row; use a fixed ID for it.
const CONFIG_ROW_ID: i64 = 1;

pub struct TgBotService {
    config_table: Arc<TgBotConfigWorkTable>,
    support_user_table: Arc<SupportUserWorkTable>,
    support_msg_table: Arc<SupportMessageWorkTable>,
    manager: Mutex<Option<Arc<SupportChatManager>>>,
    task_handle: Mutex<Option<tokio::task::AbortHandle>>,
}

impl TgBotService {
    pub fn new(
        config_table: Arc<TgBotConfigWorkTable>,
        support_user_table: Arc<SupportUserWorkTable>,
        support_msg_table: Arc<SupportMessageWorkTable>,
        initial_manager: Option<Arc<SupportChatManager>>,
    ) -> Self {
        Self {
            config_table,
            support_user_table,
            support_msg_table,
            manager: Mutex::new(initial_manager),
            task_handle: Mutex::new(None),
        }
    }

    /// Spawns the initial bot polling task (called once at app startup inside the LocalSet).
    pub fn spawn_initial_task(&self) {
        let manager = self.manager.lock().clone();
        if let Some(m) = manager {
            // spawn_local because tgbot's LongPoll is !Send
            let handle = tokio::task::spawn_local(async move { m.run().await });
            *self.task_handle.lock() = Some(handle.abort_handle());
        }
    }

    /// Returns a clone of the current manager, if the bot is running.
    pub fn current_manager(&self) -> Option<Arc<SupportChatManager>> {
        self.manager.lock().clone()
    }

    /// Returns (enabled, token_set) from the persisted config.
    pub fn get_config(&self) -> (bool, bool) {
        let row = self.config_table.select(CONFIG_ROW_ID);
        (
            row.as_ref().map(|r| r.enabled).unwrap_or(false),
            row.as_ref().and_then(|r| r.token.as_ref()).is_some(),
        )
    }

    /// Persists the new config and, only if something changed, restarts the bot.
    pub async fn set_config(&self, enabled: bool, token: Option<String>) -> Result<()> {
        let existing = self.config_table.select(CONFIG_ROW_ID);

        let current_enabled = existing.as_ref().map(|r| r.enabled).unwrap_or(false);
        let current_token = existing.as_ref().and_then(|r| r.token.clone());
        let token_changed = token.is_some() && token != current_token;
        let enabled_changed = enabled != current_enabled;

        // Effective token: prefer the new one, fall back to what is stored.
        let effective_token = token.clone().or(current_token);

        // Persist config (upsert: update existing row or insert first row).
        if let Some(mut row) = existing {
            row.enabled = enabled;
            if token.is_some() {
                row.token = token;
            }
            self.config_table.update(row).await?;
        } else {
            self.config_table.insert(TgBotConfigRow {
                id: CONFIG_ROW_ID,
                enabled,
                token,
            })?;
        }

        // Nothing that affects the running bot changed → done.
        if !enabled_changed && !token_changed {
            return Ok(());
        }

        // Abort the existing task.
        if let Some(handle) = self.task_handle.lock().take() {
            handle.abort();
        }
        *self.manager.lock() = None;

        // Start a new bot task if now enabled and a token is available.
        if enabled
            && let Some(token) = effective_token
        {
            let tg_client = Client::new(token)?;
            let manager = Arc::new(SupportChatManager::new(
                tg_client,
                self.support_user_table.clone(),
                self.support_msg_table.clone(),
            ));
            let manager_clone = manager.clone();
            // spawn_local because tgbot's LongPoll is !Send
            let handle = tokio::task::spawn_local(async move { manager_clone.run().await });
            *self.task_handle.lock() = Some(handle.abort_handle());
            *self.manager.lock() = Some(manager);
        }

        Ok(())
    }

    /// Sends a message to all support agents via the Telegram bot.
    pub async fn send_msg_to_support(
        &self,
        author_pub_id: Uuid,
        author_name: String,
        msg_content: String,
    ) -> Result<i64> {
        let manager = self
            .manager
            .lock()
            .clone()
            .ok_or_else(|| eyre!("Telegram bot is not configured or disabled"))?;
        manager
            .send_msg_to_support(author_pub_id, author_name, msg_content)
            .await
    }
}
