use crate::{
    cli::DumpConfig,
    error::RLinksError,
    req::{get_client, get_links_from_website},
    url_fix::add_http,
};
use std::{collections::HashSet, fs::File, io::Write, path::Path, time::Duration};

pub async fn dump_links(config: DumpConfig) -> Result<(), RLinksError> {
    println!("{:?}", config);
    let client = get_client(Duration::from_secs(config.timeout));
    let url = add_http(&config.url)?;
    let links = get_links_from_website(
        &client,
        &config.user_agent,
        &url,
        false,
        &config.ignore_urls,
    )
    .await?;
    let all_links = links
        .hash_map
        .values()
        .fold(
            HashSet::with_capacity(links.link_count as usize),
            |mut acc, x| {
                x.iter().for_each(|url| {
                    acc.insert(url.as_str());
                });
                acc
            },
        )
        .into_iter()
        .collect::<Vec<&str>>()
        .join("\n");

    // Open a file in write-only mode, returns `io::Result<File>`
    write_to_file(&all_links, &config.output_file);
    Ok(())
}
fn write_to_file(string: &str, output_file: &str) {
    let path = Path::new(output_file);
    let display = path.display();
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    // Write the `LOREM_IPSUM` string to `file`, returns `io::Result<()>`
    match file.write_all(string.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }
}
