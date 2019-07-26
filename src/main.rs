use futures::{
    future::{Either, Future},
    stream, Stream,
};
use reqwest::r#async::Client;

use reqwest::StatusCode;
use rlinks::{get_links_for_website, handle_response, make_app, print_error, DEFAULT_PAR_REQ};
use std::collections::HashSet;
use std::sync::mpsc;
use tokio;

#[macro_use]
extern crate clap;

fn fetch(req: HashSet<String>, parallel_requests: usize, show_ok: bool) {
    let client = Client::new();
    let (tx, rx) = mpsc::channel();
    let req_len = req.len();
    println!("Checking {} links for dead links...", req_len);
    let work = stream::iter_ok(req)
        .map(move |url| {
            client.head(&url).send().and_then(move |f| {
                if f.status() == StatusCode::METHOD_NOT_ALLOWED {
                    let client2 = Client::new();
                    Either::A(client2.get(&url).send())
                } else {
                    Either::B(futures::future::ok(f))
                }
            })
        })
        .buffer_unordered(parallel_requests)
        .then(move |response| match response {
            Ok(r) => Either::A(handle_response(r, show_ok, tx.clone())),
            Err(r) => Either::B({
                print_error(r);
                futures::future::ok(())
            }),
        })
        .for_each(|_| Ok(()));

    tokio::run(work);
    println!("Got {}/{} valid links", rx.iter().sum::<u32>(), req_len);
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
