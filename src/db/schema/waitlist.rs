use worktable::prelude::*;
use worktable::worktable;

worktable!(
    name: Waitlist,
    persist: true,
    columns: {
        id: i64 primary_key autoincrement,
        name: String,
        telegram: String optional,
        whatsapp: String optional,
        description: String,
        created_at: i64,
    }
);
