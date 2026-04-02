use async_trait::async_trait;
use std::sync::Arc;

use endpoint_libs::libs::handler::{RequestHandler, Response};
use endpoint_libs::libs::toolbox::RequestContext;

use crate::codegen::model::{SetTgBotConfigRequest, SetTgBotConfigResponse};
use crate::service::tg_bot_service::TgBotService;

pub struct MethodSetTgBotConfig {
    pub tg_bot_service: Arc<TgBotService>,
}

#[async_trait(?Send)]
impl RequestHandler for MethodSetTgBotConfig {
    type Request = SetTgBotConfigRequest;

    async fn handle(&self, _ctx: RequestContext, req: Self::Request) -> Response<Self::Request> {
        self.tg_bot_service.set_config(req.enabled, req.token).await?;
        Ok(SetTgBotConfigResponse {})
    }
}
