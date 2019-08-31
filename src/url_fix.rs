use url::Url;

use crate::error::RLinksError;

/// This adds http to a link if it does not start with http(s)
pub fn add_http(url_string: &str) -> Result<Url, RLinksError> {
    let fixed_url = if !(url_string.starts_with("http://") | url_string.starts_with("https://")) {
        format!("http://{}", url_string)
    } else {
        url_string.to_owned()
    };
    Ok(Url::parse(&fixed_url)?)
}
/// This fixes a relative link to a potential URL
pub fn fix_malformed_url(x: &str, base_url: &Url) -> Result<Url, RLinksError> {
    let fixed_url = if x.starts_with("//") {
        Result::Ok(format!("http://{}", &x[2..]))
    } else if x.starts_with('/') {
        Result::Ok(format!("{}{}", base_url.to_string(), &x[1..]))
    } else if x.starts_with("./") {
        Result::Ok(format!("{}{}", base_url.to_string(), &x[2..]))
    } else if x.starts_with("http") {
        Result::Ok(x.to_owned())
    } else if x.starts_with('#') {
        Result::Ok(format! {"{}{}",base_url.to_string(),&x})
    } else {
        Result::Err(RLinksError::MalformedUrl(x.to_string()))
    };
    match fixed_url {
        Ok(url) => Ok(Url::parse(&url)?),
        Err(e) => Err(e),
    }
}
//pub fn get_url_root(url: &Url) -> &str {
//    url.host_str().unwrap()
//}
#[cfg(test)]
mod tests {
    use url::Url;

    use crate::{
        error::RLinksError,
        url_fix::{add_http, fix_malformed_url},
    };

    #[test]
    fn test_add_http() {
        assert_eq!(
            add_http("http://test.com").unwrap(),
            Url::parse("http://test.com/").unwrap()
        );
        assert_eq!(
            add_http("https://test.com").unwrap(),
            Url::parse("https://test.com/").unwrap()
        );
        assert_eq!(
            add_http("test.com").unwrap(),
            Url::parse("http://test.com/").unwrap()
        );
        assert_eq!(
            add_http("www.test.com").unwrap(),
            Url::parse("http://www.test.com/").unwrap()
        );
    }

    #[test]
    fn test_fix_malformed_url() {
        let base_url = Url::parse("https://test.com/").unwrap();
        assert_eq!(
            fix_malformed_url("http://test.com", &base_url)
                .unwrap()
                .to_string(),
            "http://test.com/".to_owned()
        );
        assert_eq!(
            fix_malformed_url("//test2.com", &base_url)
                .unwrap()
                .to_string(),
            "http://test2.com/".to_owned()
        );
        assert_eq!(
            fix_malformed_url("/subsite", &base_url)
                .unwrap()
                .to_string(),
            "https://test.com/subsite".to_owned()
        );
        assert!(match (
            fix_malformed_url("blah", &base_url).unwrap_err(),
            RLinksError::MalformedUrl("blah".to_owned())
        ) {
            (RLinksError::MalformedUrl(e1), RLinksError::MalformedUrl(e2)) => e1 == e2,
            _ => false,
        });
    }
}
