use clap::{App, Arg};

use crate::error::RLinksError;

const TIMEOUT_SECONDS: &str = "10";
pub const DEFAULT_PAR_REQ: &str = "10";
pub const RLINKS_USER_AGENT: &str =
    "Mozilla/5.0 (compatible; Rlinks/0.5; +https://github.com/jlricon/rlinks/)";
#[derive(Debug)]
pub struct Config {
    pub n_par: usize,
    pub user_agent: String,
    pub show_ok: bool,
    pub timeout: u64,
    pub url: String,
}
pub fn get_matches_or_fail(app: App) -> Result<Config, RLinksError> {
    let matches = app.get_matches();
    let n_par = value_t!(matches.value_of("n_par"), usize)?;
    let user_agent = value_t!(matches.value_of("user_agent"), String)?;
    let show_ok = matches.is_present("show_ok");
    let timeout = value_t!(matches.value_of("timeout"), u64)?;
    let url = value_t!(matches.value_of("URL"), String)?;
    Ok(Config {
        n_par,
        user_agent,
        show_ok,
        timeout,
        url,
    })
}
pub fn make_app<'a, 'b>() -> App<'a, 'b> {
    App::new("Rusty Links")
        .version(crate_version!())
        .author("Jose Luis Ricon <jose@ricon.xyz>")
        .about("Finds dead links in websites")
        .arg(
            Arg::with_name("URL")
                .index(1)
                .help("URL to check links for (e.g. http://www.google.com)"),
        )
        .arg(
            Arg::with_name("n_par")
                .short("p")
                .long("n_par")
                .value_name("N_PAR")
                // Keep this in sync with DEFAULT_PAR_REQ
                .help("Number of parallel requests per domain")
                .default_value(DEFAULT_PAR_REQ)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("show_ok")
                .short("s")
                .long("show_ok")
                .help("Show links that are ok"),
        )
        .arg(
            Arg::with_name("user_agent")
                .short("u")
                .long("user_agent")
                .takes_value(true)
                .help("Choose your own custom user agent string")
                .default_value(RLINKS_USER_AGENT),
        )
        .arg(
            Arg::with_name("timeout")
                .short("t")
                .long("timeout")
                .takes_value(true)
                .help("Request timeout")
                .default_value(TIMEOUT_SECONDS),
        )
}
