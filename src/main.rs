use std::backtrace::Backtrace;
use std::panic;

use clap::Parser;
use endpoint_libs::libs::log::{FileLoggingConfig, LoggingConfig, setup_logging};
use eyre::{Result, eyre};

use crate::app::App;
use crate::cli::CliArgument;
use crate::config::Config;

mod app;
mod cli;
mod codegen;
mod config;
mod db;
mod handlers;
mod service;
mod util;

fn main() -> Result<()> {
    panic::set_hook(Box::new(|info| {
        let bt = Backtrace::force_capture();
        eprintln!("{info}");
        eprintln!("{bt}");
    }));

    let args = CliArgument::try_parse()?;
    let config = ::config::Config::builder()
        .add_source(::config::File::with_name(&args.config).required(false))
        .build()?
        .try_deserialize::<Config>()?;

    let log_dir = args.log.unwrap_or_else(|| config.log.folder.clone());

    // _log_guard needs to exist throughout the lifetime of app to ensure file write access
    let _log_guard = setup_logging(LoggingConfig {
        level: config.log.level,
        file_config: Some(FileLoggingConfig {
            path: log_dir,
            file_prefix: None,
            file_log_level: Some(config.log.level),
            rotation: None,
        }),
    })?;

    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| eyre!("Failed to install rustls crypto provider"))?;

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(config.runtime.working_threads())
        .enable_io()
        .enable_time()
        .build()?
        .block_on(async {
            let app = App::new(config).await?;
            app.run().await
        })
}
