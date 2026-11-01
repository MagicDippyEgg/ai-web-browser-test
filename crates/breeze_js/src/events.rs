use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct BrowserEvent {
    pub event_type: String,
    pub target_id: Option<String>,
    pub data: Option<String>,
}

#[derive(Default)]
pub struct EventManager {
    listeners: HashMap<String, HashMap<String, Vec<boa_engine::JsObject>>>,
}

impl EventManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_event_listener(&mut self, target: &str, event_type: &str, callback: boa_engine::JsObject) {
        self.listeners
            .entry(target.to_string())
            .or_default()
            .entry(event_type.to_string())
            .or_default()
            .push(callback);
    }

    pub fn get_listeners(&self, target: &str, event_type: &str) -> Vec<boa_engine::JsObject> {
        self.listeners
            .get(target)
            .and_then(|m| m.get(event_type))
            .cloned()
            .unwrap_or_default()
    }
}
