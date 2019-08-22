use clap::{App, Arg};

pub const DEFAULT_PAR_REQ: &str = "10";
pub const RLINKS_USER_AGENT: &str =
    "Mozilla/5.0 (compatible; Rlinks/0.3; +https://github.com/jlricon/rlinks/)";

pub struct Config {
    pub n_par: usize,
    pub user_agent: String,
    pub show_ok: bool,
}
pub fn get_matches_or_fail(app: App) -> Result<Config, clap::Error> {
    let matches = app.get_matches();
    let n_par = value_t!(matches.value_of("n_par"), usize)?;
    let user_agent = value_t!(matches.value_of("user_agent"), String)?;
    let show_ok = matches.is_present("show_ok");
    Ok(Config {
        n_par,
        user_agent,
        show_ok,
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
                .help("Number of parallel requests")
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
}
