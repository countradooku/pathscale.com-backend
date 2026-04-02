use async_trait::async_trait;
use std::sync::Arc;

use endpoint_libs::libs::handler::{RequestHandler, Response};
use endpoint_libs::libs::toolbox::RequestContext;

use crate::codegen::model::{GetTgBotConfigRequest, GetTgBotConfigResponse};
use crate::service::tg_bot_service::TgBotService;

pub struct MethodGetTgBotConfig {
    pub tg_bot_service: Arc<TgBotService>,
}

#[async_trait(?Send)]
impl RequestHandler for MethodGetTgBotConfig {
    type Request = GetTgBotConfigRequest;

    async fn handle(&self, _ctx: RequestContext, _req: Self::Request) -> Response<Self::Request> {
        let (enabled, token_set) = self.tg_bot_service.get_config();
        Ok(GetTgBotConfigResponse { enabled, token_set })
    }
}
