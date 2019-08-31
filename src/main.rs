#[macro_use]
extern crate clap;

use std::time::Duration;

use crate::{
    cli::{Config, get_matches_or_fail, make_app},
    error::RLinksError,
    req::{get_client, get_links_from_website, make_multiple_requests},
    url_fix::add_http,
};

mod cli;
mod error;
mod req;
mod text;
mod url_fix;

#[tokio::main]
async fn main() -> Result<(), RLinksError> {
    let mut app = make_app();
    match get_matches_or_fail(app.clone()) {
        Err(e) => {
            println!("{}", e);
            app.print_help().unwrap()
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
    make_multiple_requests(links, config.n_par, &client, &config.user_agent).await;
    Ok(())
}
