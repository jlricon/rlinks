use colored::*;
use futures::{stream, future, Stream};
use reqwest::r#async::Client;
use std::sync::mpsc;
use tokio;
const PARALLEL_REQUESTS: usize = 100;

fn fetch(req: Vec<&'static str>) {
    let client = Client::new();

    let bodies = stream::iter_ok(req)
        .map(move |url| client.get(url).send())
        .buffer_unordered(PARALLEL_REQUESTS);

    let (tx, rx) = mpsc::channel();

    let work = bodies
        .inspect_err(|err| {
            let m = format!(
                "Error on {:?}",
                match err.url() {
                    Some(u) => u,
                    _ => panic!("This shouldn't happen"),
                }
            )
            .red()
            .bold();
            println!("{}", m);
        })
        .then(|r| future::ok(stream::iter_ok::<_, ()>(r)))
        .flatten()
        .for_each(move |n| {
            let this_tx = tx.clone();
            this_tx.send(1).unwrap();
            let m = format!("Success on {:?}", n.url()).green().bold();
            println!("{}", m);

            Ok(())
        });

    tokio::run(work);
    let valid_links: u32 = rx.iter().sum();
    println!("Got {} valid links", valid_links);
}
fn main() {
    let mut req = vec!["http://ricon.dev"; 3];
    req.push("http://a323232.com");
    fetch(req);
}
