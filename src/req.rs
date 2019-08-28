use std::collections::HashSet;
use std::time::Duration;

use select::document::Document;
use select::predicate::Name;

use crate::cli::RLINKS_USER_AGENT;
use crate::error::RLinksError;
use crate::text::ColorsExt;
use crate::text::{print_error, print_response};
use crate::url_fix::{add_http, fix_malformed_url, get_url_root};
use reqwest::header::USER_AGENT;
use reqwest::r#async::Client;
use reqwest::r#async::Response;
use reqwest::{Error, StatusCode, Url};
use reqwest::{Response as SyncResponse,Client as SyncClient};

const TIMEOUT_SECONDS: u64 = 30;

pub fn is_valid_status_code(x: StatusCode) -> bool {
    x.is_success() | x.is_redirection()
}

#[derive(PartialEq, Debug)]
pub enum RequestType {
    GET,
    HEAD,
}

pub fn get_client() -> Client {
    Client::builder()
        .danger_accept_invalid_certs(true)
        .cookie_store(true)
        .timeout(Duration::from_secs(TIMEOUT_SECONDS))
        .build()
        .unwrap()
}
pub fn get_client_sync() -> SyncClient {
    SyncClient::builder()
        .danger_accept_invalid_certs(true)
        .cookie_store(true)
        .timeout(Duration::from_secs(TIMEOUT_SECONDS))
        .build()
        .unwrap()
}


fn get_sync_response(url: Url) -> Result<SyncResponse, RLinksError> {
    get_client_sync()
        .get(url)
        .header(USER_AGENT, RLINKS_USER_AGENT)
        .send().map_err(|e|e.into())
        
}

pub fn get_links_for_website(url_string: String) -> Result<HashSet<Url>, RLinksError> {
    let fixed_url = Url::parse(&add_http(&url_string));
    let fixed_url_string: String = match &fixed_url {
        Ok(e) => format!("http://{}/", get_url_root(e)),
        Err(_) => "".to_owned(),
    };
    let links = fixed_url.map(|url| {
        get_sync_response(url)
            .map(move |mut doc| {
                if is_valid_status_code(doc.status()) {
                    Document::from(doc.text().unwrap().as_str())
                        .find(Name("a"))
                        .filter_map(|n| n.attr("href"))
                        .filter_map(|x| match fix_malformed_url(x, &fixed_url_string) {
                            Ok(e) => Some(e),
                            Err(_) => None,
                        })
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
            Err(_) => Err(RLinksError::RequestError),
        },
        Err(e) => {
            println!("{:?}", e);
            Err(RLinksError::UrlParseError(e))
        }
    }
}

pub fn handle_response(
    response: Result<Response, Error>,
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
