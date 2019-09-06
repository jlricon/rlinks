use crate::{
    cli::BaseConfig,
    error::RLinksError,
    req::{get_client, get_links_from_website, make_multiple_requests},
    url_fix::add_http,
};
use std::time::Duration;

pub async fn check_links(config: BaseConfig) -> Result<(), RLinksError> {
    let client = get_client(Duration::from_secs(config.timeout));
    let url = add_http(&config.url)?;
    let links =
        get_links_from_website(&client, &config.user_agent, &url, true, &config.ignore_urls)
            .await?;
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
