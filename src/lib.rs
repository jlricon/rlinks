#[macro_use]
extern crate clap;

use colored::{ColoredString, Colorize};
use reqwest::Response;
use reqwest::{Error, StatusCode, Url};

use clap::{App, Arg};
use futures::future;
use futures::future::FutureResult;
use select::document::Document;
use select::predicate::Name;
use std::collections::HashSet;
use std::fmt::Display;
use std::sync::mpsc::Sender;
pub const DEFAULT_PAR_REQ: usize = 20;

pub fn print_error<T: Display>(x: T) {
    let formatted_str = format!("{}", x).bold_red();
    println!("{}", formatted_str);
}
fn is_valid_status_code(x: StatusCode) -> bool {
    x.is_success() | x.is_redirection()
}
pub fn print_response(x: reqwest::r#async::Response) {
    if is_valid_status_code(x.status()) {
        let formatted_str =
            format!("{} is valid ({})", x.url().as_str(), x.status().as_str()).bold_green();
        println!("{}", formatted_str);
    } else {
        let formatted_str =
            format!("{} failed ({})", x.url().as_str(), x.status().as_str()).bold_red();
        println!("{}", formatted_str);
    }
}
pub trait ColorsExt {
    fn bold_red(&self) -> ColoredString;
    fn bold_green(&self) -> ColoredString;
}
impl ColorsExt for str {
    fn bold_red(self: &str) -> ColoredString {
        self.bold().red()
    }
    fn bold_green(self: &str) -> ColoredString {
        self.bold().green()
    }
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
                .help("Number of parallel requests (Default 20)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("show_ok")
                .short("s")
                .long("show_ok")
                .help("Show links that are ok"),
        )
}
#[derive(Debug)]
pub enum RustyLinksError {
    MalformedUrl,
    RequestError,
}
fn add_http(url_string: &str) -> String {
    if !(url_string.starts_with("http://") | url_string.starts_with("https://")) {
        ["http://", url_string].concat()
    } else {
        url_string.to_owned()
    }
}
fn fix_malformed_url(x: &str, fixed_url_string: &str) -> Option<String> {
    if x.starts_with("//") {
        Option::Some(format!("http://{}", &x[2..]))
    } else if x.starts_with("/") {
        Option::Some(format!("{}{}", fixed_url_string, &x[1..]))
    } else if x.starts_with("http") {
        Option::Some(x.to_owned())
    } else {
        Option::None
    }
}
fn get_response(url: Url) -> Result<Response, Error> {
    reqwest::get(url)
}
pub fn get_links_for_website(url_string: String) -> Result<HashSet<String>, RustyLinksError> {
    let fixed_url = Url::parse(&add_http(&url_string));
    let fixed_url_string = match &fixed_url {
        Ok(e) => e.as_str().to_owned(),
        Err(_) => "".to_owned(),
    };
    let links = fixed_url.map(|url| {
        get_response(url)
            .map(|doc| {
                if is_valid_status_code(doc.status()) {
                    Document::from_read(doc)
                        .unwrap()
                        .find(Name("a"))
                        .filter_map(|n| n.attr("href"))
                        .filter_map(|x| fix_malformed_url(x, &fixed_url_string))
                        .collect()
                } else {
                    let err = format!("Could not reach website {}: {}", url_string, doc.status());
                    print_error(err);
                    HashSet::new()
                }
            })
            .map_err(|e| println!("{:?}", e))
    });

    match links {
        Ok(e) => match e {
            Ok(e) => Ok(e),
            Err(_) => Err(RustyLinksError::RequestError),
        },
        Err(e) => {
            println!("{:?}", e);
            Err(RustyLinksError::MalformedUrl)
        }
    }
}
pub fn handle_response(
    response: Result<reqwest::r#async::Response, Error>,
    show_ok: bool,
    tx: Sender<u32>,
) -> FutureResult<(), ()> {
    match response {
        Ok(x) => {
            if show_ok {
                print_response(x);
            }

            tx.send(1).unwrap();
        }

        Err(x) => {
            print_error(x);
        }
    }

    future::ok(())
}

#[cfg(test)]
mod tests {
    use crate::{add_http, fix_malformed_url};

    #[test]
    fn test_add_http() {
        assert_eq!(add_http("http://test.com"), "http://test.com");
        assert_eq!(add_http("https://test.com"), "https://test.com");
        assert_eq!(add_http("test.com"), "http://test.com");
        assert_eq!(add_http("www.test.com"), "http://www.test.com");
    }
    #[test]
    fn test_fix_malformed_url() {
        let base_url = "https://test.com/";
        assert_eq!(
            fix_malformed_url("http://test.com", base_url),
            Option::Some("http://test.com".to_owned())
        );
        assert_eq!(
            fix_malformed_url("//test2.com", base_url),
            Option::Some("http://test2.com".to_owned())
        );
        assert_eq!(
            fix_malformed_url("/subsite", base_url),
            Option::Some("https://test.com/subsite".to_owned())
        );
        assert_eq!(fix_malformed_url("blah", base_url), Option::None);
    }
}
