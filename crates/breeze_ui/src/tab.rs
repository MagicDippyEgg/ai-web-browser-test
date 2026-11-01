use breeze_css::parser::CssParser;
use breeze_css::style::StyleEngine;
use breeze_html::dom::{Document, NodeData, NodeRef};
use breeze_html::parser::HtmlParser;
use breeze_js::JsRuntime;
use breeze_layout::build_layout_tree;
use breeze_layout::layout_tree::LayoutBox;
use breeze_layout::Rect;
use breeze_net::{HttpClient, HttpResponse, Origin, ResourceType};
use breeze_render::Rasterizer;
use std::rc::Rc;
use url::Url;

pub struct Tab {
    pub id: usize,
    pub title: String,
    pub url: String,
    pub history: Vec<String>,
    pub history_idx: usize,
    pub doc: Document,
    pub layout_tree: Option<LayoutBox>,
    pub js_runtime: Option<JsRuntime>,
    pub scroll_y: f32,
    pub is_loading: bool,
    pub error_msg: Option<String>,
}

impl Tab {
    pub fn new(id: usize, initial_url: &str) -> Self {
        let mut tab = Self {
            id,
            title: "New Tab".to_string(),
            url: initial_url.to_string(),
            history: vec![initial_url.to_string()],
            history_idx: 0,
            doc: Document::new(),
            layout_tree: None,
            js_runtime: None,
            scroll_y: 0.0,
            is_loading: false,
            error_msg: None,
        };
        tab.load_url(initial_url, 1024.0, 768.0);
        tab
    }

    pub fn load_url(&mut self, url_str: &str, vp_w: f32, vp_h: f32) {
        self.url = url_str.to_string();
        self.is_loading = true;
        self.error_msg = None;
        self.scroll_y = 0.0;

        let parsed_url = if let Ok(u) = Url::parse(url_str) {
            u
        } else if let Ok(u) = Url::parse(&format!("http://{}", url_str)) {
            u
        } else {
            self.error_msg = Some(format!("Invalid URL: {}", url_str));
            self.render_error_page("Invalid URL", &format!("Could not parse URL '{}'", url_str));
            self.is_loading = false;
            return;
        };

        let html_content = if parsed_url.scheme() == "about" {
            match parsed_url.path() {
                "blank" => "<html><body></body></html>".to_string(),
                "history" => "<html><body style='font-family:sans-serif;padding:20px;'><h1>Browsing History</h1><p>No recent history.</p></body></html>".to_string(),
                "bookmarks" => "<html><body style='font-family:sans-serif;padding:20px;'><h1>Bookmarks</h1><p>No saved bookmarks.</p></body></html>".to_string(),
                _ => format!("<html><body><h1>About {}</h1></body></html>", parsed_url.path()),
            }
        } else if parsed_url.scheme() == "data" {
            let path = parsed_url.path();
            if let Some((_, data)) = path.split_once(',') {
                data.to_string()
            } else {
                path.to_string()
            }
        } else {
            let client = HttpClient::new();
            match client.fetch(&parsed_url, ResourceType::Document, None) {
                Ok(resp) => {
                    if resp.status >= 400 {
                        self.render_error_page(
                            &format!("HTTP Error {}", resp.status),
                            &format!("Server responded with status {}", resp.status),
                        );
                        self.is_loading = false;
                        return;
                    }
                    resp.text()
                }
                Err(e) => {
                    self.error_msg = Some(e.to_string());
                    self.render_error_page("Network Error", &e.to_string());
                    self.is_loading = false;
                    return;
                }
            }
        };

        self.doc = HtmlParser::parse(&html_content);

        // Extract title
        let titles = self.doc.get_elements_by_tag_name("title");
        if let Some(t_node) = titles.first() {
            let t = Document::raw_text_content(t_node).trim().to_string();
            if !t.is_empty() {
                self.title = t;
            } else {
                self.title = parsed_url.host_str().unwrap_or(url_str).to_string();
            }
        } else {
            self.title = parsed_url.host_str().unwrap_or(url_str).to_string();
        }

        // Initialize JavaScript runtime
        let mut js_runtime = JsRuntime::new(self.doc.clone(), Some(parsed_url.clone()));

        // Run <script> tags (both inline and external src)
        let scripts = self.doc.get_elements_by_tag_name("script");
        for script_node in scripts {
            let (src_opt, inline_code) = {
                let nb = script_node.borrow();
                let src = if let NodeData::Element(el) = &nb.data {
                    el.get_attribute("src").map(|s| s.to_string())
                } else {
                    None
                };
                (src, Document::raw_text_content(&script_node))
            };

            if let Some(src) = src_opt {
                if let Ok(script_url) = breeze_core::url_util::resolve_relative_url(&parsed_url, &src) {
                    let client = HttpClient::new();
                    if let Ok(resp) = client.fetch(&script_url, ResourceType::Script, None) {
                        let _ = js_runtime.execute(&resp.text());
                    }
                }
            } else if !inline_code.trim().is_empty() {
                let _ = js_runtime.execute(&inline_code);
            }
        }

        self.js_runtime = Some(js_runtime);

        // Build style engine and layout tree
        self.rebuild_layout(vp_w, vp_h);
        self.is_loading = false;
    }

    pub fn rebuild_layout(&mut self, vp_w: f32, vp_h: f32) {
        let mut style_engine = StyleEngine::new(vp_w, vp_h);

        // Extract <style> blocks
        let styles = self.doc.get_elements_by_tag_name("style");
        for style_node in styles {
            let css_text = Document::raw_text_content(&style_node);
            let sheet = CssParser::parse(&css_text);
            style_engine.add_stylesheet(sheet);
        }

        // Fetch external stylesheets: <link rel="stylesheet" href="...">
        let links = self.doc.get_elements_by_tag_name("link");
        let client = HttpClient::new();
        for link_node in links {
            let (rel, href) = {
                let nb = link_node.borrow();
                if let NodeData::Element(el) = &nb.data {
                    (
                        el.get_attribute("rel").unwrap_or_default().to_string(),
                        el.get_attribute("href").unwrap_or_default().to_string(),
                    )
                } else {
                    (String::new(), String::new())
                }
            };

            if rel.eq_ignore_ascii_case("stylesheet") && !href.is_empty() {
                if let Ok(base_u) = Url::parse(&self.url) {
                    if let Ok(css_url) = breeze_core::url_util::resolve_relative_url(&base_u, &href) {
                        if let Ok(resp) = client.fetch(&css_url, ResourceType::Style, None) {
                            let sheet = CssParser::parse(&resp.text());
                            style_engine.add_stylesheet(sheet);
                        }
                    }
                }
            }
        }

        if let Some(mut root_box) = build_layout_tree(&self.doc.root, &style_engine, None) {
            let vp = Rect::new(0.0, 0.0, vp_w, vp_h);
            root_box.layout(&vp);
            self.layout_tree = Some(root_box);
        }
    }

    pub fn tick(&mut self, vp_w: f32, vp_h: f32) {
        if let Some(runtime) = &mut self.js_runtime {
            runtime.tick_timers();
        }
    }

    pub fn can_go_back(&self) -> bool {
        self.history_idx > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.history_idx + 1 < self.history.len()
    }

    pub fn go_back(&mut self, vp_w: f32, vp_h: f32) {
        if self.can_go_back() {
            self.history_idx -= 1;
            let prev_url = self.history[self.history_idx].clone();
            self.load_url(&prev_url, vp_w, vp_h);
        }
    }

    pub fn go_forward(&mut self, vp_w: f32, vp_h: f32) {
        if self.can_go_forward() {
            self.history_idx += 1;
            let next_url = self.history[self.history_idx].clone();
            self.load_url(&next_url, vp_w, vp_h);
        }
    }

    pub fn reload(&mut self, vp_w: f32, vp_h: f32) {
        let curr = self.url.clone();
        self.load_url(&curr, vp_w, vp_h);
    }

    fn render_error_page(&mut self, error_title: &str, error_details: &str) {
        let err_html = format!(
            r#"
            <html>
                <head><title>{}</title></head>
                <body style="font-family:sans-serif; background-color:#f9f9f9; padding:40px; color:#333;">
                    <div style="background-color:#ffffff; padding:30px; border:1px solid #ddd; border-radius:8px; max-width:600px; margin:0 auto;">
                        <h1 style="color:#d9534f; margin-top:0;">{}</h1>
                        <p style="font-size:16px;">The requested URL <b>{}</b> could not be loaded.</p>
                        <p style="color:#666; font-size:14px; background:#f4f4f4; padding:10px; border-left:4px solid #d9534f;">{}</p>
                    </div>
                </body>
            </html>
            "#,
            error_title, error_title, self.url, error_details
        );
        self.title = error_title.to_string();
        self.doc = HtmlParser::parse(&err_html);
        self.rebuild_layout(1024.0, 768.0);
    }
}
