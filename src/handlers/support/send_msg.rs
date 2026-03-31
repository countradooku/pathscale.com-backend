use async_trait::async_trait;
use honey_id_types::HoneyIdConnection;
use std::sync::Arc;
use tracing::{debug, error, info};
use url::Url;

use endpoint_libs::libs::{
    error_code::ErrorCode,
    handler::{RequestHandler, Response},
    toolbox::{CustomError, RequestContext},
    ws::WsRequest,
};

use crate::{
    codegen::model::{SendMsgRequest, SendMsgResponse},
    db::schema::user::UserWorkTable,
    service::support_chat::SupportChatManager,
};

pub struct MethodSendMsg {
    pub chat_manager: Option<Arc<SupportChatManager>>,
    pub user_table: Arc<UserWorkTable>,
}

#[async_trait(?Send)]
impl RequestHandler for MethodSendMsg {
    type Request = SendMsgRequest;

    async fn handle(&self, _ctx: RequestContext, req: Self::Request) -> Response<Self::Request> {
        // let user_id = ctx.user_id as i64;
        // let user = self
        //     .user_table
        //     .select(user_id)
        //     .ok_or(eyre!("Error fetching user with ID: {user_id}"))?;
        // self.chat_manager
        //     .send_msg_to_support(user.pub_id, user.username, req.message)
        //     .await?;
        // Ok(SendMsgResponse {})

        info!(server = %req.server, "Proxying SendMsg request");

        let url = Url::parse(&req.server).map_err(|e| {
            error!(server = %req.server, err = %e, "Failed to parse server URL");
            CustomError::new(ErrorCode::BAD_REQUEST, format!("Invalid server URL: {e}"))
        })?;

        debug!(url = %url, "Connecting to server via WebSocket");
        let mut conn = HoneyIdConnection::connect(&url, None)
            .await
            .map_err(|e| {
                error!(url = %url, err = %e, "Failed to establish WebSocket connection");
                CustomError::new(ErrorCode::BAD_REQUEST, format!("Failed to connect to server: {e}"))
            })?;
        info!(url = %url, "WebSocket connection established");

        debug!(method_id = SendMsgRequest::METHOD_ID, "Sending request");
        conn.send_request_raw(SendMsgRequest::METHOD_ID, &req).await?;
        debug!("Request sent, waiting for response");

        let response: SendMsgResponse = conn.receive_response().await.map_err(|e| {
            error!(url = %url, err = %e, "Failed to receive response from server");
            CustomError::new(ErrorCode::BAD_REQUEST, format!("Failed to receive response from server: {e}"))
        })?;
        info!(url = %url, "Received response from server");

        Ok(response)
    }
}
