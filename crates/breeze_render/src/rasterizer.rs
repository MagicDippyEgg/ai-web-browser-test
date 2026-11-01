use crate::font::FontContext;
use breeze_html::dom::NodeData;
use breeze_layout::layout_tree::{BoxType, LayoutBox};
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, Rect, Transform};

pub struct Rasterizer {
    font_ctx: FontContext,
}

impl Rasterizer {
    pub fn new() -> Self {
        Self {
            font_ctx: FontContext::new(),
        }
    }

    pub fn render_to_pixmap(&self, root_box: &LayoutBox, width: u32, height: u32, scroll_y: f32) -> Pixmap {
        let mut pixmap = Pixmap::new(width, height).unwrap();
        pixmap.fill(Color::WHITE);
        self.render_box(&mut pixmap, root_box, scroll_y);
        pixmap
    }

    pub fn render_to_buffer(&self, root_box: &LayoutBox, width: u32, height: u32, scroll_y: f32) -> Vec<u32> {
        let pixmap = self.render_to_pixmap(root_box, width, height, scroll_y);
        let data = pixmap.data();
        let mut buffer = vec![0u32; (width * height) as usize];
        for (i, chunk) in data.chunks_exact(4).enumerate() {
            let r = chunk[0] as u32;
            let g = chunk[1] as u32;
            let b = chunk[2] as u32;
            let a = chunk[3] as u32;
            buffer[i] = (a << 24) | (r << 16) | (g << 8) | b;
        }
        buffer
    }

    fn render_box(&self, pixmap: &mut Pixmap, layout_box: &LayoutBox, scroll_y: f32) {
        let b = layout_box.dimensions.border_box();
        let x = b.x;
        let y = b.y - scroll_y;
        let w = b.width;
        let h = b.height;

        let bg = layout_box.style.background_color;
        if bg[3] > 0 && w > 0.0 && h > 0.0 {
            if let Some(rect) = Rect::from_xywh(x, y, w, h) {
                let mut paint = Paint::default();
                paint.set_color_rgba8(bg[0], bg[1], bg[2], bg[3]);
                paint.anti_alias = true;

                let radius = layout_box.style.border_radius;
                if radius > 0.0 {
                    let mut pb = PathBuilder::new();
                    pb.move_to(x + radius, y);
                    pb.line_to(x + w - radius, y);
                    pb.quad_to(x + w, y, x + w, y + radius);
                    pb.line_to(x + w, y + h - radius);
                    pb.quad_to(x + w, y + h, x + w - radius, y + h);
                    pb.line_to(x + radius, y + h);
                    pb.quad_to(x, y + h, x, y + h - radius);
                    pb.line_to(x, y + radius);
                    pb.quad_to(x, y, x + radius, y);
                    pb.close();
                    if let Some(path) = pb.finish() {
                        pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
                    }
                } else {
                    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
                }
            }
        }

        let bw = layout_box.dimensions.border.top.max(layout_box.dimensions.border.left);
        let bc = layout_box.style.border_color;
        if bw > 0.0 && bc[3] > 0 && w > 0.0 && h > 0.0 {
            let mut stroke_paint = Paint::default();
            stroke_paint.set_color_rgba8(bc[0], bc[1], bc[2], bc[3]);
            stroke_paint.anti_alias = true;

            let mut stroke = tiny_skia::Stroke::default();
            stroke.width = bw;

            if let Some(rect) = Rect::from_xywh(x + bw / 2.0, y + bw / 2.0, (w - bw).max(0.0), (h - bw).max(0.0)) {
                let path = PathBuilder::from_rect(rect);
                pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }

        if let BoxType::Text(text) = &layout_box.box_type {
            let col = layout_box.style.color;
            let font_size = layout_box.style.font_size;
            let font = self.font_ctx.get_font(&layout_box.style.font_family);

            let mut pen_x = layout_box.dimensions.content.x;
            let pen_y = layout_box.dimensions.content.y - scroll_y + font_size * 0.8;

            for ch in text.chars() {
                let (metrics, bitmap) = font.rasterize(ch, font_size);
                let gx = pen_x + metrics.xmin as f32;
                let gy = pen_y - metrics.height as f32 - metrics.ymin as f32;

                for row in 0..metrics.height {
                    for col_idx in 0..metrics.width {
                        let alpha = bitmap[row * metrics.width + col_idx];
                        if alpha > 0 {
                            let px = gx as i32 + col_idx as i32;
                            let py = gy as i32 + row as i32;
                            if px >= 0 && px < pixmap.width() as i32 && py >= 0 && py < pixmap.height() as i32 {
                                let target_alpha = ((alpha as u32 * col[3] as u32) / 255) as u8;
                                let color = Color::from_rgba8(col[0], col[1], col[2], target_alpha);
                                Self::blend_pixel(pixmap, px as u32, py as u32, color);
                            }
                        }
                    }
                }
                pen_x += metrics.advance_width;
            }
        }

        if let Some(node) = &layout_box.node {
            if let NodeData::Element(el) = &node.borrow().data {
                if el.tag_name == "input" {
                    let input_type = el.get_attribute("type").unwrap_or("text");
                    let val = el.get_attribute("value").unwrap_or("");
                    if input_type == "text" && !val.is_empty() && layout_box.children.is_empty() {
                        let font = self.font_ctx.get_font("sans-serif");
                        let mut pen_x = layout_box.dimensions.content.x + 4.0;
                        let pen_y = layout_box.dimensions.content.y - scroll_y + 14.0;
                        for ch in val.chars() {
                            let (metrics, bitmap) = font.rasterize(ch, 14.0);
                            let gx = pen_x + metrics.xmin as f32;
                            let gy = pen_y - metrics.height as f32 - metrics.ymin as f32;

                            for row in 0..metrics.height {
                                for col_idx in 0..metrics.width {
                                    let alpha = bitmap[row * metrics.width + col_idx];
                                    if alpha > 0 {
                                        let px = gx as i32 + col_idx as i32;
                                        let py = gy as i32 + row as i32;
                                        if px >= 0 && px < pixmap.width() as i32 && py >= 0 && py < pixmap.height() as i32 {
                                            let color = Color::from_rgba8(0, 0, 0, alpha);
                                            Self::blend_pixel(pixmap, px as u32, py as u32, color);
                                        }
                                    }
                                }
                            }
                            pen_x += metrics.advance_width;
                        }
                    } else if input_type == "checkbox" {
                        let checked = el.get_attribute("checked").is_some();
                        if checked {
                            let mut paint = Paint::default();
                            paint.set_color_rgba8(0, 120, 215, 255);
                            let ix = x + 3.0;
                            let iy = y + 3.0;
                            if let Some(r) = Rect::from_xywh(ix, iy, 10.0, 10.0) {
                                pixmap.fill_rect(r, &paint, Transform::identity(), None);
                            }
                        }
                    }
                }
            }
        }

        for child in &layout_box.children {
            self.render_box(pixmap, child, scroll_y);
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
