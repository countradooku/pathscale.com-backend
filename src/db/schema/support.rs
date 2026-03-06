use uuid::Uuid;
use worktable::{prelude::*, worktable};

worktable!(
    name: SupportUser,
    persist: true,
    columns: {
        handle: String primary_key,
        chat_id: i64 optional,
    },
    indexes: {
        // chat id will be unique when not None
        // wt would panic if this was `unique`
        chat_id_idx: chat_id,
    }
);

worktable!(
    name: SupportMessage,
    persist: true,
    columns: {
        id: i64 primary_key autoincrement,
        //true if message is from support,
        //false if message is from nf user
        incoming: bool,
        sent_by: String,
        user_pub_id: Uuid,
        chat_id: i64,
        sent_at: i64,
        content: String,
    },
    indexes: {
        user_pub_id_idx: user_pub_id,
    }
);
