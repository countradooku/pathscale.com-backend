use endpoint_libs::libs::ws::WebsocketServer;

use crate::app::AppCtx;
use crate::handlers::admin::add_supports::MethodAddSupports;
use crate::handlers::admin::get_tg_bot_config::MethodGetTgBotConfig;
use crate::handlers::admin::list_supports::MethodListSupports;
use crate::handlers::admin::list_users::MethodListUsers;
use crate::handlers::admin::remove_supports::MethodRemoveSupports;
use crate::handlers::admin::set_role::MethodSetRole;
use crate::handlers::admin::set_tg_bot_config::MethodSetTgBotConfig;

pub fn register_admin_handlers(server: &mut WebsocketServer, ctx: &AppCtx) {
    server.add_handler(MethodListUsers {
        user_table: ctx.db.user_table.clone(),
    });
    server.add_handler(MethodSetRole {
        user_table: ctx.db.user_table.clone(),
    });
    server.add_handler(MethodAddSupports {
        support_user_table: ctx.db.support_user_table.clone(),
    });

    server.add_handler(MethodRemoveSupports {
        support_user_table: ctx.db.support_user_table.clone(),
    });

    server.add_handler(MethodListSupports {
        support_user_table: ctx.db.support_user_table.clone(),
    });

    server.add_handler(MethodSetTgBotConfig {
        config_table: ctx.db.tg_bot_config_table.clone(),
        support_chat_manager: ctx.support_chat_manager.clone(),
        tg_bot_task: ctx.tg_bot_task.clone(),
        support_user_table: ctx.db.support_user_table.clone(),
        support_msg_table: ctx.db.support_msg_table.clone(),
    });

    server.add_handler(MethodGetTgBotConfig {
        config_table: ctx.db.tg_bot_config_table.clone(),
    });
}
