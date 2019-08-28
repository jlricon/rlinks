use url::Url;

use crate::error::RLinksError;

pub fn add_http(url_string: &str) -> String {
    if !(url_string.starts_with("http://") | url_string.starts_with("https://")) {
        format!("http://{}", url_string)
    } else {
        url_string.to_owned()
    }
}

pub fn fix_malformed_url(x: &str, base_url: &str) -> Result<Url, RLinksError> {
    let fixed_url = if x.starts_with("//") {
        Result::Ok(format!("http://{}", &x[2..]))
    } else if x.starts_with('/') {
        Result::Ok(format!("{}{}", base_url, &x[1..]))
    } else if x.starts_with("./") {
        Result::Ok(format!("{}{}", base_url, &x[2..]))
    } else if x.starts_with("http") {
        Result::Ok(x.to_owned())
    } else {
        Result::Err(RLinksError::MalformedUrl(x.to_string()))
    };
    match fixed_url {
        Ok(url) => Ok(Url::parse(&url)?),
        Err(e) => Err(e),
    }
}
pub fn get_url_root(url: &Url) -> &str {
    url.host_str().unwrap()
}
#[cfg(test)]
mod tests {
    use crate::error::RLinksError;
    use crate::url_fix::{add_http, fix_malformed_url};

    #[test]
    fn test_add_http() {
        assert_eq!(add_http("http://test.com"), "http://test.com");
        assert_eq!(add_http("https://test.com"), "https://test.com");
        assert_eq!(add_http("test.com"), "http://test.com");
        assert_eq!(add_http("www.test.com"), "http://www.test.com");
    }

    #[test]
    fn test_fix_malformed_url() {
        let base_url = "https://test.com/";
        assert_eq!(
            fix_malformed_url("http://test.com", base_url)
                .unwrap()
                .to_string(),
            "http://test.com".to_owned()
        );
        assert_eq!(
            fix_malformed_url("//test2.com", base_url)
                .unwrap()
                .to_string(),
            "http://test2.com".to_owned()
        );
        assert_eq!(
            fix_malformed_url("/subsite", base_url).unwrap().to_string(),
            "https://test.com/subsite".to_owned()
        );
        assert_eq!(
            fix_malformed_url("blah", base_url).unwrap_err(),
            RLinksError::MalformedUrl("blah".to_owned())
        );
    }
}
