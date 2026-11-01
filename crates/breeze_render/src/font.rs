use fontdue::{Font, FontSettings};
use std::sync::Arc;

pub struct FontContext {
    default_font: Arc<Font>,
}

impl FontContext {
    pub fn new() -> Self {
        let dummy_bytes = include_bytes!("../fonts/LiberationSans-Regular.ttf");
        let default_font = Arc::new(Font::from_bytes(dummy_bytes.as_slice(), FontSettings::default()).expect("valid font"));

        Self {
            default_font,
        }
    }

    pub fn get_font(&self, _family: &str) -> Arc<Font> {
        Arc::clone(&self.default_font)
    }
}
