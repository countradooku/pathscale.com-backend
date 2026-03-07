use async_trait::async_trait;
use std::sync::Arc;

use endpoint_libs::libs::{
    handler::{RequestHandler, Response},
    toolbox::RequestContext,
};

use crate::{
    codegen::model::{AddSupportsRequest, AddSupportsResponse},
    db::schema::support::{SupportUserRow, SupportUserWorkTable},
};

pub struct MethodAddSupports {
    pub support_user_table: Arc<SupportUserWorkTable>,
}

#[async_trait(?Send)]
impl RequestHandler for MethodAddSupports {
    type Request = AddSupportsRequest;

    async fn handle(&self, _ctx: RequestContext, req: Self::Request) -> Response<Self::Request> {
        for handle in req
            .tg_handles
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            self.support_user_table
                .insert(SupportUserRow { handle, chat_id: None })?;
        }
        Ok(AddSupportsResponse {})
    }
}
