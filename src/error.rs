#[derive(Debug)]
pub enum RLinksError {
    MalformedUrl(String),
    RequestError,
    UrlParseError(url::ParseError),
    ReqwestError(reqwest::Error)

}

impl From<url::ParseError> for RLinksError {
    fn from(err: url::ParseError) -> RLinksError {
        RLinksError::UrlParseError(err)
    }
}
impl From<reqwest::Error> for RLinksError {
    fn from(err: reqwest::Error) -> RLinksError {
        RLinksError::ReqwestError(err)
    }
}

