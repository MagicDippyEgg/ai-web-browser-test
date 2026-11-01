use fontdue::{Font, FontSettings};
use std::sync::Arc;
use tiny_skia::{Color, Paint, Pixmap, Rect, Transform};

pub struct UiChrome {
    font: Arc<Font>,
}

impl UiChrome {
    pub fn new() -> Self {
        let dummy_bytes = include_bytes!("../../../crates/breeze_render/fonts/LiberationSans-Regular.ttf");
        let font = Arc::new(Font::from_bytes(dummy_bytes.as_slice(), FontSettings::default()).expect("valid font"));
        Self { font }
    }

    pub fn render_chrome(
        &self,
        pixmap: &mut Pixmap,
        width: u32,
        height: u32,
        tabs: &[crate::tab::Tab],
        active_tab_idx: usize,
        address_bar_text: &str,
        address_bar_focused: bool,
        is_loading: bool,
    ) {
        // 1. Tab Bar background (height 36px)
        let tab_bar_bg = Color::from_rgba8(235, 238, 242, 255);
        if let Some(r) = Rect::from_xywh(0.0, 0.0, width as f32, 36.0) {
            let mut paint = Paint::default();
            paint.set_color(tab_bar_bg);
            pixmap.fill_rect(r, &paint, Transform::identity(), None);
        }

        // Render Tabs
        let mut tab_x = 8.0f32;
        let tab_w = 160.0f32;
        let tab_h = 28.0f32;

        for (idx, tab) in tabs.iter().enumerate() {
            let is_active = idx == active_tab_idx;
            let bg_color = if is_active {
                Color::from_rgba8(255, 255, 255, 255)
            } else {
                Color::from_rgba8(220, 224, 230, 255)
            };

            if let Some(r) = Rect::from_xywh(tab_x, 8.0, tab_w, tab_h) {
                let mut paint = Paint::default();
                paint.set_color(bg_color);
                pixmap.fill_rect(r, &paint, Transform::identity(), None);

                // Tab Title Text
                let mut title = tab.title.clone();
                if title.len() > 16 {
                    title.truncate(14);
                    title.push_str("..");
                }
                self.draw_text(pixmap, &title, tab_x + 10.0, 26.0, 12.0, [50, 50, 50, 255]);

                // Close button 'x'
                self.draw_text(pixmap, "×", tab_x + tab_w - 18.0, 25.0, 14.0, [120, 120, 120, 255]);
            }
            tab_x += tab_w + 4.0;
        }

        // '+' New tab button
        if let Some(r) = Rect::from_xywh(tab_x, 8.0, 28.0, tab_h) {
            let mut paint = Paint::default();
            paint.set_color(Color::from_rgba8(220, 224, 230, 255));
            pixmap.fill_rect(r, &paint, Transform::identity(), None);
            self.draw_text(pixmap, "+", tab_x + 9.0, 26.0, 16.0, [80, 80, 80, 255]);
        }

        // 2. Navigation & Address Bar background (height 40px, y: 36..76)
        if let Some(r) = Rect::from_xywh(0.0, 36.0, width as f32, 40.0) {
            let mut paint = Paint::default();
            paint.set_color(Color::from_rgba8(245, 246, 248, 255));
            pixmap.fill_rect(r, &paint, Transform::identity(), None);
        }

        // Navigation buttons: Back [<], Forward [>], Reload [R]
        let btn_y = 42.0;
        let btn_h = 28.0;

        // Back button
        if let Some(r) = Rect::from_xywh(10.0, btn_y, 28.0, btn_h) {
            let mut paint = Paint::default();
            paint.set_color(Color::from_rgba8(230, 232, 236, 255));
            pixmap.fill_rect(r, &paint, Transform::identity(), None);
            self.draw_text(pixmap, "<", 19.0, btn_y + 19.0, 14.0, [80, 80, 80, 255]);
        }

        // Forward button
        if let Some(r) = Rect::from_xywh(44.0, btn_y, 28.0, btn_h) {
            let mut paint = Paint::default();
            paint.set_color(Color::from_rgba8(230, 232, 236, 255));
            pixmap.fill_rect(r, &paint, Transform::identity(), None);
            self.draw_text(pixmap, ">", 53.0, btn_y + 19.0, 14.0, [80, 80, 80, 255]);
        }

        // Reload button
        if let Some(r) = Rect::from_xywh(78.0, btn_y, 28.0, btn_h) {
            let mut paint = Paint::default();
            paint.set_color(Color::from_rgba8(230, 232, 236, 255));
            pixmap.fill_rect(r, &paint, Transform::identity(), None);
            self.draw_text(pixmap, "⟳", 86.0, btn_y + 20.0, 14.0, [80, 80, 80, 255]);
        }

        // Address bar box
        let addr_x = 116.0;
        let addr_w = (width as f32 - 130.0).max(100.0);
        if let Some(r) = Rect::from_xywh(addr_x, btn_y, addr_w, btn_h) {
            let mut paint = Paint::default();
            paint.set_color(Color::WHITE);
            pixmap.fill_rect(r, &paint, Transform::identity(), None);

            // Border
            let border_col = if address_bar_focused {
                Color::from_rgba8(26, 115, 232, 255) // blue focus ring
            } else {
                Color::from_rgba8(200, 204, 210, 255)
            };
            let mut stroke_paint = Paint::default();
            stroke_paint.set_color(border_col);
            let stroke = tiny_skia::Stroke { width: 1.5, ..Default::default() };
            let path = tiny_skia::PathBuilder::from_rect(r);
            pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);

            // Text
            let display_text = if address_bar_text.is_empty() {
                "Enter URL or search..."
            } else {
                address_bar_text
            };
            let text_color = if address_bar_text.is_empty() {
                [150, 150, 150, 255]
            } else {
                [30, 30, 30, 255]
            };
            self.draw_text(pixmap, display_text, addr_x + 10.0, btn_y + 19.0, 13.0, text_color);
        }

        // Loading indicator line (height 2px below chrome)
        if is_loading {
            if let Some(r) = Rect::from_xywh(0.0, 75.0, (width as f32) * 0.65, 3.0) {
                let mut paint = Paint::default();
                paint.set_color(Color::from_rgba8(26, 115, 232, 255));
                pixmap.fill_rect(r, &paint, Transform::identity(), None);
            }
        }
    }

    fn draw_text(&self, pixmap: &mut Pixmap, text: &str, start_x: f32, baseline_y: f32, size: f32, color: [u8; 4]) {
        let mut pen_x = start_x;
        for ch in text.chars() {
            let (metrics, bitmap) = self.font.rasterize(ch, size);
            let gx = pen_x + metrics.xmin as f32;
            let gy = baseline_y - metrics.height as f32 - metrics.ymin as f32;

            for row in 0..metrics.height {
                for col_idx in 0..metrics.width {
                    let alpha = bitmap[row * metrics.width + col_idx];
                    if alpha > 0 {
                        let px = gx as i32 + col_idx as i32;
                        let py = gy as i32 + row as i32;
                        if px >= 0 && px < pixmap.width() as i32 && py >= 0 && py < pixmap.height() as i32 {
                            let target_alpha = ((alpha as u32 * color[3] as u32) / 255) as u8;
                            let src_col = Color::from_rgba8(color[0], color[1], color[2], target_alpha);
                            Self::blend_pixel(pixmap, px as u32, py as u32, src_col);
                        }
                    }
                }
            }
            pen_x += metrics.advance_width;
        }
    }

    fn blend_pixel(pixmap: &mut Pixmap, x: u32, y: u32, src: Color) {
        let idx = (y * pixmap.width() + x) as usize * 4;
        let data = pixmap.data_mut();
        let src_a = src.alpha();
        let inv_a = 1.0 - src_a;

        let dst_r = data[idx] as f32 / 255.0;
        let dst_g = data[idx + 1] as f32 / 255.0;
        let dst_b = data[idx + 2] as f32 / 255.0;
        let dst_a = data[idx + 3] as f32 / 255.0;

        let out_r = src.red() * src_a + dst_r * inv_a;
        let out_g = src.green() * src_a + dst_g * inv_a;
        let out_b = src.blue() * src_a + dst_b * inv_a;
        let out_a = src_a + dst_a * inv_a;

        data[idx] = (out_r * 255.0).clamp(0.0, 255.0) as u8;
        data[idx + 1] = (out_g * 255.0).clamp(0.0, 255.0) as u8;
        data[idx + 2] = (out_b * 255.0).clamp(0.0, 255.0) as u8;
        data[idx + 3] = (out_a * 255.0).clamp(0.0, 255.0) as u8;
    }
}
