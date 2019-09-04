use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
    time::Duration,
};

use crate::{error::RLinksError, text::ColorsExt, url_fix::fix_malformed_url};
use futures::{stream, StreamExt, TryFutureExt};
use http::{header::USER_AGENT, StatusCode, Version};
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
    debug!("Getting client");
    HttpClient::builder()
        .timeout(timeout)
        .connect_timeout(timeout)
        // HTTP2 sometimes has issues
        .preferred_http_version(Version::HTTP_11)
        .redirect_policy(RedirectPolicy::Limit(2))
        .danger_allow_unsafe_ssl(true)
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
    .body(Body::empty())
    // This unwrap is safe, we are merely building the request
    .unwrap();
    debug!("Requesting {}", url);
    match client
        .send_async(req)
        .map_err(RLinksError::RequestError)
        .await
    {
        Ok(e) => Ok(e),

        // Timeouts become errors, but we want to make these not error just yet, so we make them into fake responses
        Err(RLinksError::RequestError(isahc::Error::Timeout)) => {
            error!("[ERROR] Timeout for {}", url);
            Ok(build_fake_response(StatusCode::REQUEST_TIMEOUT))
        }
        Err(RLinksError::RequestError(isahc::Error::CouldntResolveHost)) => {
            error!("[ERROR] Could not resolve host for {}", url);
            Ok(build_fake_response(StatusCode::NOT_FOUND))
        }
        Err(RLinksError::RequestError(isahc::Error::ConnectFailed)) => {
            error!("[ERROR] Connection failed for {}", url);
            Ok(build_fake_response(StatusCode::NOT_FOUND))
        }
        Err(RLinksError::RequestError(isahc::Error::TooManyRedirects)) => {
            error!("[ERROR] Too many redirects for {}", url);
            Ok(build_fake_response(StatusCode::MISDIRECTED_REQUEST))
        }
        // This function should not error, so we panic
        Err(e) => {
            error!(
                "[ERROR] Found unrecoverable error: {} when accessing {}",
                e, url
            );
            panic!(e)
        }
    }
}
type HostHashMap = HashMap<Host, HashSet<Url>>;
#[derive(Debug)]
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
    truncate_fragments: bool,
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
    let body = Document::from(response.into_body().text().unwrap().as_str());
    let href_links = get_href_links(base_url, &body).into_iter();
    let img_links = get_img_links(base_url, &body).into_iter();
    let all_links = href_links
        .chain(img_links)
        .map(|result| {
            result.map(|mut url| {
                if truncate_fragments {
                    url.set_fragment(None)
                };
                url
            })
        })
        .collect::<Vec<Result<Url, RLinksError>>>();

    // We now split the urls by domain
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

    let hash_map = get_unique_link_hashmap(unique_valid_links);
    format!("Found {} domains", hash_map.len()).print_in_green();
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

fn get_href_links(base_url: &Url, body: &Document) -> Vec<Result<Url, RLinksError>> {
    // Unwrapping is safe here as the response has been validated
    body.find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .map(|r| fix_malformed_url(r, base_url))
        .collect()
}
fn get_img_links(base_url: &Url, body: &Document) -> Vec<Result<Url, RLinksError>> {
    // Unwrapping is safe here as the response has been validated
    body.find(Name("img"))
        .filter_map(|n| n.attr("src"))
        .map(|r| fix_malformed_url(r, base_url))
        .collect()
}

fn get_unique_link_hashmap(unique_valid_links: HashSet<&Url>) -> HostHashMap {
    let mut hash_map: HashMap<Host, HashSet<Url>> = HashMap::new();
    unique_valid_links.into_iter().for_each(|url| {
        hash_map
            .entry(url.host().unwrap().to_owned())
            .or_insert_with(HashSet::new)
            .insert(url.to_owned());
    });
    hash_map
}

/// Request a url trying with both HEAD and then GET
async fn is_reachable_url(
    client: &HttpClient,
    user_agent: &str,
    url: &Url,
    show_ok: bool,
    pbar: &ProgressBar,
) -> StatusCode {
    let status = request_with_header(client, user_agent, RequestType::HEAD, url)
        .await
        .unwrap()
        .status();
    let r = match get_status_code_kind(status) {
        StatusCodeKind::Valid(_) => Ok(status),
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
    pbar.inc(1);
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
    pbar.set_style(
        ProgressStyle::default_bar()
            .template("ETA: [{eta_precise}] {bar:40} {pos:>7}/{len:7} {msg}"),
    );

    pbar.enable_steady_tick(1000);
    let stream_of_streams = links.hash_map.values().map(|values| {
        stream::iter(values.iter())
            .map(|url| is_reachable_url(client, user_agent, url, show_ok, &pbar))
            .buffer_unordered(max_domain_concurrency)
    });
    let outp = stream::select_all(stream_of_streams).collect().await;
    pbar.finish();
    outp
    //    let urls: Vec<&Url> = links.hash_map.values().flatten().collect();
    //    stream::iter(urls.iter())
    //        .map(|url| is_reachable_url(client, user_agent, url, show_ok, &pbar))
    //        .buffer_unordered(5)
    //        .collect()
    //        .await
}
