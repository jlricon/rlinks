use futures::{future, stream, Stream};
use reqwest::r#async::Client;
use rlinks::{print_error, print_response, get_links_for_website, make_app};
use std::collections::HashSet;
use std::sync::mpsc;
use tokio;
const PARALLEL_REQUESTS: usize = 1000;

fn fetch(req: HashSet<String>) {
    let client = Client::new();
    let (tx, rx) = mpsc::channel();
    let req_len = req.len();
    println!("Checking for dead links...");
    let work = stream::iter_ok(req)
        .map(move |url| client.get(&url).send())
        .buffer_unordered(PARALLEL_REQUESTS)
        .then(move |response| {
            let this_tx = tx.clone();
            match response {
                Ok(x) => {
                    print_response(x);

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

    match url {
        Some(e) => {
            get_links_for_website(e.to_owned())
                .map(|f| fetch(f))
                .map_err(|err| println!("{:?}", err));
        }
        // If there is no input argument
        _ => {
            make_app().print_help().unwrap();
            println!();
        }
    }
}
