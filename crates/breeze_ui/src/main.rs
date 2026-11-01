use clap::Parser;
use minifb::{Key, Window, WindowOptions};
use std::time::Duration;

pub mod app;
pub mod tab;
pub mod ui_chrome;

use app::App;

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

    window.limit_update_rate(Some(Duration::from_micros(16600))); // ~60fps

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Handle Mouse Clicks
        if window.get_mouse_down(minifb::MouseButton::Left) {
            if let Some((mx, my)) = window.get_mouse_pos(minifb::MouseMode::Discard) {
                browser_app.handle_mouse_click(mx, my);
            }
        }

        // Handle Keyboard Keys
        let keys = window.get_keys_pressed(minifb::KeyRepeat::Yes);
        for key in keys {
            browser_app.handle_key_input(key);
        }

        let buffer = browser_app.render_frame();
        window.update_with_buffer(&buffer, args.width, args.height).unwrap();
    }
}
