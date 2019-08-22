#[macro_use]
extern crate clap;

use std::collections::HashSet;
use std::time::Duration;

use reqwest::{Client, Error, header::USER_AGENT, Response, StatusCode, Url};
use reqwest::r#async::{Client as AsyncClient, Response as AsyncResponse};
use select::document::Document;
use select::predicate::Name;

pub use cli::{get_matches_or_fail, make_app};

use crate::cli::RLINKS_USER_AGENT;
use crate::text::{print_error, print_response};
use crate::text::ColorsExt;


const TIMEOUT_SECONDS: u64 = 30;
mod cli;
mod text;

pub fn is_valid_status_code(x: StatusCode) -> bool {
    x.is_success() | x.is_redirection()
}

#[derive(PartialEq, Debug)]
pub enum RequestType {
    GET,
    HEAD,
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
    } else if x.starts_with('/') {
        Option::Some(format!("{}{}", fixed_url_string, &x[1..]))
    } else if x.starts_with("./") {
        Option::Some(format!("{}{}", fixed_url_string, &x[2..]))
    } else if x.starts_with("http") {
        Option::Some(x.to_owned())
    } else {
        Option::None
    }
}

pub fn get_client() -> AsyncClient {
    AsyncClient::builder()
        .danger_accept_invalid_certs(true)
        .cookie_store(true)
        .timeout(Duration::from_secs(TIMEOUT_SECONDS))
        .build()
        .unwrap()
}

fn get_sync_client() -> Client {
    Client::builder()
        .danger_accept_invalid_certs(true)
        .cookie_store(true)
        .timeout(Duration::from_secs(TIMEOUT_SECONDS))
        .build()
        .unwrap()
}

fn get_response(url: Url) -> Result<Response, Error> {
    get_sync_client()
        .get(url)
        .header(USER_AGENT, RLINKS_USER_AGENT)
        .send()
}

fn get_url_root(url: &Url) -> &str {
    url.host_str().unwrap()
}

pub fn get_links_for_website(url_string: String) -> Result<HashSet<String>, RustyLinksError> {
    let fixed_url = Url::parse(&add_http(&url_string));
    let fixed_url_string: String = match &fixed_url {
        Ok(e) => format!("http://{}/", get_url_root(e)),
        Err(_) => "".to_owned(),
    };
    println!("{}", fixed_url_string);
    let links = fixed_url.map(|url| {
        get_response(url)
            .map(move |mut doc| {
                if is_valid_status_code(doc.status()) {
                    Document::from(doc.text().unwrap().as_str())
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
    response: Result<AsyncResponse, Error>,
    show_ok: bool,
    method: RequestType,
) -> Result<(), ()> {
    match response {
        Ok(x) => {
            if is_valid_status_code(x.status()) {
                if show_ok {
                    print_response(x, method);
                }
                Ok(())
            } else {
                if method == RequestType::GET {
                    print_response(x, method);
                }
                Err(())
            }
        }
        Err(e) => {
            let err_msg = format!("{}", e);
            if err_msg.contains("Infinite redirect loop") {
                println!("{}", err_msg.bold_green());
                Ok(())
            } else {
                if method == RequestType::GET {
                    print_error(e);
                }
                Err(())
            }
        }
    }
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
