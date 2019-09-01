use futures::{stream, StreamExt};
use isahc::{Error, HttpClient};

fn get_client() -> HttpClient {
    HttpClient::builder().cookies().build().unwrap()
}
async fn make_req(client: &HttpClient, url: &str) -> Result<(), Error> {
    client.get_async(url).await;
    Ok(())
}

#[tokio::main]
async fn main() {
    let client = get_client();
    let vec_reqs = vec!["https://google.com", "https://github.com"];
    let res: Vec<Result<(), Error>> = stream::iter(vec_reqs)
        .map(|r| make_req(&client, r))
        .buffer_unordered(1)
        .collect()
        .await;
}
