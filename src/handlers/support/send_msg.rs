use async_trait::async_trait;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, info};

use endpoint_libs::libs::{
    error_code::ErrorCode,
    handler::{RequestHandler, Response},
    toolbox::{CustomError, RequestContext},
    ws::WsClient,
};

use crate::{
    codegen::model::{SendMsgRequest, SendMsgResponse},
    db::schema::user::UserWorkTable,
    service::support_chat::SupportChatManager,
};

#[derive(Serialize)]
struct ForwardedMsg<'a> {
    message: &'a str,
}

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

        debug!(server = %req.server, "Connecting to server via WsClient");
        let mut client = timeout(Duration::from_secs(30), WsClient::new(&req.server, "", None))
            .await
            .map_err(|_| {
                error!(server = %req.server, "Timed out connecting to server");
                CustomError::new(ErrorCode::BAD_REQUEST, "Timed out connecting to server")
            })?
            .map_err(|e| {
                error!(server = %req.server, err = %e, "Failed to establish WebSocket connection");
                CustomError::new(ErrorCode::BAD_REQUEST, format!("Failed to connect to server: {e}"))
            })?;
        info!(server = %req.server, "WebSocket connection established");

        debug!("Sending request");
        timeout(
            Duration::from_secs(30),
            client.send_req(1, &ForwardedMsg { message: &req.message }),
        )
        .await
        .map_err(|_| {
            error!(server = %req.server, "Timed out sending request");
            CustomError::new(ErrorCode::BAD_REQUEST, "Timed out sending request to server")
        })??;
        debug!("Request sent, waiting for response");

        let raw = timeout(Duration::from_secs(30), client.recv_raw())
            .await
            .map_err(|_| {
                error!(server = %req.server, "Timed out waiting for response");
                CustomError::new(ErrorCode::BAD_REQUEST, "Timed out waiting for response from server")
            })?
            .map_err(|e| {
                error!(server = %req.server, err = %e, "Failed to receive response from server");
                CustomError::new(ErrorCode::BAD_REQUEST, format!("Failed to receive response from server: {e}"))
            })?;
        info!(server = %req.server, "Received response from server");

        Ok(SendMsgResponse {
            serverResponse: serde_json::to_string(&raw).unwrap_or_default(),
        })
    }
}
