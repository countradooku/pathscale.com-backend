use async_trait::async_trait;
use eyre::eyre;
use std::sync::Arc;

use endpoint_libs::libs::{
    handler::{RequestHandler, Response},
    toolbox::RequestContext,
};

use crate::{
    codegen::model::{SendMsgRequest, SendMsgResponse},
    db::schema::user::UserWorkTable,
    service::support_chat::SupportChatManager,
};

pub struct MethodSendMsg {
    pub chat_manager: Arc<SupportChatManager>,
    pub user_table: Arc<UserWorkTable>,
}

#[async_trait(?Send)]
impl RequestHandler for MethodSendMsg {
    type Request = SendMsgRequest;

    async fn handle(&self, ctx: RequestContext, req: Self::Request) -> Response<Self::Request> {
        let user_id = ctx.user_id as i64;
        let user = self
            .user_table
            .select(user_id)
            .ok_or(eyre!("Error fetching user with ID: {user_id}"))?;
        self.chat_manager
            .send_msg_to_support(user.pub_id, user.username, req.message)
            .await?;
        Ok(SendMsgResponse {})
    }
}
