use worktable::{prelude::*, worktable};

worktable!(
    name: TgBotConfig,
    persist: true,
    columns: {
        id: i64 primary_key autoincrement,
        enabled: bool,
        token: String optional,
    },
);
