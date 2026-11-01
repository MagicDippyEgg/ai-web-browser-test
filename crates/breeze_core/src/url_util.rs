use url::Url;

pub fn parse_url(raw: &str) -> Result<Url, url::ParseError> {
    if !raw.contains("://") && !raw.starts_with("data:") && !raw.starts_with("about:") {
        Url::parse(&format!("http://{}", raw))
    } else {
        Url::parse(raw)
    }
}

pub fn resolve_relative_url(base: &Url, relative: &str) -> Result<Url, url::ParseError> {
    base.join(relative)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        let u1 = parse_url("example.com").unwrap();
        assert_eq!(u1.scheme(), "http");
        assert_eq!(u1.host_str(), Some("example.com"));

        let u2 = parse_url("https://example.com/path?arg=1#frag").unwrap();
        assert_eq!(u2.scheme(), "https");
        assert_eq!(u2.path(), "/path");
    }

    #[test]
    fn test_resolve_relative() {
        let base = Url::parse("https://example.com/a/b.html").unwrap();
        let joined = resolve_relative_url(&base, "../style.css").unwrap();
        assert_eq!(joined.as_str(), "https://example.com/style.css");
    }
}
