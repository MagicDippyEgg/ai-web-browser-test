use crate::cache::HttpCache;
use crate::cookie_store::CookieStore;
use crate::security::{Origin, SecurityPolicy};
use breeze_core::error::{BreezeError, Result};
use reqwest::blocking::Client as ReqwestClient;
use reqwest::header::{HeaderMap, HeaderValue, CACHE_CONTROL, CONTENT_TYPE, COOKIE, SET_COOKIE};
use reqwest::redirect::Policy;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    Document,
    Style,
    Script,
    Image,
    Font,
    Fetch,
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub url: Url,
    pub status: u16,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }
}

pub struct HttpClient {
    client: ReqwestClient,
    cookie_store: Arc<Mutex<CookieStore>>,
    cache: Arc<Mutex<HttpCache>>,
}

impl HttpClient {
    pub fn new() -> Self {
        let client = ReqwestClient::builder()
            .use_rustls_tls()
            .redirect(Policy::limited(10))
            .timeout(Duration::from_secs(15))
            .build()
            .expect("failed to build reqwest client");

        Self {
            client,
            cookie_store: Arc::new(Mutex::new(CookieStore::new())),
            cache: Arc::new(Mutex::new(HttpCache::new())),
        }
    }

    pub fn fetch(
        &self,
        url: &Url,
        resource_type: ResourceType,
        current_origin: Option<&Origin>,
    ) -> Result<HttpResponse> {
        let is_xhr = resource_type == ResourceType::Fetch;
        if !SecurityPolicy::is_request_allowed(current_origin, url, is_xhr) {
            return Err(BreezeError::Network(format!(
                "Blocked by Same-Origin Policy: origin {:?} cannot fetch {:?}",
                current_origin, url
            )));
        }

        if url.scheme() == "data" {
            let path = url.path();
            let body = if let Some((_, data)) = path.split_once(',') {
                data.as_bytes().to_vec()
            } else {
                path.as_bytes().to_vec()
            };
            return Ok(HttpResponse {
                url: url.clone(),
                status: 200,
                headers: HeaderMap::new(),
                body,
            });
        }

        {
            let cache = self.cache.lock().unwrap();
            if let Some(entry) = cache.get(url.as_str()) {
                let mut headers = HeaderMap::new();
                if let Some(ct) = &entry.content_type {
                    if let Ok(hv) = HeaderValue::from_str(ct) {
                        headers.insert(CONTENT_TYPE, hv);
                    }
                }
                return Ok(HttpResponse {
                    url: url.clone(),
                    status: 200,
                    headers,
                    body: entry.body.clone(),
                });
            }
        }

        let mut req = self.client.get(url.as_str());

        if let Some(cookie_val) = self.cookie_store.lock().unwrap().get_cookie_header(url) {
            if let Ok(hv) = HeaderValue::from_str(&cookie_val) {
                req = req.header(COOKIE, hv);
            }
        }

        let resp = req.send().map_err(|e| BreezeError::Network(e.to_string()))?;
        let final_url = resp.url().clone();
        let status = resp.status().as_u16();
        let headers = resp.headers().clone();

        for val in headers.get_all(SET_COOKIE) {
            if let Ok(s) = val.to_str() {
                self.cookie_store.lock().unwrap().set_cookie_header(url, s);
            }
        }

        let bytes = resp.bytes().map_err(|e| BreezeError::Network(e.to_string()))?.to_vec();

        if is_xhr {
            if let Some(origin) = current_origin {
                let allow_origin = headers.get("access-control-allow-origin").and_then(|v| v.to_str().ok());
                let target_origin = Origin::from_url(url);
                let same = target_origin.map(|t| origin.is_same_origin(&t)).unwrap_or(false);
                if !same && !SecurityPolicy::check_cors(origin, allow_origin) {
                    return Err(BreezeError::Network("CORS header check failed".to_string()));
                }
            }
        }

        let mut max_age = None;
        if let Some(cc) = headers.get(CACHE_CONTROL).and_then(|v| v.to_str().ok()) {
            for part in cc.split(',') {
                let part = part.trim();
                if let Some(s) = part.strip_prefix("max-age=") {
                    if let Ok(secs) = s.parse::<u64>() {
                        max_age = Some(Duration::from_secs(secs));
                    }
                }
            }
        }

        let content_type = headers.get(CONTENT_TYPE).and_then(|v| v.to_str().ok()).map(|s| s.to_string());
        let etag = headers.get("etag").and_then(|v| v.to_str().ok()).map(|s| s.to_string());

        if status == 200 {
            self.cache.lock().unwrap().put(
                url.as_str().to_string(),
                bytes.clone(),
                content_type,
                etag,
                max_age,
            );
        }

        Ok(HttpResponse {
            url: final_url,
            status,
            headers,
            body: bytes,
        })
    }
}
