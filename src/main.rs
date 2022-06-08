#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;

use crate::{
    cli::{get_config, make_app, CommandConfig},
    error::RLinksError,
};
use clap::App;

mod cli;
mod commands;
mod error;
mod req;
mod text;
mod url_fix;

async fn run_app<'a, 'b>(app: App<'a, 'b>) -> Result<(), RLinksError> {
    match get_config(app) {
        Err(e) => Err(e),
        Ok(CommandConfig::Base(config)) => commands::check::check_links(config).await,

        Ok(CommandConfig::Dump(config)) => commands::dump::dump_links(config).await,
    }
}

fn main() {
    env_logger::init();
    let app = make_app();
    let result = futures::executor::block_on(run_app(app));
    if let Err(e) = result {
        println!("{}", e);
    }
}
