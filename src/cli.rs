use clap::{App, AppSettings, Arg, SubCommand};

use crate::error::RLinksError;
use regex::Regex;

const TIMEOUT_SECONDS: u64 = 10;
const DEFAULT_PAR_REQ: usize = 2;
const RLINKS_USER_AGENT: &str =
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
    pub ignore_urls: Option<Regex>,
}
#[derive(Debug)]
pub struct DumpConfig {
    pub url: String,
    pub user_agent: String,
    pub timeout: u64,
    pub output_file: String,
    pub ignore_urls: Option<Regex>,
}

pub fn get_config(app: App) -> Result<CommandConfig, RLinksError> {
    let matches = app.get_matches();
    let subcommand_matches = matches.subcommand().1.unwrap();
    let ignore_urls = subcommand_matches
        .value_of("ignore_urls")
        .map(|v| Regex::new(v).unwrap());
    let url = value_t!(subcommand_matches.value_of("URL"), String)?;
    let timeout = subcommand_matches
        .value_of("timeout")
        .map_or_else(|| TIMEOUT_SECONDS, |val| val.parse().unwrap());

    let user_agent = subcommand_matches
        .value_of("user_agent")
        .unwrap_or(&RLINKS_USER_AGENT)
        .to_owned();
    match matches.subcommand_name().unwrap() {
        "dump" => Ok(CommandConfig::Dump(DumpConfig {
            url,
            user_agent,
            timeout,
            output_file: value_t!(subcommand_matches.value_of("output"), String)?,
            ignore_urls,
        })),
        "check" => {
            let n_par = subcommand_matches
                .value_of("n_par")
                .map_or(DEFAULT_PAR_REQ, |v| v.parse().unwrap());

            Ok(CommandConfig::Base(BaseConfig {
                n_par,
                user_agent,
                show_ok: subcommand_matches.is_present("show_ok"),
                timeout,
                url,
                ignore_urls,
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
                )
                .arg(
                    Arg::with_name("ignore_urls")
                        .short("i")
                        .long("ignore_urls")
                        .takes_value(true)
                        .help("Ignores certain patterns. Uses a single regex expression"),
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
                )
                .arg(
                    Arg::with_name("ignore_urls")
                        .short("i")
                        .long("ignore_urls")
                        .takes_value(true)
                        .help("Ignores certain patterns. Uses a single regex expression"),
                ),
        )
}
