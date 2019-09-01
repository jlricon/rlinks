use std::fmt::{Display, Error as FmtErr, Formatter};

use http::StatusCode;
use url::Url;

#[derive(Debug)]
pub enum RLinksError {
    UrlParseError(url::ParseError),
    RequestError(isahc::Error),
    ArgumentParsingError(clap::Error),
    StatusCodeError(StatusCode, Url),
}

impl From<url::ParseError> for RLinksError {
    fn from(err: url::ParseError) -> RLinksError {
        RLinksError::UrlParseError(err)
    }
}
impl From<isahc::Error> for RLinksError {
    fn from(err: isahc::Error) -> RLinksError {
        RLinksError::RequestError(err)
    }
}
impl From<clap::Error> for RLinksError {
    fn from(err: clap::Error) -> RLinksError {
        RLinksError::ArgumentParsingError(err)
    }
}
impl Display for RLinksError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtErr> {
        match self {
            RLinksError::RequestError(ref err) => err.fmt(f),
            RLinksError::UrlParseError(ref err) => err.fmt(f),
            RLinksError::ArgumentParsingError(ref err) => err.fmt(f),
            RLinksError::StatusCodeError(ref status, url) => f.write_str(&format!(
                "Could not reach {} (Status code: {})",
                url, status
            )),
        }
    }
}
