# Breeze Web Browser

**Breeze** is a desktop web browser engine built from scratch in Rust.
It does **not** rely on Chromium, WebKit, Gecko, Servo, CEF, Qt WebEngine, Electron, or any other pre-existing browser engine.

---

## Architecture Overview

Breeze is cleanly architected into modular Rust crates:

- **`breeze_core`**: Shared foundational types, error definitions, and robust URL parsing and resolution utilities.
- **`breeze_html`**: HTML tokenizer and parser building a full DOM tree (Node, Element, Text, Comment, DocumentType). Supports self-closing void elements, raw text elements (`<script>`, `<style>`), entity decoding (`&quot;`, `&amp;`, `&#...;`), and form inputs.
- **`breeze_css`**: CSS tokenizer, parser for stylesheet rules and `@media` queries, complex and compound selectors (tag, class, ID, attributes, `:first-child`), specificity calculation, property inheritance, cascade resolution, and default User-Agent styling.
- **`breeze_layout`**: Layout box construction and box model calculation (margin, border, padding, content). Supports:
  - Block formatting context and vertical normal flow.
  - Inline formatting context with text wrapping and word metrics.
  - Flexbox layout (`row`, `column`, `flex-grow`, `gap`, `justify-content`).
  - Grid layout (`grid-template-columns`, `grid-gap`).
  - Positioning (`relative`, `absolute`, offsets, z-index).
  - Box-sizing (`content-box` and `border-box`).
- **`breeze_render`**: 2D software rasterizer built on `tiny-skia` and `fontdue`. Renders background rectangles, rounded corners (`border-radius`), borders, anti-aliased font glyphs, and interactive form controls (text fields, buttons, checkboxes).
- **`breeze_net`**: HTTP and HTTPS networking using `reqwest` + `rustls` (strictly enforcing TLS certificate validation). Implements cookie jar persistence, HTTP caching (ETag / Cache-Control), redirect following, Same-Origin Policy (SOP), and CORS header validation.
- **`breeze_js`**: JavaScript runtime integration using the standalone `boa_engine` (pure Rust ECMAScript implementation). Exposes browser globals (`window`, `document`, `console`, `location`, `localStorage`), DOM manipulation APIs (`getElementById`, `createElement`, `appendChild`, `setAttribute`, `getAttribute`), and timer queue (`setTimeout`, `setInterval`, `clearTimeout`, `clearInterval`).
- **`breeze_ui`**: Desktop GUI application built with `minifb`. Features an interactive multi-tab bar, back and forward navigation, reload and stop buttons, address bar, loading indicator, custom error pages, and headless CLI modes for automated testing and screenshot capture.

---

## Test Pages Collection

Located in `test_pages/`:
1. `basic_html.html`: Standard HTML tags, paragraphs, links, and forms.
2. `css_styling.html`: Colors, fonts, borders, box model, and background styling.
3. `layout_flex_grid.html`: Flexbox rows and CSS grid item layouts.
4. `positioning.html`: Relative positioning and coordinate offsets.
5. `javascript_dom.html`: DOM manipulation and timers via JavaScript.
6. `responsive.html`: Responsive design with CSS `@media` queries.

---

## Build & Run Instructions

### Prerequisites (Ubuntu / Debian Linux)
```bash
sudo apt-get update
sudo apt-get install -y libx11-dev libxcursor-dev libxrandr-dev libxi-dev fonts-liberation
```

### Build
```bash
cargo build --release
```

### Running the Browser GUI
```bash
# Launch default blank page
cargo run -p breeze_ui

# Launch with a specific URL or HTML file
cargo run -p breeze_ui -- https://example.com
```

### Headless Verification & Screenshots
```bash
# Dump rendered DOM text
cargo run -p breeze_ui -- --headless --dump-dom "test_pages/basic_html.html"

# Render and capture a screenshot without opening a GUI window
cargo run -p breeze_ui -- --headless --screenshot output.png "test_pages/css_styling.html"
```

### Running Tests
```bash
cargo test --workspace
```

---

## Known Limitations

- **Complex CSS Specs**: Advanced CSS features such as CSS Animations, complex multi-line flex wrapping, and complex subgrids are simplified.
- **JavaScript Web APIs**: Basic Web APIs (`document`, `window`, `console`, `localStorage`, `setTimeout`, `setInterval`) are implemented; full HTML5 canvas context, WebSockets, and Web Audio are not yet implemented.
- **Image Formats**: Common raster image formats (PNG, JPEG, WebP, GIF) are supported; SVG rendering is currently limited to basic paths.
