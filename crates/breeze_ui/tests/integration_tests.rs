use std::fs;

#[test]
fn test_render_all_test_pages_headless() {
    let test_pages = [
        "test_pages/basic_html.html",
        "test_pages/css_styling.html",
        "test_pages/layout_flex_grid.html",
        "test_pages/positioning.html",
        "test_pages/javascript_dom.html",
        "test_pages/responsive.html",
    ];

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let root_dir = std::path::Path::new(&manifest_dir).parent().unwrap().parent().unwrap();

    for page_path in test_pages {
        let full_path = root_dir.join(page_path);
        let content = fs::read_to_string(&full_path).expect("read test page");
        let doc = breeze_html::parser::HtmlParser::parse(&content);
        assert!(!doc.root.borrow().children.is_empty(), "Document should have children");

        let style_engine = breeze_css::style::StyleEngine::new(800.0, 600.0);
        let layout_tree = breeze_layout::build_layout_tree(&doc.root, &style_engine, None);
        assert!(layout_tree.is_some(), "Layout tree should be generated for {}", page_path);

        let mut root_box = layout_tree.unwrap();
        let vp = breeze_layout::Rect::new(0.0, 0.0, 800.0, 600.0);
        root_box.layout(&vp);

        let rasterizer = breeze_render::Rasterizer::new();
        let buffer = rasterizer.render_to_buffer(&root_box, 800, 600, 0.0);
        assert_eq!(buffer.len(), 800 * 600);
    }
}
