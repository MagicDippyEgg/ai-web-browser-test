use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct CachedResponse {
    pub body: Vec<u8>,
    pub content_type: Option<String>,
    pub etag: Option<String>,
    pub expires_at: Option<Instant>,
}

#[derive(Default)]
pub struct HttpCache {
    entries: HashMap<String, CachedResponse>,
}

impl HttpCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, url: &str) -> Option<&CachedResponse> {
        let entry = self.entries.get(url)?;
        if let Some(expires) = entry.expires_at {
            if Instant::now() > expires {
                return None;
            }
        }
        Some(entry)
    }

    pub fn put(&mut self, url: String, body: Vec<u8>, content_type: Option<String>, etag: Option<String>, max_age: Option<Duration>) {
        let expires_at = max_age.map(|d| Instant::now() + d);
        self.entries.insert(url, CachedResponse {
            body,
            content_type,
            etag,
            expires_at,
        });
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
