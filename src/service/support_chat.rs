use std::sync::Arc;

use chrono::Utc;
use eyre::Result;
use futures::SinkExt;
use tgbot::api::Client;
use tgbot::handler::{LongPoll, UpdateHandler};
use tgbot::types::{ChatPeerId, Command, ReplyTo, SendMessage, Update, UpdateType};
use tracing::warn;
use uuid::Uuid;
use worktable::select::SelectQueryExecutor;

use crate::codegen::model::{SubMsgEventsResponse, SupportMessage};
use crate::db::schema::support::{
    SupportMessageRow, SupportMessageWorkTable, SupportUserPrimaryKey, SupportUserRow, SupportUserWorkTable,
};
use crate::handlers::utils::RoutingMessage;
use crate::util::pipeline::{BroadcastPipeConnector, BroadcastPipeConnectorSink, BroadcastPipeConnectorStream};

pub struct SupportChatManager {
    update_handler: SupportUpdateHandler,
    support_user_table: Arc<SupportUserWorkTable>,
    support_msg_table: Arc<SupportMessageWorkTable>,
    event_connector: BroadcastPipeConnector<RoutingMessage<Uuid, SubMsgEventsResponse>>,
}

impl SupportChatManager {
    pub fn new(
        tg_client: Client,
        support_user_table: Arc<SupportUserWorkTable>,
        support_msg_table: Arc<SupportMessageWorkTable>,
    ) -> Self {
        let event_connector = BroadcastPipeConnector::new(100);
        Self {
            update_handler: SupportUpdateHandler::new(
                tg_client,
                support_user_table.clone(),
                support_msg_table.clone(),
                event_connector.sink(),
            ),
            support_user_table,
            support_msg_table,
            event_connector,
        }
    }

    pub fn event_stream(&self) -> BroadcastPipeConnectorStream<RoutingMessage<Uuid, SubMsgEventsResponse>> {
        self.event_connector.stream()
    }

    pub async fn run(&self) {
        LongPoll::new(self.update_handler.client.clone(), self.update_handler.clone())
            .run()
            .await;
    }

    pub async fn send_msg_to_support(
        &self,
        author_pub_id: Uuid,
        author_name: String,
        msg_content: String,
    ) -> Result<i64> {
        // Send the user msg to all supports connected to the bot
        let supports = self
            .support_user_table
            .select_all()
            .execute()?
            .into_iter()
            .filter_map(|row| row.chat_id);
        let msg = format!(
            "{}\nfrom: {}\n{}",
            author_pub_id,
            author_name.clone(),
            msg_content.clone()
        );
        let sent_at = Utc::now().timestamp_millis();
        for chat_id in supports {
            self.support_msg_table.insert(SupportMessageRow {
                id: self.support_msg_table.get_next_pk().into(),
                incoming: false,
                sent_by: author_name.clone(),
                user_pub_id: author_pub_id,
                chat_id,
                sent_at,
                content: msg_content.clone(),
            })?;
            let method = SendMessage::new(ChatPeerId::from(chat_id), msg.clone());
            self.update_handler.client.execute(method).await?;
        }

        let response = SubMsgEventsResponse {
            data: vec![SupportMessage {
                incoming: false,
                sentBy: author_name,
                sentAt: sent_at.to_string(),
                content: msg_content,
            }],
        };
        let routing_msg = RoutingMessage::for_concrete(author_pub_id, response);
        // TODO: do something with that.....
        let mut sink = self.event_connector.sink();
        let _ = sink.send(routing_msg).await;

        Ok(sent_at)
    }

    pub async fn list_support_msgs(&self, user_pub_id: Uuid) -> Result<Vec<SupportMessage>> {
        let msgs: Vec<SupportMessage> = self
            .support_msg_table
            .select_by_user_pub_id(user_pub_id)
            .execute()?
            .iter()
            .map(|row| SupportMessage {
                incoming: row.incoming,
                sentBy: row.sent_by.clone(),
                sentAt: row.sent_at.to_string(),
                content: row.content.clone(),
            })
            .collect();
        Ok(msgs)
    }
}

#[derive(Clone)]
struct SupportUpdateHandler {
    client: Client,
    support_user_table: Arc<SupportUserWorkTable>,
    support_msg_table: Arc<SupportMessageWorkTable>,
    event_sink: BroadcastPipeConnectorSink<RoutingMessage<Uuid, SubMsgEventsResponse>>,
}

impl SupportUpdateHandler {
    fn new(
        client: Client,
        support_user_table: Arc<SupportUserWorkTable>,
        support_msg_table: Arc<SupportMessageWorkTable>,
        event_sink: BroadcastPipeConnectorSink<RoutingMessage<Uuid, SubMsgEventsResponse>>,
    ) -> Self {
        Self {
            client,
            support_user_table,
            support_msg_table,
            event_sink,
        }
    }

    /// tries to send a message as a reaction to an update
    async fn try_send_msg_internal<S: Into<String>>(&self, chat_id: i64, msg: S) {
        let _ = self
            .client
            .execute(SendMessage::new(ChatPeerId::from(chat_id), msg))
            .await
            .inspect_err(|e| warn!("Error sending message: {e:?}"));
    }
}

impl UpdateHandler for SupportUpdateHandler {
    async fn handle(&self, update: Update) {
        let UpdateType::Message(message) = update.update_type else {
            return;
        };
        let chat_id: i64 = message.chat.get_id().into();

        let Some(user_handle) = message.chat.get_username() else {
            self.try_send_msg_internal(chat_id, "Couldn't fetch user handle").await;
            return;
        };

        let Some(existing_user) = self
            .support_user_table
            .select(SupportUserPrimaryKey::from(format!("@{user_handle}")))
        else {
            self.try_send_msg_internal(chat_id, "Not verified").await;
            return;
        };

        // TODO: split this into separate methods
        if let Some(ReplyTo::Message(ref origin_msg)) = message.reply_to {
            if let Some(origin_txt) = origin_msg.get_text()
                && let Some(user_id_str) = origin_txt.data.lines().nth(0)
                && let Ok(user_id) = Uuid::parse_str(user_id_str)
            {
                let Some(reply_txt) = message.get_text() else {
                    self.try_send_msg_internal(chat_id, "Error fetching message text").await;
                    return;
                };

                let sent_at = Utc::now().timestamp_millis();
                if let Err(e) = self.support_msg_table.insert(SupportMessageRow {
                    id: self.support_msg_table.get_next_pk().into(),
                    sent_by: "Support".to_string(),
                    incoming: true,
                    user_pub_id: user_id,
                    chat_id,
                    sent_at,
                    content: reply_txt.data.clone(),
                }) {
                    warn!("Error saving support msg: {e:?}");
                    self.try_send_msg_internal(chat_id, "Internal Server Error").await;
                    return;
                };

                let response = SubMsgEventsResponse {
                    data: vec![SupportMessage {
                        incoming: true,
                        sentBy: "Support".to_string(),
                        sentAt: sent_at.to_string(),
                        content: reply_txt.data.clone(),
                    }],
                };

                let routing_msg = RoutingMessage::for_concrete(user_id, response);
                // TODO: do something with that.....
                let mut sink = self.event_sink.clone();
                let _ = sink.send(routing_msg).await;
            } else {
                self.try_send_msg_internal(chat_id, "Guest ID not found in reply").await;
            }
        } else if let Ok(cmd) = Command::try_from(message.as_ref().clone())
            && cmd.get_name() == "/start"
        {
            let updated_row = SupportUserRow {
                handle: existing_user.handle,
                chat_id: Some(chat_id),
            };
            if let Err(e) = self.support_user_table.update(updated_row).await {
                warn!("Error updating support chat id: {e:?}");
                self.try_send_msg_internal(chat_id, "Internal Server Error").await;
            } else {
                self.try_send_msg_internal(chat_id, "Your chat is saved for future use")
                    .await;
            };
        } else {
            self.try_send_msg_internal(chat_id, "Please, supply a valid command")
                .await;
        };
    }
}
