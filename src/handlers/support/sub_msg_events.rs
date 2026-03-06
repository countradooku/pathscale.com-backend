use async_trait::async_trait;
use eyre::eyre;
use std::sync::Arc;
use uuid::Uuid;

use endpoint_libs::libs::{
    handler::{RequestHandler, Response},
    toolbox::RequestContext,
};

use crate::{
    codegen::model::{SubMsgEventsRequest, SubMsgEventsResponse},
    db::schema::user::UserWorkTable,
    handlers::utils::SubscriptionRouter,
};

pub struct MethodSubMsgEvents {
    pub router: SubscriptionRouter<Uuid, SubMsgEventsResponse>,
    pub user_table: Arc<UserWorkTable>,
}

#[async_trait(?Send)]
impl RequestHandler for MethodSubMsgEvents {
    type Request = SubMsgEventsRequest;

    async fn handle(&self, ctx: RequestContext, req: Self::Request) -> Response<Self::Request> {
        if let Some(true) = req.unsub {
            self.router.unsubscribe(ctx.connection_id).await;
        } else {
            let user_id = ctx.user_id as i64;

            let user = self
                .user_table
                .select(user_id)
                .ok_or(eyre!("Error fetching user with ID: {user_id}"))?;
            self.router.subscribe(ctx, vec![user.pub_id]).await;
        }
        Ok(SubMsgEventsResponse { data: vec![] })
    }
}
