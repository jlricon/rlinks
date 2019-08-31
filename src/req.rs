use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};
use std::iter::FromIterator;

use futures::{stream, StreamExt, TryFutureExt};
use http::{header::USER_AGENT, StatusCode};
use isahc::{
    Body,
    config::RedirectPolicy,
    prelude::{HttpClient, Request, Response},
};
use select::{document::Document, predicate::Name};
use url::{Host, Url};

use crate::{error::RLinksError, text::ColorsExt, url_fix::fix_malformed_url};

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

pub enum RequestType {
    GET,
    HEAD,
}
pub fn get_client(timeout: Duration) -> HttpClient {
    HttpClient::builder()
        .timeout(timeout)
        .redirect_policy(RedirectPolicy::Follow)
        //                .cookies()
        .build()
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
    client
        .send_async(req)
        .map_err(RLinksError::RequestError)
        .await
}
type HostHashMap = HashMap<Host, HashSet<Url>>;
/// Returns a hashmap mapping from root domains to all urls that are related to those domains
/// For example nintil.com :[nintil.com/a,nintil.com/b]
/// This is so that we can then turn each into streams and set individual rate limits
pub async fn get_links_from_website(
    client: &HttpClient,
    user_agent: &str,
    base_url: &Url,
) -> Result<HostHashMap, RLinksError> {
    let response = request_with_header(client, user_agent, RequestType::GET, base_url).await?;

    match get_status_code_kind(response.status()) {
        StatusCodeKind::Valid(_) => (),
        _ => return Err(RLinksError::StatusCodeError(response.status())),
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
            Ok(url) => Some(url),
        })
        .collect();
    let valid_links_len = valid_links.len();
    let unique_valid_links: HashSet<&Url> = HashSet::from_iter(valid_links);
    println!(
        "Got {}/{} initial valid links from {} out of which {} are unique",
        valid_links_len,
        all_links.len(),
        base_url,
        unique_valid_links.len()
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
    Ok(hash_map)
}
/// Request a url trying with both HEAD and then GET
async fn is_reachable_url(
    client: &HttpClient,
    user_agent: &str,
    url: &Url,
) -> Result<StatusCode, RLinksError> {
    let response = request_with_header(client, user_agent, RequestType::HEAD, url).await?;
    match get_status_code_kind(response.status()) {
        StatusCodeKind::Valid(_) => Ok(response.status()),
        StatusCodeKind::MethodNotAllowed(_) => {
            match get_status_code_kind(
                request_with_header(client, user_agent, RequestType::GET, url)
                    .await?
                    .status(),
            ) {
                StatusCodeKind::Valid(e) => Ok(e),
                StatusCodeKind::Fail(e) | StatusCodeKind::MethodNotAllowed(e) => {
                    Err(RLinksError::StatusCodeError(e))
                }
            }
        }
        StatusCodeKind::Fail(e) => Err(RLinksError::StatusCodeError(e)),
    }
    .map(|response| {
        format!("Success for {} ({})", url, response).print_in_green();
        response
    })
    .map_err(|err| {
        format!("Failure for {} ({})", url, err).print_in_red();
        err
    })
}
type VectorOfResponses = Vec<Result<StatusCode, RLinksError>>;
/// Given a hashmap of domains:urls, make each set of urls into stream, then merge everything into
/// One big stream, introduce buffering per sub-stream to avoid hammering a domain with requests
pub async fn make_multiple_requests(
    hash_map: HostHashMap,
    max_domain_concurrency: usize,
    client: &HttpClient,
    user_agent: &str,
) -> VectorOfResponses {
    let stream_of_streams = hash_map.values().into_iter().map(|values| {
        stream::iter(values.iter())
            .map(|url| {
                is_reachable_url(client, user_agent, url)
                //                    .map_err(|err| futures::future::ok(Response::new(Body::empty())))
            })
            .buffer_unordered(max_domain_concurrency)
    });
    stream::select_all(stream_of_streams).collect().await
}
