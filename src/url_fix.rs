use reqwest::Url;

pub fn add_http(url_string: &str) -> String {
    if !(url_string.starts_with("http://") | url_string.starts_with("https://")) {
        format!("http://{}",url_string)
    } else {
        url_string.to_owned()
    }
}

pub fn fix_malformed_url(x: &str, base_url: &str) -> Option<String> {
    if x.starts_with("//") {
        Option::Some(format!("http://{}", &x[2..]))
    } else if x.starts_with('/') {
        Option::Some(format!("{}{}", base_url, &x[1..]))
    } else if x.starts_with("./") {
        Option::Some(format!("{}{}", base_url, &x[2..]))
    } else if x.starts_with("http") {
        Option::Some(x.to_owned())
    } else {
        Option::None
    }
}
pub fn get_url_root(url: &Url) -> &str {
    url.host_str().unwrap()
}
#[cfg(test)]
mod tests {
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
            fix_malformed_url("http://test.com", base_url),
            Option::Some("http://test.com".to_owned())
        );
        assert_eq!(
            fix_malformed_url("//test2.com", base_url),
            Option::Some("http://test2.com".to_owned())
        );
        assert_eq!(
            fix_malformed_url("/subsite", base_url),
            Option::Some("https://test.com/subsite".to_owned())
        );
        assert_eq!(fix_malformed_url("blah", base_url), Option::None);
    }
}
