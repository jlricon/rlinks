use futures::{
    future::{self, Either, Future},
    stream, Stream,
};

use futures::sync::oneshot;
use reqwest::header::USER_AGENT;
use reqwest::r#async::Client;
use rlinks::{
    get_client, get_links_for_website, handle_response, make_app, DEFAULT_PAR_REQ,
    RLINKS_USER_AGENT,
};
use std::collections::HashSet;
use tokio;

#[macro_use]
extern crate clap;
fn make_request(client: Client, url: String, show_ok: bool) -> impl Future<Item = u32, Error = ()> {
    client
        .head(&url)
        .header(USER_AGENT, RLINKS_USER_AGENT)
        .send()
        .then(move |result| {
            if handle_response(result, show_ok, "HEAD").is_err() {
                Either::A(
                    client
                        .get(&url)
                        .header(USER_AGENT, RLINKS_USER_AGENT)
                        .send()
                        .then(move |result| {
                            let num = match handle_response(result, show_ok, "GET") {
                                Ok(_) => 1,
                                Err(_) => 0,
                            };

                            Ok(num)
                        }),
                )
            } else {
                Either::B(future::ok(1))
            }
        })
}
fn fetch(req: HashSet<String>, parallel_requests: usize, show_ok: bool) {
    let client = get_client();
    let (tx, rx) = oneshot::channel();
    let req_len = req.len();
    println!("Checking {} links for dead links...", req_len);
    let work = stream::iter_ok(req)
        .map(move |url| make_request(client.clone(), url, show_ok))
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
    let app = make_app().get_matches();

    match app.value_of("URL") {
        Some(e) => {
            get_links_for_website(e.to_owned())
                .map(|f| {
                    fetch(
                        f,
                        value_t!(app.value_of("n_par"), usize).unwrap_or(DEFAULT_PAR_REQ),
                        app.is_present("show_ok"),
                    )
                })
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
