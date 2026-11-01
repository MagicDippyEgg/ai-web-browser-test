use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Origin {
    pub scheme: String,
    pub host: String,
    pub port: u16,
}

impl Origin {
    pub fn from_url(url: &Url) -> Option<Self> {
        let scheme = url.scheme().to_lowercase();
        let host = url.host_str()?.to_lowercase();
        let port = url.port_or_known_default()?;
        Some(Self { scheme, host, port })
    }

    pub fn is_same_origin(&self, other: &Origin) -> bool {
        self.scheme == other.scheme && self.host == other.host && self.port == other.port
    }
}

pub struct SecurityPolicy;

impl SecurityPolicy {
    pub fn check_cors(origin: &Origin, allow_origin_header: Option<&str>) -> bool {
        let Some(header) = allow_origin_header else {
            return false;
        };

        let header = header.trim();
        if header == "*" {
            return true;
        }

        if let Ok(allowed_url) = Url::parse(header) {
            if let Some(allowed_origin) = Origin::from_url(&allowed_url) {
                return origin.is_same_origin(&allowed_origin);
            }
        }

        false
    }

    pub fn is_request_allowed(current_origin: Option<&Origin>, target_url: &Url, is_xhr_or_fetch: bool) -> bool {
        if target_url.scheme() == "javascript" {
            return false;
        }

        if !is_xhr_or_fetch {
            return true;
        }

        if target_url.scheme() == "data" || target_url.scheme() == "about" {
            return true;
        }

        let Some(curr) = current_origin else {
            return true;
        };

        if let Some(target_origin) = Origin::from_url(target_url) {
            curr.is_same_origin(&target_origin)
        } else {
            false
        }
    }
}
