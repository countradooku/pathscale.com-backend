use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct CliArgument {
    /// The path to config file
    #[clap(short, long, value_parser, value_name = "FILE", env = "CONFIG")]
    pub config: Box<str>,
    /// the location to read the log file
    pub log: Option<PathBuf>,
}
