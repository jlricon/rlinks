use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
    time::Duration,
};

use crate::{error::RLinksError, text::ColorsExt, url_fix::fix_malformed_url};
use futures::{stream, StreamExt, TryFutureExt};
use http::{header::USER_AGENT, StatusCode};
use indicatif::{ProgressBar, ProgressStyle};
use isahc::{
    config::RedirectPolicy,
    prelude::{HttpClient, Request, Response},
    Body,
};
use select::{document::Document, predicate::Name};
use url::{Host, Url};

#[derive(Debug)]
enum StatusCodeKind {
    Valid(StatusCode),
    MethodNotAllowed(StatusCode),
    Fail(StatusCode),
}
fn get_status_code_kind(x: StatusCode) -> StatusCodeKind {
    match x {
        x if x.is_success() | x.is_redirection() => StatusCodeKind::Valid(x),
        x if x == StatusCode::METHOD_NOT_ALLOWED => StatusCodeKind::MethodNotAllowed(x),
        x => StatusCodeKind::Fail(x),
    }
}
#[derive(Debug)]
pub enum RequestType {
    GET,
    HEAD,
}
pub fn get_client(timeout: Duration) -> HttpClient {
    HttpClient::builder()
        .timeout(timeout)
        .redirect_policy(RedirectPolicy::Follow)
        //                        .cookies()
        .build()
        .unwrap()
}
// This generates a response with a timeout status so that we can make errors into response
fn build_fake_response(status: StatusCode) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(Body::empty())
        .unwrap()
}
async fn request_with_header(
    client: &HttpClient,
    user_agent: &str,
    request_type: RequestType,
    url: &Url,
) -> Result<Response<Body>, RLinksError> {
    let req = match request_type {
        RequestType::HEAD => Request::head(url.clone().into_string()),
        RequestType::GET => Request::get(url.clone().into_string()),
    }
    .header(USER_AGENT, user_agent)
    .body(())
    // This unwrap is safe, we are merely building the request
    .unwrap();
    match client
        .send_async(req)
        .map_err(RLinksError::RequestError)
        .await
    {
        Ok(e) => Ok(e),
        // Timeouts become errors, but we want to make these not error just yet, so we make them into fake responses
        Err(RLinksError::RequestError(isahc::Error::Timeout)) => {
            Ok(build_fake_response(StatusCode::REQUEST_TIMEOUT))
        }
        Err(RLinksError::RequestError(isahc::Error::CouldntResolveHost)) => {
            Ok(build_fake_response(StatusCode::NOT_FOUND))
        }
        Err(RLinksError::RequestError(isahc::Error::ConnectFailed)) => {
            Ok(build_fake_response(StatusCode::NOT_FOUND))
        }
        // This function should not error, so we panic
        Err(e) => {
            println!("{}", url);
            format!("Found unrecoverable error: {}", e).print_in_red();
            panic!(e)
        }
    }
}
type HostHashMap = HashMap<Host, HashSet<Url>>;
pub struct Links {
    pub hash_map: HostHashMap,
    pub link_count: u64,
}

/// Returns a hashmap mapping from root domains to all urls that are related to those domains
/// For example nintil.com :[nintil.com/a,nintil.com/b]
/// This is so that we can then turn each into streams and set individual rate limits
pub async fn get_links_from_website(
    client: &HttpClient,
    user_agent: &str,
    base_url: &Url,
) -> Result<Links, RLinksError> {
    let response = request_with_header(client, user_agent, RequestType::GET, base_url)
        .await
        .unwrap();

    match get_status_code_kind(response.status()) {
        StatusCodeKind::Valid(_) => (),
        _ => {
            return Err(RLinksError::StatusCodeError(
                response.status(),
                base_url.to_owned(),
            ))
        }
    }

    let all_links: Vec<Result<Url, RLinksError>> =
    // Unwrapping is safe here as the response has been validated
        Document::from(response.into_body().text().unwrap().as_str())
            .find(Name("a"))
            .filter_map(|n| n.attr("href"))
            .map(|r| fix_malformed_url(r, base_url))
            .collect();
    // We now split the urls by domain
    // TODO: Fix this to avoid having to do mutation. Because A E S T H E T I C S
    let mut hash_map: HashMap<Host, HashSet<Url>> = HashMap::new();
    // This valid list links can contain duplicates
    let valid_links: Vec<&Url> = all_links
        .iter()
        .filter_map(|url| match url {
            Err(e) => {
                println!("{}", e);
                None
            }
            Ok(url) => {
                // If there is no host, it's probably a fake link like javascript:void(0)
                if url.has_host() {
                    Some(url)
                } else {
                    None
                }
            }
        })
        .collect();
    let valid_links_len = valid_links.len();
    let unique_valid_links: HashSet<&Url> = HashSet::from_iter(valid_links);
    let unique_valid_links_len = unique_valid_links.len();
    println!(
        "Got {}/{} initial valid links from {} out of which {} are unique",
        valid_links_len,
        all_links.len(),
        base_url,
        unique_valid_links_len
    );
    // This unwrap is safe, every URL has a host
    unique_valid_links.into_iter().for_each(|url| {
        hash_map
            .entry(url.host().unwrap().to_owned())
            .or_insert_with(HashSet::new)
            .insert(url.to_owned());
    });
    //    let fake: HashSet<Url> = vec![Url::parse("https://www.understood.org/en/school-learning/learning-at-home/encouraging-reading-writing/6-strategies-to-teach-kids-self-regulation-in-writing").unwrap()]
    //        .into_iter()
    //        .collect();
    //    let mut fake2: HashMap<url::Host, HashSet<Url>> = HashMap::new();
    //    fake2.insert(url::Host::parse("o.com").unwrap(), fake);
    //    Ok(fake2)
    Ok(Links {
        hash_map,
        link_count: unique_valid_links_len as u64,
    })
}
/// Request a url trying with both HEAD and then GET
async fn is_reachable_url(
    client: &HttpClient,
    user_agent: &str,
    url: &Url,
    show_ok: bool,
    pbar: &ProgressBar,
) -> StatusCode {
    let response = request_with_header(client, user_agent, RequestType::HEAD, url)
        .await
        .unwrap();
    let r = match get_status_code_kind(response.status()) {
        StatusCodeKind::Valid(_) => Ok(response.status()),
        StatusCodeKind::MethodNotAllowed(_) => {
            match get_status_code_kind(
                request_with_header(client, user_agent, RequestType::GET, url)
                    .await
                    .unwrap()
                    .status(),
            ) {
                StatusCodeKind::Valid(e) => Ok(e),
                StatusCodeKind::Fail(e) | StatusCodeKind::MethodNotAllowed(e) => {
                    Err(RLinksError::StatusCodeError(e, url.to_owned()))
                }
            }
        }
        StatusCodeKind::Fail(e) => Err(RLinksError::StatusCodeError(e, url.to_owned())),
    }
    .map(|response| {
        if show_ok {
            pbar.println(
                format!("Success for {} ({})", url, response)
                    .bold_green()
                    .to_string(),
            );
        }
        response
    })
    .map_err(|err| {
        pbar.println(format!("{}", err).bold_red().to_string());
        err
    });
    match r {
        Ok(e) => e,
        // TODO: Find a better solution around this
        Err(_) => StatusCode::NOT_FOUND,
    }
}
type VectorOfResponses = Vec<StatusCode>;
/// Given a hashmap of domains:urls, make each set of urls into stream, then merge everything into
/// One big stream, introduce buffering per sub-stream to avoid hammering a domain with requests
pub async fn make_multiple_requests(
    links: Links,
    max_domain_concurrency: usize,
    client: &HttpClient,
    user_agent: &str,
    show_ok: bool,
) -> VectorOfResponses {
    let pbar = ProgressBar::new(links.link_count);
    pbar.enable_steady_tick(1000);
    pbar.set_style(
        ProgressStyle::default_bar()
            .template("ETA: [{eta_precise}] {bar:40} {pos:>7}/{len:7} {msg}"),
    );

    let stream_of_streams = links.hash_map.values().map(|values| {
        stream::iter(values.iter())
            .map(|url| {
                pbar.inc(1);
                is_reachable_url(client, user_agent, url, show_ok, &pbar)
            })
            .buffer_unordered(max_domain_concurrency)
    });
    let outp = stream::select_all(stream_of_streams).collect().await;
    pbar.finish();
    outp
    //    let urls:Vec<&Url>=hash_map.values().flatten().collect();
    //        stream::iter(urls.iter()).map(|url| is_reachable_url(client, user_agent, url, show_ok))
    //        .buffer_unordered(3).collect()
    //        .await
}
