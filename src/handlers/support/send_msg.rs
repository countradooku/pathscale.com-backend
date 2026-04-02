use async_trait::async_trait;
use std::sync::Arc;

use endpoint_libs::libs::handler::{RequestHandler, Response};
use endpoint_libs::libs::toolbox::RequestContext;
use eyre::eyre;

use crate::codegen::model::{SendMsgRequest, SendMsgResponse};
use crate::db::schema::user::UserWorkTable;
use crate::service::tg_bot_service::TgBotService;

pub struct MethodSendMsg {
    pub tg_bot_service: Arc<TgBotService>,
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
        self.tg_bot_service
            .send_msg_to_support(user.pub_id, user.username, req.message)
            .await?;
        Ok(SendMsgResponse {})
    }
}
