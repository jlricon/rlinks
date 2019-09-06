use std::fmt::{Display, Error as FmtErr, Formatter};

use http::StatusCode;
use url::Url;
#[derive(Debug)]
pub enum RLinksError {
    UrlParseError(url::ParseError),
    RequestError(isahc::Error),
    ArgumentParsingError(clap::Error),
    StatusCodeError(StatusCode, Url),
    IgnoredPattern(String, String),
    RegexParsingError(regex::Error),
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

impl From<regex::Error> for RLinksError {
    fn from(err: regex::Error) -> RLinksError {
        RLinksError::RegexParsingError(err)
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
            RLinksError::IgnoredPattern(url, pattern) => f.write_str(&format!(
                "Ignored url {} because it matches {}",
                url, pattern
            )),
            RLinksError::RegexParsingError(ref err) => err.fmt(f),
        }
    }
}
