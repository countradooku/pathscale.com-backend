use std::path::PathBuf;

use endpoint_libs::libs::log::LogLevel;
use endpoint_libs::libs::ws::WsServerConfig;
use honey_id_types::HoneyIdConfig;
use serde::Deserialize;
use smart_default::SmartDefault;
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub runtime: RuntimeConfig,
    pub server: WsServerConfig,
    pub log: LogConfig,
    pub database: DatabaseConfig,
    pub honey_id: HoneyIdConfig,
    pub tg_bot: TgBotConfig,
    #[serde(default)]
    pub user: Option<UserConfig>,
}

#[derive(Clone, Debug, Deserialize, SmartDefault)]
pub struct RuntimeConfig {
    #[default(4)]
    pub threads: usize,
    #[default(1.0)]
    pub tasks_ratio: f64,
}

impl RuntimeConfig {
    pub fn working_threads(&self) -> usize {
        (self.threads as f64 * self.tasks_ratio).floor() as usize
    }
}

#[derive(Debug, Clone, Deserialize, SmartDefault)]
#[serde(default)]
pub struct LogConfig {
    #[default(LogLevel::Info)]
    pub level: LogLevel,
    #[default("logs/".into())]
    pub folder: PathBuf,
}

#[derive(Debug, Clone, Deserialize, SmartDefault)]
#[serde(default)]
pub struct DatabaseConfig {
    #[default("data/".into())]
    pub path: PathBuf,
}

#[derive(Clone, Debug, Deserialize, SmartDefault)]
pub struct TgBotConfig {
    pub token: String,
}

#[derive(Clone, Debug, Deserialize, SmartDefault)]
pub struct UserConfig {
    pub admin_pub_id: Uuid,
}
