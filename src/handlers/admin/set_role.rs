use async_trait::async_trait;
use eyre::eyre;
use std::sync::Arc;

use endpoint_libs::libs::handler::{RequestHandler, Response};
use endpoint_libs::libs::toolbox::RequestContext;

use crate::codegen::model::{SetRoleRequest, SetRoleResponse};
use crate::db::schema::user::{RoleByIdQuery, UserWorkTable};

pub struct MethodSetRole {
    pub user_table: Arc<UserWorkTable>,
}

#[async_trait(?Send)]
impl RequestHandler for MethodSetRole {
    type Request = SetRoleRequest;

    async fn handle(&self, _ctx: RequestContext, req: Self::Request) -> Response<Self::Request> {
        let user_pub_id = req.userPubId;
        let role = req.role;

        let user = self
            .user_table
            .select_by_pub_id(user_pub_id)
            .ok_or(eyre!(format!("Error fetching user with pub ID {user_pub_id}")))?;
        self.user_table
            .update_role_by_id(RoleByIdQuery { role }, user.id)
            .await?;

        Ok(SetRoleResponse {})
    }
}
