use async_trait::async_trait;
use std::sync::Arc;

use endpoint_libs::libs::{
    handler::{RequestHandler, Response},
    toolbox::RequestContext,
};

use crate::{
    codegen::model::{RemoveSupportsRequest, RemoveSupportsResponse},
    db::schema::support::SupportUserWorkTable,
};

pub struct MethodRemoveSupports {
    pub support_user_table: Arc<SupportUserWorkTable>,
}

#[async_trait(?Send)]
impl RequestHandler for MethodRemoveSupports {
    type Request = RemoveSupportsRequest;

    async fn handle(&self, _ctx: RequestContext, req: Self::Request) -> Response<Self::Request> {
        for handle in req
            .tg_handles
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            self.support_user_table.delete(handle).await?;
        }
        Ok(RemoveSupportsResponse {})
    }
}
