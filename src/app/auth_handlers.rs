use eyre::eyre;
use std::sync::Arc;

use crate::codegen::model::{EnumEndpoint, InitRequest, InitResponse};
use crate::db::database::Db;

use endpoint_libs::libs::ws::{EndpointAuthController, WebsocketServer};
use honey_id_types::HoneyIdClient;
use honey_id_types::enums::HoneyEndpointMethodCode;
use honey_id_types::handlers::auth_to_app::{MethodApiKeyConnect, MethodReceiveToken, MethodReceiveUserInfo};
use honey_id_types::handlers::convenience_utils::generic_auth_handler::{
    AuthorizedConnectContext, AuthorizedConnectRequest, GenericAuthorizedConnect,
};

use honey_id_types::handlers::convenience_utils::token_management::TokenStorage;
use uuid::Uuid;

pub fn register_auth_handlers(
    server: &mut WebsocketServer,
    tables: Arc<Db>,
    token_storage: Arc<dyn TokenStorage + Sync + Send>,
    honey_id_client: Arc<HoneyIdClient>,
) {
    // TODO: later move this into separate fn in types repo
    let mut auth_controller = EndpointAuthController::default();

    let tables_clone = tables.clone();

    auth_controller.add_auth_endpoint(
        EnumEndpoint::Init.schema(),
        GenericAuthorizedConnect::<InitRequest, InitResponse>::new(
            token_storage.clone(),
            tables.user_table.clone(),
            move |_req, ctx| {
                let tables = tables_clone.clone();

                dashboard_init(ctx, tables)
            },
        ),
    );

    auth_controller.add_auth_endpoint(
        HoneyEndpointMethodCode::ApiKeyConnect.schema(),
        MethodApiKeyConnect {
            honey_id_client: honey_id_client.clone(),
            user_storage: tables.user_table.clone(),
        },
    );

    server.set_auth_controller(auth_controller);

    server.add_handler(MethodReceiveToken {
        token_storage: token_storage.clone(),
        user_storage: tables.user_table.clone(),
    });

    server.add_handler(MethodReceiveUserInfo {
        token_storage,
        user_storage: tables.user_table.clone(),
    });
}

async fn dashboard_init(ctx: AuthorizedConnectContext, tables: Arc<Db>) -> eyre::Result<InitResponse> {
    let user = tables
        .user_table
        .select_by_pub_id(Uuid::from(ctx.user_pub_id))
        .ok_or(eyre!(format!("Error fetching user with pub ID: {}", ctx.user_pub_id)))?;

    ctx.conn.set_user_id(user.id as u64);
    // The generic authorized connect handler already sets the user role on the conn before getting here

    Ok(InitResponse {
        user_public_id: user.pub_id,
        role: user.role,
    })
}

impl AuthorizedConnectRequest for InitRequest {
    fn get_access_token(&self) -> &str {
        &self.access_token
    }
}
