use std::sync::Arc;

use chrono::Utc;
use endpoint_libs::libs::handler::{RequestHandler, Response};
use endpoint_libs::libs::toolbox::RequestContext;
use eyre::eyre;

use crate::codegen::model::{AddLeadRequest, AddLeadResponse};
use crate::db::schema::waitlist::{WaitlistRow, WaitlistWorkTable};

#[derive(Debug)]
pub struct MethodAddLead {
    pub waitlist_table: Arc<WaitlistWorkTable>,
}

#[async_trait::async_trait(?Send)]
impl RequestHandler for MethodAddLead {
    type Request = AddLeadRequest;

    async fn handle(&self, _ctx: RequestContext, req: Self::Request) -> Response<Self::Request> {
        if req.telegram.is_none() && req.whatsApp.is_none() {
            return Err(eyre!("Telegram or WhatsApp must be specified"));
        }

        self.waitlist_table.insert(WaitlistRow {
            id: self.waitlist_table.get_next_pk().into(),
            name: req.name,
            telegram: req.telegram,
            whatsapp: req.whatsApp,
            description: req.description,
            created_at: Utc::now().timestamp(),
        })?;

        Ok(AddLeadResponse {})
    }
}
