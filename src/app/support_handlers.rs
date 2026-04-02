use endpoint_libs::libs::ws::WebsocketServer;
use uuid::Uuid;

use crate::app::AppCtx;
use crate::codegen::model::SubMsgEventsResponse;
use crate::handlers::support::list_msgs::MethodListMsgs;
use crate::handlers::support::send_msg::MethodSendMsg;
use crate::handlers::support::sub_msg_events::MethodSubMsgEvents;
use crate::handlers::utils::SubscriptionRouter;

pub fn register_support_handlers(server: &mut WebsocketServer, ctx: &AppCtx) {
    server.add_handler(MethodSendMsg {
        tg_bot_service: ctx.tg_bot_service.clone(),
        user_table: ctx.db.user_table.clone(),
    });

    let Some(chat_manager) = ctx.tg_bot_service.current_manager() else {
        return;
    };

    // Get stream from service (service owns the connector)
    let event_stream = chat_manager.event_stream();

    // Create router - it owns SubscriptionManager internally
    let router: SubscriptionRouter<Uuid, SubMsgEventsResponse> = SubscriptionRouter::new(
        0, // stream_code for SubscriptionManager
        event_stream,
        server.toolbox.clone(),
    );

    server.add_handler(MethodListMsgs {
        chat_manager: chat_manager.clone(),
        user_table: ctx.db.user_table.clone(),
    });

    // Pass router to subscription handler (router owns event_manager)
    server.add_handler(MethodSubMsgEvents {
        router,
        user_table: ctx.db.user_table.clone(),
    });
}
