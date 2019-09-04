use clap::{App, AppSettings, Arg, SubCommand};

use crate::error::RLinksError;

const TIMEOUT_SECONDS: u64 = 10;
pub const DEFAULT_PAR_REQ: usize = 4;
pub const RLINKS_USER_AGENT: &str =
    "Mozilla/5.0 (compatible; Rlinks/0.5; +https://github.com/jlricon/rlinks/)";
#[derive(Debug)]
pub enum CommandConfig {
    Base(BaseConfig),
    Dump(DumpConfig),
}
#[derive(Debug)]
pub struct BaseConfig {
    pub n_par: usize,
    pub user_agent: String,
    pub show_ok: bool,
    pub timeout: u64,
    pub url: String,
}
#[derive(Debug)]
pub struct DumpConfig {
    pub url: String,
    pub user_agent: String,
    pub timeout: u64,
    pub output_file: String,
}

pub fn get_config(app: App) -> Result<CommandConfig, RLinksError> {
    match app.get_matches().subcommand() {
        ("dump", Some(matches)) => Ok(CommandConfig::Dump(DumpConfig {
            url: value_t!(matches.value_of("URL"), String)?,
            user_agent: matches
                .value_of("user_agent")
                .unwrap_or(RLINKS_USER_AGENT)
                .to_string(),
            timeout: matches
                .value_of("timeout")
                .map_or_else(|| TIMEOUT_SECONDS, |val| val.parse().unwrap()),
            output_file: value_t!(matches.value_of("output"), String)?,
        })),
        ("check", Some(matches)) => {
            let n_par = matches
                .value_of("n_par")
                .map_or_else(|| DEFAULT_PAR_REQ, |v| v.parse().unwrap());
            let user_agent = matches
                .value_of("user_agent")
                .unwrap_or(&RLINKS_USER_AGENT)
                .to_owned();
            let show_ok = matches.is_present("show_ok");
            let timeout = matches
                .value_of("timeout")
                .map_or_else(|| TIMEOUT_SECONDS, |val| val.parse().unwrap());
            let url = value_t!(matches.value_of("URL"), String)?;
            Ok(CommandConfig::Base(BaseConfig {
                n_par,
                user_agent,
                show_ok,
                timeout,
                url,
            }))
        }
        _ => unreachable!(),
    }
}

pub fn make_app<'a, 'b>() -> App<'a, 'b> {
    App::new("Rusty Links")
        .version(crate_version!())
        .author("Jose Luis Ricon <jose@ricon.xyz>")
        .about("RLinks finds dead links in websites, or dumps scraped links to a file")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("check")
                .about("Checks links")
                .setting(AppSettings::ArgRequiredElseHelp)
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
                        .help("Number of parallel requests per domain")
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
                        .help("Choose your own custom user agent string"),
                )
                .arg(
                    Arg::with_name("timeout")
                        .short("t")
                        .long("timeout")
                        .takes_value(true)
                        .help("Request timeout"),
                ),
        )
        .subcommand(
            SubCommand::with_name("dump")
                .about("Dump links")
                .setting(AppSettings::ArgRequiredElseHelp)
                .arg(
                    Arg::with_name("URL")
                        .index(1)
                        .help("URL to check links for (e.g. http://www.google.com)"),
                )
                .arg(
                    Arg::with_name("output")
                        .short("o")
                        .long("output")
                        .value_name("OUTPUT_FILE")
                        .help("File to write the links to"),
                )
                .arg(
                    Arg::with_name("user_agent")
                        .short("u")
                        .long("user_agent")
                        .takes_value(true)
                        .help("Choose your own custom user agent string"),
                )
                .arg(
                    Arg::with_name("timeout")
                        .short("t")
                        .long("timeout")
                        .takes_value(true)
                        .help("Request timeout"),
                ),
        )
}
