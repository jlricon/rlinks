#[macro_use]
extern crate clap;

use std::collections::HashSet;

use futures::{
    future::{self, Either, Future},
    stream, Stream,
};
use futures::sync::oneshot;
use reqwest::header::USER_AGENT;
use reqwest::r#async::Client;
use reqwest::r#async::Response;
use tokio;

use crate::cli::{get_matches_or_fail, make_app};
use crate::req::{
    get_client, get_links_for_website, handle_response, RequestType,
};

mod error;
mod req;
mod cli;
mod url_fix;
mod text;
fn request_with_header(
    client: Client,
    user_agent: &str,
    request_type: RequestType,
    url: &str,
) -> impl Future<Item = Response, Error = reqwest::Error> {
    match request_type {
        RequestType::HEAD => client.head(url),
        RequestType::GET => client.get(url),
    }
    .header(USER_AGENT, user_agent)
    .send()
}
fn make_request(
    url: String,
    show_ok: bool,
    user_agent: String,
) -> impl Future<Item = u32, Error = ()> {
    let client = get_client();
    request_with_header(client.clone(), &user_agent, RequestType::HEAD, &url).then(move |result| {
        if handle_response(result, show_ok, RequestType::HEAD).is_err() {
            Either::A(
                request_with_header(client, &user_agent, RequestType::GET, &url).then(
                    move |result| {
                        let num = match handle_response(result, show_ok, RequestType::GET) {
                            Ok(_) => 1,
                            Err(_) => 0,
                        };

                        Ok(num)
                    },
                ),
            )
        } else {
            Either::B(future::ok(1))
        }
    })
}
fn fetch(req: HashSet<String>, parallel_requests: usize, show_ok: bool, user_agent: String) {
    let (tx, rx) = oneshot::channel();
    let req_len = req.len();
    println!("Checking {} links for dead links...", req_len);
    let work = stream::iter_ok(req)
        .map(move |url| make_request(url, show_ok, user_agent.clone()))
        .buffer_unordered(parallel_requests)
        .fold(0, |count, res| Ok(count + res))
        .then(|result| {
            let _ = tx.send(result);
            Ok::<(), ()>(())
        });
    tokio::run(work);
    if let Ok(count) = rx.wait().unwrap() {
        println!("Got {}/{} valid links", count, req_len);
    } else {
        eprintln!("Error fetching links.");
    }
}

fn main() {
    let app = make_app();
    let config = match get_matches_or_fail(app.clone()) {
        Ok(c) => c,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };
    match app.get_matches().value_of("URL") {
        Some(e) => {
            get_links_for_website(e.to_owned())
                .map(|f| fetch(f, config.n_par, config.show_ok, config.user_agent))
                .map_err(|err| println!("{:?}", err))
                .unwrap();
        }
        // If there is no input argument
        _ => {
            make_app().print_help().unwrap();
            println!();
        }
    }
}
