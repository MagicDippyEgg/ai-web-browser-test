use crate::tab::Tab;
use crate::ui_chrome::UiChrome;
use breeze_render::Rasterizer;
use minifb::{Key, Window, WindowOptions};
use tiny_skia::Pixmap;

const CHROME_HEIGHT: f32 = 78.0;

pub struct App {
    pub tabs: Vec<Tab>,
    pub active_tab_idx: usize,
    pub width: usize,
    pub height: usize,
    pub address_bar_text: String,
    pub address_bar_focused: bool,
    pub chrome: UiChrome,
    pub rasterizer: Rasterizer,
}

impl App {
    pub fn new(width: usize, height: usize, initial_url: &str) -> Self {
        let first_tab = Tab::new(1, initial_url);
        let address_bar_text = initial_url.to_string();
        Self {
            tabs: vec![first_tab],
            active_tab_idx: 0,
            width,
            height,
            address_bar_text,
            address_bar_focused: false,
            chrome: UiChrome::new(),
            rasterizer: Rasterizer::new(),
        }
    }

    pub fn new_tab(&mut self, url: &str) {
        let new_id = self.tabs.len() + 1;
        let tab = Tab::new(new_id, url);
        self.tabs.push(tab);
        self.active_tab_idx = self.tabs.len() - 1;
        self.address_bar_text = url.to_string();
    }

    pub fn close_tab(&mut self, idx: usize) {
        if self.tabs.len() > 1 {
            self.tabs.remove(idx);
            if self.active_tab_idx >= self.tabs.len() {
                self.active_tab_idx = self.tabs.len() - 1;
            }
            self.address_bar_text = self.tabs[self.active_tab_idx].url.clone();
        }
    }

    pub fn render_frame(&mut self) -> Vec<u32> {
        let w = self.width as u32;
        let h = self.height as u32;
        let mut pixmap = Pixmap::new(w, h).unwrap();
        pixmap.fill(tiny_skia::Color::WHITE);

        self.tabs[self.active_tab_idx].tick(self.width as f32, self.height as f32 - CHROME_HEIGHT);

        // Render Web Page Content (below chrome)
        if let Some(root_box) = &self.tabs[self.active_tab_idx].layout_tree {
            let content_h = (h as f32 - CHROME_HEIGHT) as u32;
            let scroll_y = self.tabs[self.active_tab_idx].scroll_y;
            let page_pixmap = self.rasterizer.render_to_pixmap(root_box, w, content_h, scroll_y);
            pixmap.draw_pixmap(
                0,
                CHROME_HEIGHT as i32,
                page_pixmap.as_ref(),
                &tiny_skia::PixmapPaint::default(),
                tiny_skia::Transform::identity(),
                None,
            );
        }

        let is_loading = self.tabs[self.active_tab_idx].is_loading;

        // Render UI Chrome (tab bar, nav buttons, address bar)
        self.chrome.render_chrome(
            &mut pixmap,
            w,
            h,
            &self.tabs,
            self.active_tab_idx,
            &self.address_bar_text,
            self.address_bar_focused,
            is_loading,
        );

        // Convert pixmap to Vec<u32> 0xAARRGGBB for minifb
        let data = pixmap.data();
        let mut buffer = vec![0u32; self.width * self.height];
        for (i, chunk) in data.chunks_exact(4).enumerate() {
            let r = chunk[0] as u32;
            let g = chunk[1] as u32;
            let b = chunk[2] as u32;
            let a = chunk[3] as u32;
            buffer[i] = (a << 24) | (r << 16) | (g << 8) | b;
        }
        buffer
    }

    pub fn handle_mouse_click(&mut self, mouse_x: f32, mouse_y: f32) {
        if mouse_y < 36.0 {
            // Click inside tab bar
            let mut tab_x = 8.0f32;
            let tab_w = 160.0f32;
            for (idx, _tab) in self.tabs.iter().enumerate() {
                if mouse_x >= tab_x && mouse_x <= tab_x + tab_w {
                    // Check if clicked close 'x'
                    if mouse_x >= tab_x + tab_w - 24.0 {
                        self.close_tab(idx);
                    } else {
                        self.active_tab_idx = idx;
                        self.address_bar_text = self.tabs[self.active_tab_idx].url.clone();
                    }
                    return;
                }
                tab_x += tab_w + 4.0;
            }
            // Check if clicked '+' button
            if mouse_x >= tab_x && mouse_x <= tab_x + 28.0 {
                self.new_tab("about:blank");
            }
        } else if mouse_y >= 36.0 && mouse_y <= 76.0 {
            // Click inside navigation bar
            let btn_y = 42.0;
            let btn_h = 28.0;
            let vp_w = self.width as f32;
            let vp_h = self.height as f32 - CHROME_HEIGHT;

            // Back button: [10..38]
            if mouse_x >= 10.0 && mouse_x <= 38.0 && mouse_y >= btn_y && mouse_y <= btn_y + btn_h {
                self.tabs[self.active_tab_idx].go_back(vp_w, vp_h);
                self.address_bar_text = self.tabs[self.active_tab_idx].url.clone();
                return;
            }

            // Forward button: [44..72]
            if mouse_x >= 44.0 && mouse_x <= 72.0 && mouse_y >= btn_y && mouse_y <= btn_y + btn_h {
                self.tabs[self.active_tab_idx].go_forward(vp_w, vp_h);
                self.address_bar_text = self.tabs[self.active_tab_idx].url.clone();
                return;
            }

            // Reload button: [78..106]
            if mouse_x >= 78.0 && mouse_x <= 106.0 && mouse_y >= btn_y && mouse_y <= btn_y + btn_h {
                self.tabs[self.active_tab_idx].reload(vp_w, vp_h);
                return;
            }

            // Address bar: [116..]
            let addr_w = (self.width as f32 - 130.0).max(100.0);
            if mouse_x >= 116.0 && mouse_x <= 116.0 + addr_w && mouse_y >= btn_y && mouse_y <= btn_y + btn_h {
                self.address_bar_focused = true;
                return;
            }
        } else {
            // Content area click
            self.address_bar_focused = false;
        }
    }

    pub fn handle_key_input(&mut self, key: Key) {
        if self.address_bar_focused {
            let vp_w = self.width as f32;
            let vp_h = self.height as f32 - CHROME_HEIGHT;

            match key {
                Key::Enter => {
                    self.address_bar_focused = false;
                    let url = self.address_bar_text.clone();
                    self.tabs[self.active_tab_idx].load_url(&url, vp_w, vp_h);
                }
                Key::Backspace => {
                    self.address_bar_text.pop();
                }
                _ => {}
            }
        }
    }
}
