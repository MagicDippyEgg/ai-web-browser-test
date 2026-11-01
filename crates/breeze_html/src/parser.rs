use crate::dom::{Document, Node, NodeData, NodeRef};
use std::collections::HashMap;
use std::rc::Rc;

const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input",
    "link", "meta", "param", "source", "track", "wbr",
];

const RAW_TEXT_ELEMENTS: &[&str] = &["script", "style"];

pub struct HtmlParser<'a> {
    input: &'a str,
}

impl<'a> HtmlParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input }
    }

    pub fn parse(input: &'a str) -> Document {
        Self::new(input).parse_document()
    }

    pub fn parse_document(&mut self) -> Document {
        let doc = Document::new();
        let mut stack: Vec<NodeRef> = vec![Rc::clone(&doc.root)];
        let mut chars = self.input.chars().peekable();

        let mut current_text = String::new();

        while let Some(&c) = chars.peek() {
            if c == '<' {
                if !current_text.is_empty() {
                    let parent = stack.last().unwrap();
                    let text_node = Node::new_text(&current_text);
                    Document::append_child(parent, &text_node);
                    current_text.clear();
                }

                chars.next(); // consume '<'

                if chars.clone().take(3).collect::<String>() == "!--" {
                    chars.next(); chars.next(); chars.next();
                    let mut comment = String::new();
                    while let Some(ch) = chars.next() {
                        if ch == '-' && chars.clone().take(2).collect::<String>() == "->" {
                            chars.next(); chars.next();
                            break;
                        }
                        comment.push(ch);
                    }
                    let parent = stack.last().unwrap();
                    Document::append_child(parent, &Node::new_comment(&comment));
                    continue;
                }

                let peek_7: String = chars.clone().take(7).collect();
                if peek_7.eq_ignore_ascii_case("doctype") {
                    for _ in 0..7 { chars.next(); }
                    let mut doctype_str = String::new();
                    while let Some(ch) = chars.next() {
                        if ch == '>' { break; }
                        doctype_str.push(ch);
                    }
                    let parent = stack.last().unwrap();
                    Document::append_child(parent, &Node::new(NodeData::DocumentType {
                        name: doctype_str.trim().to_string(),
                        public_id: None,
                        system_id: None,
                    }));
                    continue;
                }

                if let Some('/') = chars.peek() {
                    chars.next(); // consume '/'
                    let mut tag_name = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch == '>' || ch.is_whitespace() {
                            break;
                        }
                        tag_name.push(chars.next().unwrap());
                    }
                    while let Some(ch) = chars.next() {
                        if ch == '>' { break; }
                    }

                    let tag_name = tag_name.to_ascii_lowercase();
                    if let Some(pos) = stack.iter().rposition(|node| {
                        if let NodeData::Element(el) = &node.borrow().data {
                            el.tag_name == tag_name
                        } else {
                            false
                        }
                    }) {
                        if pos > 0 {
                            stack.truncate(pos);
                        }
                    }
                    continue;
                }

                let mut tag_name = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_whitespace() || ch == '>' || ch == '/' {
                        break;
                    }
                    tag_name.push(chars.next().unwrap());
                }

                if tag_name.is_empty() {
                    current_text.push('<');
                    continue;
                }

                let tag_name = tag_name.to_ascii_lowercase();
                let mut attributes = HashMap::new();
                let mut self_closing = VOID_ELEMENTS.contains(&tag_name.as_str());

                loop {
                    while let Some(&ch) = chars.peek() {
                        if ch.is_whitespace() { chars.next(); } else { break; }
                    }

                    match chars.peek() {
                        None => break,
                        Some('/') => {
                            chars.next();
                            while let Some(&ch) = chars.peek() {
                                if ch.is_whitespace() { chars.next(); } else { break; }
                            }
                            if let Some('>') = chars.peek() {
                                chars.next();
                                self_closing = true;
                                break;
                            }
                        }
                        Some('>') => {
                            chars.next();
                            break;
                        }
                        _ => {
                            let mut attr_name = String::new();
                            while let Some(&ch) = chars.peek() {
                                if ch.is_whitespace() || ch == '=' || ch == '/' || ch == '>' {
                                    break;
                                }
                                attr_name.push(chars.next().unwrap());
                            }

                            if attr_name.is_empty() {
                                chars.next();
                                continue;
                            }

                            while let Some(&ch) = chars.peek() {
                                if ch.is_whitespace() { chars.next(); } else { break; }
                            }

                            let attr_value = if let Some('=') = chars.peek() {
                                chars.next();
                                while let Some(&ch) = chars.peek() {
                                    if ch.is_whitespace() { chars.next(); } else { break; }
                                }
                                match chars.peek() {
                                    Some(&q) if q == '"' || q == '\'' => {
                                        chars.next();
                                        let mut val = String::new();
                                        while let Some(ch) = chars.next() {
                                            if ch == q { break; }
                                            if ch == '&' {
                                                val.push(consume_entity_from_iter(&mut chars));
                                            } else {
                                                val.push(ch);
                                            }
                                        }
                                        val
                                    }
                                    _ => {
                                        let mut val = String::new();
                                        while let Some(&ch) = chars.peek() {
                                            if ch.is_whitespace() || ch == '>' { break; }
                                            chars.next();
                                            if ch == '&' {
                                                val.push(consume_entity_from_iter(&mut chars));
                                            } else {
                                                val.push(ch);
                                            }
                                        }
                                        val
                                    }
                                }
                            } else {
                                String::new()
                            };

                            attributes.insert(attr_name.to_ascii_lowercase(), attr_value);
                        }
                    }
                }

                let elem_node = Node::new_element(&tag_name, attributes);
                let parent = stack.last().unwrap();
                Document::append_child(parent, &elem_node);

                if RAW_TEXT_ELEMENTS.contains(&tag_name.as_str()) && !self_closing {
                    let end_token = format!("</{}", tag_name);
                    let mut raw_content = String::new();
                    while let Some(ch) = chars.next() {
                        if ch == '<' {
                            let mut peek_chars = chars.clone();
                            let mut buf = String::from("<");
                            for _ in 0..end_token.len() - 1 {
                                if let Some(pc) = peek_chars.next() {
                                    buf.push(pc);
                                }
                            }
                            if buf.eq_ignore_ascii_case(&end_token) {
                                for _ in 0..end_token.len() - 1 {
                                    chars.next();
                                }
                                while let Some(close_ch) = chars.next() {
                                    if close_ch == '>' { break; }
                                }
                                break;
                            } else {
                                raw_content.push('<');
                            }
                        } else {
                            raw_content.push(ch);
                        }
                    }
                    if !raw_content.is_empty() {
                        Document::append_child(&elem_node, &Node::new_text(&raw_content));
                    }
                } else if !self_closing {
                    stack.push(elem_node);
                }
            } else if c == '&' {
                chars.next();
                let entity = consume_entity_from_iter(&mut chars);
                current_text.push(entity);
            } else {
                chars.next();
                current_text.push(c);
            }
        }

        if !current_text.is_empty() {
            let parent = stack.last().unwrap();
            Document::append_child(parent, &Node::new_text(&current_text));
        }

        doc
    }
}

fn consume_entity_from_iter<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) -> char {
    let mut entity_name = String::new();
    while let Some(&c) = chars.peek() {
        if c == ';' {
            chars.next();
            break;
        } else if c.is_alphanumeric() || c == '#' {
            entity_name.push(chars.next().unwrap());
        } else {
            break;
        }
    }

    match entity_name.as_str() {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => '\u{00A0}',
        "copy" => '©',
        "reg" => '®',
        s if s.starts_with("#x") || s.starts_with("#X") => {
            u32::from_str_radix(&s[2..], 16)
                .ok()
                .and_then(char::from_u32)
                .unwrap_or('&')
        }
        s if s.starts_with('#') => {
            s[1..].parse::<u32>()
                .ok()
                .and_then(char::from_u32)
                .unwrap_or('&')
        }
        _ => '&',
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_html() {
        let html = r#"
            <!DOCTYPE html>
            <html>
                <head><title>Test Page</title></head>
                <body>
                    <!-- A comment -->
                    <h1 id="header" class="main heading">Hello &amp; Welcome</h1>
                    <p>Paragraph with <a href="https://example.com">link</a> and <img src="pic.png" /> image.</p>
                </body>
            </html>
        "#;

        let doc = HtmlParser::parse(html);
        let header = doc.get_element_by_id("header").expect("header found");
        assert_eq!(Document::text_content(&header), "Hello & Welcome");
    }
}
