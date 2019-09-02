#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate env_logger;
use crate::{
    cli::{get_matches_or_fail, make_app, Config},
    error::RLinksError,
    req::{get_client, get_links_from_website, make_multiple_requests},
    url_fix::add_http,
};
use std::time::Duration;

mod cli;
mod error;
mod req;
mod text;
mod url_fix;

#[tokio::main]
async fn main() -> Result<(), RLinksError> {
    env_logger::init();
    let mut app = make_app();
    match get_matches_or_fail(app.clone()) {
        Err(e) => {
            println!("{}", e);
            app.print_help().unwrap();
            // Because print help ends up with a weird %
            println!();
        }
        Ok(config) => {
            check_links(config).await?;
        }
    }
    Ok(())
}

async fn check_links(config: Config) -> Result<(), RLinksError> {
    let client = get_client(Duration::from_secs(config.timeout));
    let url = add_http(&config.url)?;
    let links = get_links_from_website(&client, &config.user_agent, &url).await?;
    make_multiple_requests(
        links,
        config.n_par,
        &client,
        &config.user_agent,
        config.show_ok,
    )
    .await;
    Ok(())
}
