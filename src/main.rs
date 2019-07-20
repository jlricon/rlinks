use futures::{future, stream, Stream};
use reqwest::r#async::Client;
use rlinks::{get_links_for_website, make_app, print_error, print_response};
use std::collections::HashSet;
use std::sync::mpsc;
use tokio;
const DEFAULT_PAR_REQ: usize = 100;
#[macro_use]
extern crate clap;
fn fetch(req: HashSet<String>, parallel_requests: usize, show_ok: bool) {
    let client = Client::new();
    let (tx, rx) = mpsc::channel();
    let req_len = req.len();
    println!("Checking for dead links...");
    let work = stream::iter_ok(req)
        .map(move |url| client.get(&url).send())
        .buffer_unordered(parallel_requests)
        .then(move |response| {
            let this_tx = tx.clone();
            match response {
                Ok(x) => {
                    if show_ok {
                        print_response(x);
                    }

                    this_tx.send(1).unwrap();
                }

                Err(x) => {
                    print_error(x);
                }
            }

            future::ok(())
        })
        .for_each(|_| Ok(()));

    tokio::run(work);
    let valid_links: u32 = rx.iter().sum();
    println!("Got {}/{} valid links", valid_links, req_len);
}

fn main() {
    let app = make_app().get_matches();

    let url = app.value_of("INPUT");
    let parallel_req = value_t!(app.value_of("n_par"), usize).unwrap_or(DEFAULT_PAR_REQ);
    let show_ok = value_t!(app.value_of("show_ok"), bool).unwrap_or(false);

    match url {
        Some(e) => {
            get_links_for_website(e.to_owned())
                .map(|f| fetch(f, parallel_req, show_ok))
                .map_err(|err| println!("{:?}", err));
        }
        // If there is no input argument
        _ => {
            make_app().print_help().unwrap();
            println!();
        }
    }
}
