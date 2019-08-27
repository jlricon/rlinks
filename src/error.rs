
#[derive(Debug)]
pub enum RLinksError {
    MalformedUrl(String),
    RequestError,
    UrlParseError(String)
}
