use std::sync::Arc;

use endpoint_libs::libs::handler::{RequestHandler, Response};
use endpoint_libs::libs::toolbox::RequestContext;
use worktable::select::SelectQueryExecutor;

use crate::codegen::model::{ListLeadsRequest, ListLeadsResponse, WaitlistLead};
use crate::db::schema::waitlist::WaitlistWorkTable;

#[derive(Debug)]
pub struct MethodListLeads {
    pub waitlist_table: Arc<WaitlistWorkTable>,
}

#[async_trait::async_trait(?Send)]
impl RequestHandler for MethodListLeads {
    type Request = ListLeadsRequest;

    async fn handle(&self, _ctx: RequestContext, _req: Self::Request) -> Response<Self::Request> {
        let data = self
            .waitlist_table
            .select_all()
            .execute()?
            .into_iter()
            .map(|row| WaitlistLead {
                id: row.id,
                name: row.name,
                telegram: row.telegram,
                whatsApp: row.whatsapp,
                description: row.description,
            })
            .collect();

        Ok(ListLeadsResponse { data })
    }
}
