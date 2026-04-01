use async_trait::async_trait;
use std::sync::Arc;

use endpoint_libs::libs::handler::{RequestHandler, Response};
use endpoint_libs::libs::toolbox::RequestContext;
use worktable::select::SelectQueryExecutor;

use crate::codegen::model::{GetTgBotConfigRequest, GetTgBotConfigResponse};
use crate::db::schema::tg_bot_config::TgBotConfigWorkTable;

pub struct MethodGetTgBotConfig {
    pub config_table: Arc<TgBotConfigWorkTable>,
}

#[async_trait(?Send)]
impl RequestHandler for MethodGetTgBotConfig {
    type Request = GetTgBotConfigRequest;

    async fn handle(&self, _ctx: RequestContext, _req: Self::Request) -> Response<Self::Request> {
        let config = self.config_table.select_all().execute()?.into_iter().next();

        Ok(GetTgBotConfigResponse {
            enabled: config.as_ref().map(|c| c.enabled).unwrap_or(false),
            token_set: config.and_then(|c| c.token).is_some(),
        })
    }
}
