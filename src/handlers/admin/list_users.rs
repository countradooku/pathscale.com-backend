use async_trait::async_trait;
use std::sync::Arc;

use endpoint_libs::libs::handler::{RequestHandler, Response};
use endpoint_libs::libs::toolbox::RequestContext;
use worktable::select::SelectQueryExecutor;

use crate::codegen::model::{ListUsersRequest, ListUsersResponse, UserView};
use crate::db::schema::user::UserWorkTable;

pub struct MethodListUsers {
    pub user_table: Arc<UserWorkTable>,
}

#[async_trait(?Send)]
impl RequestHandler for MethodListUsers {
    type Request = ListUsersRequest;

    async fn handle(&self, _ctx: RequestContext, _req: Self::Request) -> Response<Self::Request> {
        Ok(ListUsersResponse {
            data: self
                .user_table
                .select_all()
                .execute()?
                .into_iter()
                .map(|user| UserView {
                    userPubId: user.pub_id,
                    username: user.username,
                    role: user.role,
                })
                .collect(),
        })
    }
}
