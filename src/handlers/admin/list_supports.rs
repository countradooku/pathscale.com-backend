use async_trait::async_trait;
use std::sync::Arc;

use endpoint_libs::libs::{
    handler::{RequestHandler, Response},
    toolbox::RequestContext,
};
use worktable::select::SelectQueryExecutor;

use crate::{
    codegen::model::{ListSupportsRequest, ListSupportsResponse},
    db::schema::support::SupportUserWorkTable,
};

pub struct MethodListSupports {
    pub support_user_table: Arc<SupportUserWorkTable>,
}

#[async_trait(?Send)]
impl RequestHandler for MethodListSupports {
    type Request = ListSupportsRequest;

    async fn handle(&self, _ctx: RequestContext, _req: Self::Request) -> Response<Self::Request> {
        Ok(ListSupportsResponse {
            tg_handles: self
                .support_user_table
                .select_all()
                .execute()?
                .iter()
                .map(|row| row.handle.clone())
                .collect(),
        })
    }
}
