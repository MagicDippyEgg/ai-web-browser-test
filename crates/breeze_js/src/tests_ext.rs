#[cfg(test)]
mod additional_tests {
    use crate::JsRuntime;
    use breeze_html::dom::{Document, NodeData};
    use breeze_html::parser::HtmlParser;

    #[test]
    fn test_query_selector_and_event_listener() {
        let html = r##"<div id="box" class="active">Hello</div>"##;
        let doc = HtmlParser::parse(html);
        let mut runtime = JsRuntime::new(doc, None);

        let script = r##"
            let el = document.querySelector('#box');
            if (el) {
                el.setAttribute('data-found', 'yes');
            }
            window.addEventListener('load', function() {
                console.log('window loaded!');
            });
            document.addEventListener('click', function() {
                console.log('document clicked!');
            });
        "##;
        runtime.execute(script).unwrap();

        let elem = runtime.doc.get_element_by_id("box").unwrap();
        let nb = elem.borrow();
        if let NodeData::Element(el) = &nb.data {
            assert_eq!(el.get_attribute("data-found"), Some("yes"));
        } else {
            panic!("expected element");
        }
    }

    #[test]
    fn test_fetch_binding() {
        let doc = Document::new();
        let mut runtime = JsRuntime::new(doc, None);

        let script = r##"
            let p = fetch('data:text/plain,HelloFetch');
            p.then(function(res) {
                console.log('Fetch status:', res.status);
            });
        "##;
        let res = runtime.execute(script);
        assert!(res.is_ok());
    }
}
