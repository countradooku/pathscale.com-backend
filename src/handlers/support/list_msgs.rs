use eyre::eyre;
use std::sync::Arc;

use async_trait::async_trait;
use endpoint_libs::libs::{
    handler::{RequestHandler, Response},
    toolbox::RequestContext,
};

use crate::{
    codegen::model::{ListMsgsRequest, ListMsgsResponse},
    db::schema::user::UserWorkTable,
    service::support_chat::SupportChatManager,
};

pub struct MethodListMsgs {
    pub chat_manager: Arc<SupportChatManager>,
    pub user_table: Arc<UserWorkTable>,
}

#[async_trait(?Send)]
impl RequestHandler for MethodListMsgs {
    type Request = ListMsgsRequest;

    async fn handle(&self, ctx: RequestContext, _req: Self::Request) -> Response<Self::Request> {
        let user_id = ctx.user_id as i64;
        let user = self
            .user_table
            .select(user_id)
            .ok_or(eyre!("Error fetching user with ID: {user_id}"))?;
        let msgs = self.chat_manager.list_support_msgs(user.pub_id).await?;
        Ok(ListMsgsResponse { data: msgs })
    }
}
