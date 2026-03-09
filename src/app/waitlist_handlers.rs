use endpoint_libs::libs::ws::WebsocketServer;

use crate::app::AppCtx;
use crate::handlers::waitlist::{add_lead::MethodAddLead, list_leads::MethodListLeads};

pub fn register_waitlist_handlers(server: &mut WebsocketServer, ctx: &AppCtx) {
    server.add_handler(MethodAddLead {
        waitlist_table: ctx.db.waitlist_table.clone(),
    });

    server.add_handler(MethodListLeads {
        waitlist_table: ctx.db.waitlist_table.clone(),
    });
}
