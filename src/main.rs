#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;

use crate::{
    cli::{get_config, make_app, CommandConfig},
    error::RLinksError,
};

mod cli;
mod commands;
mod error;
mod req;
mod text;
mod url_fix;
#[tokio::main]
async fn main() -> Result<(), RLinksError> {
    env_logger::init();
    let app = make_app();
    match get_config(app) {
        Err(e) => {
            println!("{}", e);
        }
        Ok(CommandConfig::Base(config)) => {
            commands::check::check_links(config).await?;
        }
        Ok(CommandConfig::Dump(config)) => {
            commands::dump::dump_links(config).await?;
        }
    }
    Ok(())
}
