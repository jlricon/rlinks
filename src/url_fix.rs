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
    // Links that have fragments can be treated as the same link, as they don't affect checking
    match base_url.join(x) {
        Ok(mut url) => {
            url.set_fragment(None);
            Ok(url)
        }
        // If there is a fragment, skip
        Err(e) => Err(RLinksError::UrlParseError(e)),
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use crate::url_fix::{add_http, fix_malformed_url};

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
            "https://test2.com/".to_owned()
        );
        assert_eq!(
            fix_malformed_url("/subsite", &base_url)
                .unwrap()
                .to_string(),
            "https://test.com/subsite".to_owned()
        );
        assert_eq!(
            fix_malformed_url(
                "/wiki/Phoney_War",
                &Url::parse("https://en.wikipedia.org/wiki/World_War_II").unwrap()
            )
            .unwrap()
            .to_string(),
            "https://en.wikipedia.org/wiki/Phoney_War"
        );
    }
}
