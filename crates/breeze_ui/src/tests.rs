#[cfg(test)]
mod tests {
    use crate::app::App;
    use minifb::Key;

    #[test]
    fn test_address_bar_typing() {
        let mut app = App::new(800, 600, "about:blank");
        // Click on the address bar to focus it
        app.handle_mouse_click(150.0, 50.0);
        assert!(app.address_bar_focused);

        // Clear existing text by backspacing or setting
        app.address_bar_text.clear();

        // Type characters
        for c in "https://example.org".chars() {
            app.handle_char_input(c);
        }
        assert_eq!(app.address_bar_text, "https://example.org");

        // Backspace
        app.handle_key_input(Key::Backspace);
        assert_eq!(app.address_bar_text, "https://example.or");

        // Type 'g' and hit Enter
        app.handle_char_input('g');
        app.handle_key_input(Key::Enter);
        assert!(!app.address_bar_focused);
        assert_eq!(app.tabs[app.active_tab_idx].url, "https://example.org");
    }
}
