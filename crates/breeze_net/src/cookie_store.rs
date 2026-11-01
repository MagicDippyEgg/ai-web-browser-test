use cookie::Cookie;
use std::collections::HashMap;
use url::Url;

#[derive(Debug, Default, Clone)]
pub struct CookieStore {
    cookies: HashMap<String, HashMap<String, HashMap<String, String>>>,
}

impl CookieStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_cookie_header(&mut self, url: &Url, header_val: &str) {
        if let Ok(c) = Cookie::parse(header_val) {
            let host = c.domain()
                .map(|d| d.trim_start_matches('.').to_string())
                .or_else(|| url.host_str().map(|s| s.to_string()))
                .unwrap_or_default();

            let path = c.path().unwrap_or("/").to_string();
            let name = c.name().to_string();
            let value = c.value().to_string();

            self.cookies
                .entry(host)
                .or_default()
                .entry(path)
                .or_default()
                .insert(name, value);
        }
    }

    pub fn get_cookie_header(&self, url: &Url) -> Option<String> {
        let host = url.host_str()?;
        let path = url.path();

        let mut matched = Vec::new();
        for (domain, paths) in &self.cookies {
            if host == domain || host.ends_with(&format!(".{}", domain)) {
                for (p, map) in paths {
                    if path.starts_with(p) {
                        for (k, v) in map {
                            matched.push(format!("{}={}", k, v));
                        }
                    }
                }
            }
        }

        if matched.is_empty() {
            None
        } else {
            Some(matched.join("; "))
        }
    }
}
