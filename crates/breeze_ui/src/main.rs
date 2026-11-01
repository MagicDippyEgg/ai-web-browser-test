use clap::Parser;
use minifb::{InputCallback, Key, Window, WindowOptions};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub mod app;
pub mod tab;
pub mod ui_chrome;
#[cfg(test)]
pub mod tests;

use app::App;

struct WindowInputHandler {
    char_queue: Arc<Mutex<Vec<char>>>,
}

impl InputCallback for WindowInputHandler {
    fn add_char(&mut self, uni_char: u32) {
        if let Some(c) = char::from_u32(uni_char) {
            self.char_queue.lock().unwrap().push(c);
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Breeze Web Browser built from scratch", long_about = None)]
struct Args {
    #[arg(default_value = "about:blank")]
    url: String,

    #[arg(long)]
    headless: bool,

    #[arg(long)]
    screenshot: Option<String>,

    #[arg(long)]
    dump_dom: bool,

    #[arg(long, default_value_t = 1024)]
    width: usize,

    #[arg(long, default_value_t = 768)]
    height: usize,
}

fn main() {
    let args = Args::parse();

    let mut browser_app = App::new(args.width, args.height, &args.url);

    // Headless mode
    if args.headless {
        let frame_buffer = browser_app.render_frame();

        if args.dump_dom {
            let active_tab = &browser_app.tabs[browser_app.active_tab_idx];
            let text = breeze_html::dom::Document::text_content(&active_tab.doc.root);
            println!("{}", text);
        }

        if let Some(screenshot_path) = args.screenshot {
            let mut img_buf = image::RgbaImage::new(args.width as u32, args.height as u32);
            for (idx, &pixel) in frame_buffer.iter().enumerate() {
                let x = (idx % args.width) as u32;
                let y = (idx / args.width) as u32;
                let a = ((pixel >> 24) & 0xff) as u8;
                let r = ((pixel >> 16) & 0xff) as u8;
                let g = ((pixel >> 8) & 0xff) as u8;
                let b = (pixel & 0xff) as u8;
                img_buf.put_pixel(x, y, image::Rgba([r, g, b, a]));
            }
            img_buf.save(&screenshot_path).expect("failed to save screenshot");
            println!("Screenshot saved to {}", screenshot_path);
        }
        return;
    }

    // Interactive GUI desktop window
    let mut window = Window::new(
        "Breeze Browser",
        args.width,
        args.height,
        WindowOptions {
            resize: false,
            ..WindowOptions::default()
        },
    ).expect("Unable to open window");

    let char_queue = Arc::new(Mutex::new(Vec::new()));
    let input_handler = WindowInputHandler {
        char_queue: Arc::clone(&char_queue),
    };
    window.set_input_callback(Box::new(input_handler));
    window.set_target_fps(60);

    let mut prev_mouse_down = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Debounce Mouse Clicks (only trigger on edge transition mouse down)
        let is_mouse_down = window.get_mouse_down(minifb::MouseButton::Left);
        if is_mouse_down && !prev_mouse_down {
            if let Some((mx, my)) = window.get_mouse_pos(minifb::MouseMode::Discard) {
                browser_app.handle_mouse_click(mx, my);
            }
        }
        prev_mouse_down = is_mouse_down;

        // 1. Handle Typed Characters from InputCallback (X11 / macOS / Windows)
        let typed_chars: Vec<char> = {
            let mut q = char_queue.lock().unwrap();
            q.drain(..).collect()
        };
        let had_input_chars = !typed_chars.is_empty();
        for ch in typed_chars {
            browser_app.handle_char_input(ch);
        }

        // 2. Fallback Key-to-char mapping for Wayland when set_input_callback does not dispatch
        let shift = window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift);
        let keys = window.get_keys_pressed(minifb::KeyRepeat::Yes);

        for key in &keys {
            if !had_input_chars && browser_app.address_bar_focused {
                if let Some(ch) = map_key_to_char(*key, shift) {
                    browser_app.handle_char_input(ch);
                }
            }
            browser_app.handle_key_input(*key);
        }

        let buffer = browser_app.render_frame();
        window.update_with_buffer(&buffer, args.width, args.height).unwrap();
    }
}

fn map_key_to_char(key: Key, shift: bool) -> Option<char> {
    match key {
        Key::A => Some(if shift { 'A' } else { 'a' }),
        Key::B => Some(if shift { 'B' } else { 'b' }),
        Key::C => Some(if shift { 'C' } else { 'c' }),
        Key::D => Some(if shift { 'D' } else { 'd' }),
        Key::E => Some(if shift { 'E' } else { 'e' }),
        Key::F => Some(if shift { 'F' } else { 'f' }),
        Key::G => Some(if shift { 'G' } else { 'g' }),
        Key::H => Some(if shift { 'H' } else { 'h' }),
        Key::I => Some(if shift { 'I' } else { 'i' }),
        Key::J => Some(if shift { 'J' } else { 'j' }),
        Key::K => Some(if shift { 'K' } else { 'k' }),
        Key::L => Some(if shift { 'L' } else { 'l' }),
        Key::M => Some(if shift { 'M' } else { 'm' }),
        Key::N => Some(if shift { 'N' } else { 'n' }),
        Key::O => Some(if shift { 'O' } else { 'o' }),
        Key::P => Some(if shift { 'P' } else { 'p' }),
        Key::Q => Some(if shift { 'Q' } else { 'q' }),
        Key::R => Some(if shift { 'R' } else { 'r' }),
        Key::S => Some(if shift { 'S' } else { 's' }),
        Key::T => Some(if shift { 'T' } else { 't' }),
        Key::U => Some(if shift { 'U' } else { 'u' }),
        Key::V => Some(if shift { 'V' } else { 'v' }),
        Key::W => Some(if shift { 'W' } else { 'w' }),
        Key::X => Some(if shift { 'X' } else { 'x' }),
        Key::Y => Some(if shift { 'Y' } else { 'y' }),
        Key::Z => Some(if shift { 'Z' } else { 'z' }),
        Key::Key0 => Some(if shift { ')' } else { '0' }),
        Key::Key1 => Some(if shift { '!' } else { '1' }),
        Key::Key2 => Some(if shift { '@' } else { '2' }),
        Key::Key3 => Some(if shift { '#' } else { '3' }),
        Key::Key4 => Some(if shift { '$' } else { '4' }),
        Key::Key5 => Some(if shift { '%' } else { '5' }),
        Key::Key6 => Some(if shift { '^' } else { '6' }),
        Key::Key7 => Some(if shift { '&' } else { '7' }),
        Key::Key8 => Some(if shift { '*' } else { '8' }),
        Key::Key9 => Some(if shift { '(' } else { '9' }),
        Key::Period => Some(if shift { '>' } else { '.' }),
        Key::Slash => Some(if shift { '?' } else { '/' }),
        Key::Minus => Some(if shift { '_' } else { '-' }),
        Key::Equal => Some(if shift { '+' } else { '=' }),
        Key::Semicolon => Some(if shift { ':' } else { ';' }),
        Key::Space => Some(' '),
        _ => None,
    }
}
