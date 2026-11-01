use crate::selector::{Combinator, ComplexSelector, CompoundSelector, SimpleSelector};
use crate::stylesheet::{CssValue, Declaration, MediaRule, MediaQuery, Rule, Stylesheet, Unit};

pub struct CssParser<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> CssParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }

    pub fn parse(input: &'a str) -> Stylesheet {
        Self::new(input).parse_stylesheet()
    }

    pub fn parse_stylesheet(&mut self) -> Stylesheet {
        let mut stylesheet = Stylesheet::default();

        while self.peek_char().is_some() {
            self.consume_whitespace_and_comments();
            if self.peek_char().is_none() {
                break;
            }

            if self.peek_char() == Some('@') {
                if let Some(media_rule) = self.parse_at_rule() {
                    stylesheet.media_rules.push(media_rule);
                }
            } else if let Some(rule) = self.parse_rule() {
                stylesheet.rules.push(rule);
            } else {
                self.consume_char();
            }
        }

        stylesheet
    }

    fn parse_at_rule(&mut self) -> Option<MediaRule> {
        self.consume_char(); // consume '@'
        let name = self.consume_ident();
        if name.eq_ignore_ascii_case("media") {
            self.consume_whitespace_and_comments();
            let mut query_str = String::new();
            while let Some(c) = self.peek_char() {
                if c == '{' {
                    break;
                }
                query_str.push(self.consume_char().unwrap());
            }

            let query = Self::parse_media_query(&query_str);

            if self.peek_char() == Some('{') {
                self.consume_char();
            }

            let mut rules = Vec::new();
            while let Some(c) = self.peek_char() {
                self.consume_whitespace_and_comments();
                if c == '}' {
                    self.consume_char();
                    break;
                }
                if let Some(rule) = self.parse_rule() {
                    rules.push(rule);
                } else if self.peek_char() == Some('}') {
                    self.consume_char();
                    break;
                } else {
                    self.consume_char();
                }
            }

            Some(MediaRule { query, rules })
        } else {
            while let Some(c) = self.consume_char() {
                if c == ';' {
                    break;
                }
                if c == '{' {
                    self.skip_balanced_block();
                    break;
                }
            }
            None
        }
    }

    fn parse_media_query(q: &str) -> MediaQuery {
        let mut min_width = None;
        let mut max_width = None;

        for part in q.split("and") {
            let part = part.trim().trim_matches(|c| c == '(' || c == ')');
            if let Some((k, v)) = part.split_once(':') {
                let k = k.trim().to_ascii_lowercase();
                let v = v.trim();
                let val_px = if let Some(stripped) = v.strip_suffix("px") {
                    stripped.trim().parse::<f32>().ok()
                } else {
                    v.parse::<f32>().ok()
                };

                if k == "max-width" {
                    max_width = val_px;
                } else if k == "min-width" {
                    min_width = val_px;
                }
            }
        }

        MediaQuery { max_width, min_width }
    }

    fn skip_balanced_block(&mut self) {
        let mut depth = 1;
        while let Some(c) = self.consume_char() {
            if c == '{' {
                depth += 1;
            } else if c == '}' {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
        }
    }

    fn parse_rule(&mut self) -> Option<Rule> {
        self.consume_whitespace_and_comments();
        let mut selectors_str = String::new();
        while let Some(c) = self.peek_char() {
            if c == '{' || c == '}' {
                break;
            }
            selectors_str.push(self.consume_char().unwrap());
        }

        if self.peek_char() != Some('{') {
            return None;
        }
        self.consume_char();

        let selectors = Self::parse_selectors(&selectors_str);
        let declarations = self.parse_declarations();

        if self.peek_char() == Some('}') {
            self.consume_char();
        }

        Some(Rule {
            selectors,
            declarations,
        })
    }

    pub fn parse_selectors(input: &str) -> Vec<ComplexSelector> {
        let mut selectors = Vec::new();
        for part in input.split(',') {
            let part = part.trim();
            if !part.is_empty() {
                if let Some(sel) = Self::parse_complex_selector(part) {
                    selectors.push(sel);
                }
            }
        }
        selectors
    }

    fn parse_complex_selector(input: &str) -> Option<ComplexSelector> {
        let input = input.trim();
        if input.is_empty() {
            return None;
        }

        let mut parts: Vec<(Combinator, String)> = Vec::new();
        let mut curr = String::new();
        let mut last_comb = Combinator::Descendant;
        let mut chars = input.chars().peekable();

        while let Some(&c) = chars.peek() {
            if c == '>' {
                chars.next();
                if !curr.trim().is_empty() {
                    parts.push((last_comb, curr.trim().to_string()));
                    curr.clear();
                }
                last_comb = Combinator::Child;
            } else if c.is_whitespace() {
                while let Some(&wc) = chars.peek() {
                    if wc.is_whitespace() { chars.next(); } else { break; }
                }
                if chars.peek() == Some(&'>') {
                    chars.next();
                    if !curr.trim().is_empty() {
                        parts.push((last_comb, curr.trim().to_string()));
                        curr.clear();
                    }
                    last_comb = Combinator::Child;
                } else if !curr.trim().is_empty() {
                    parts.push((last_comb, curr.trim().to_string()));
                    curr.clear();
                    last_comb = Combinator::Descendant;
                }
            } else {
                curr.push(chars.next().unwrap());
            }
        }
        if !curr.trim().is_empty() {
            parts.push((last_comb, curr.trim().to_string()));
        }

        if parts.is_empty() {
            return None;
        }

        let mut target_selector: Option<ComplexSelector> = None;
        for (_i, (comb, part_str)) in parts.iter().enumerate().rev() {
            let compound = Self::parse_compound_selector(part_str)?;
            if target_selector.is_none() {
                target_selector = Some(ComplexSelector {
                    compound,
                    next: None,
                });
            } else {
                let comb_to_use = comb.clone();
                let prev = target_selector.take().unwrap();
                target_selector = Some(ComplexSelector {
                    compound: prev.compound,
                    next: Some((comb_to_use, Box::new(ComplexSelector {
                        compound,
                        next: prev.next,
                    }))),
                });
            }
        }

        target_selector
    }

    fn parse_compound_selector(input: &str) -> Option<CompoundSelector> {
        let mut simple_selectors = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(&c) = chars.peek() {
            if c == '*' {
                chars.next();
                simple_selectors.push(SimpleSelector::Universal);
            } else if c == '.' {
                chars.next();
                let mut name = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '-' || ch == '_' {
                        name.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if !name.is_empty() {
                    simple_selectors.push(SimpleSelector::Class(name));
                }
            } else if c == '#' {
                chars.next();
                let mut id = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '-' || ch == '_' {
                        id.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if !id.is_empty() {
                    simple_selectors.push(SimpleSelector::Id(id));
                }
            } else if c == '[' {
                chars.next();
                let mut inner = String::new();
                while let Some(ch) = chars.next() {
                    if ch == ']' {
                        break;
                    }
                    inner.push(ch);
                }
                let (name, value) = if let Some((k, v)) = inner.split_once('=') {
                    (k.trim().to_string(), Some(v.trim().trim_matches(|q| q == '"' || q == '\'').to_string()))
                } else {
                    (inner.trim().to_string(), None)
                };
                simple_selectors.push(SimpleSelector::Attribute { name, value });
            } else if c == ':' {
                chars.next();
                let mut pseudo = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '-' || ch == '_' {
                        pseudo.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if !pseudo.is_empty() {
                    simple_selectors.push(SimpleSelector::PseudoClass(pseudo));
                }
            } else if c.is_alphabetic() {
                let mut tag = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '-' || ch == '_' {
                        tag.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                simple_selectors.push(SimpleSelector::Tag(tag.to_ascii_lowercase()));
            } else {
                chars.next();
            }
        }

        if simple_selectors.is_empty() {
            None
        } else {
            Some(CompoundSelector { simple_selectors })
        }
    }

    pub fn parse_declarations(&mut self) -> Vec<Declaration> {
        let mut decls = Vec::new();
        loop {
            self.consume_whitespace_and_comments();
            match self.peek_char() {
                None | Some('}') => break,
                _ => {}
            }

            let mut prop = String::new();
            while let Some(ch) = self.peek_char() {
                if ch == ':' || ch == '}' || ch == ';' {
                    break;
                }
                prop.push(self.consume_char().unwrap());
            }

            if self.peek_char() != Some(':') {
                while let Some(ch) = self.consume_char() {
                    if ch == ';' || ch == '}' {
                        break;
                    }
                }
                continue;
            }
            self.consume_char();

            let mut val_str = String::new();
            while let Some(ch) = self.peek_char() {
                if ch == ';' || ch == '}' {
                    break;
                }
                val_str.push(self.consume_char().unwrap());
            }

            if self.peek_char() == Some(';') {
                self.consume_char();
            }

            let mut val_trim = val_str.trim();
            let mut important = false;
            if val_trim.ends_with("!important") {
                val_trim = val_trim.trim_end_matches("!important").trim();
                important = true;
            }

            let property = prop.trim().to_ascii_lowercase();
            if !property.is_empty() && !val_trim.is_empty() {
                let value = Self::parse_value(val_trim);
                decls.push(Declaration {
                    property,
                    value,
                    important,
                });
            }
        }
        decls
    }

    pub fn parse_value(raw: &str) -> CssValue {
        let s = raw.trim();

        if s.starts_with('#') {
            if let Some(color) = Self::parse_hex_color(&s[1..]) {
                return color;
            }
        }

        if s.starts_with("rgb(") || s.starts_with("rgba(") {
            if let Some(color) = Self::parse_rgba_function(s) {
                return color;
            }
        }

        if s.starts_with("url(") && s.ends_with(')') {
            let inner = &s[4..s.len() - 1].trim().trim_matches(|c| c == '"' || c == '\'');
            return CssValue::Url(inner.to_string());
        }

        if let Some(color) = Self::parse_named_color(s) {
            return color;
        }

        if let Some(val) = Self::parse_length(s) {
            return val;
        }

        if let Ok(num) = s.parse::<f32>() {
            return CssValue::Number(num);
        }

        if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
            return CssValue::String(s[1..s.len() - 1].to_string());
        }

        CssValue::Keyword(s.to_ascii_lowercase())
    }

    fn parse_length(s: &str) -> Option<CssValue> {
        let (num_part, unit) = if let Some(n) = s.strip_suffix("px") {
            (n, Unit::Px)
        } else if let Some(n) = s.strip_suffix("rem") {
            (n, Unit::Rem)
        } else if let Some(n) = s.strip_suffix("em") {
            (n, Unit::Em)
        } else if let Some(n) = s.strip_suffix('%') {
            (n, Unit::Percent)
        } else if let Some(n) = s.strip_suffix("vw") {
            (n, Unit::Vw)
        } else if let Some(n) = s.strip_suffix("vh") {
            (n, Unit::Vh)
        } else if let Some(n) = s.strip_suffix("pt") {
            (n, Unit::Pt)
        } else {
            return None;
        };

        let num = num_part.trim().parse::<f32>().ok()?;
        Some(CssValue::Length(num, unit))
    }

    fn parse_hex_color(hex: &str) -> Option<CssValue> {
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                Some(CssValue::Color(r, g, b, 255))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(CssValue::Color(r, g, b, 255))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(CssValue::Color(r, g, b, a))
            }
            _ => None,
        }
    }

    fn parse_rgba_function(s: &str) -> Option<CssValue> {
        let start = s.find('(')? + 1;
        let end = s.rfind(')')?;
        let inner = &s[start..end];
        let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
        if parts.len() == 3 || parts.len() == 4 {
            let r = parts[0].parse::<u8>().ok()?;
            let g = parts[1].parse::<u8>().ok()?;
            let b = parts[2].parse::<u8>().ok()?;
            let a = if parts.len() == 4 {
                if let Ok(alpha_float) = parts[3].parse::<f32>() {
                    (alpha_float * 255.0).clamp(0.0, 255.0) as u8
                } else {
                    parts[3].parse::<u8>().unwrap_or(255)
                }
            } else {
                255
            };
            Some(CssValue::Color(r, g, b, a))
        } else {
            None
        }
    }

    fn parse_named_color(name: &str) -> Option<CssValue> {
        match name.to_ascii_lowercase().as_str() {
            "black" => Some(CssValue::Color(0, 0, 0, 255)),
            "white" => Some(CssValue::Color(255, 255, 255, 255)),
            "red" => Some(CssValue::Color(255, 0, 0, 255)),
            "green" => Some(CssValue::Color(0, 128, 0, 255)),
            "blue" => Some(CssValue::Color(0, 0, 255, 255)),
            "yellow" => Some(CssValue::Color(255, 255, 0, 255)),
            "cyan" => Some(CssValue::Color(0, 255, 255, 255)),
            "magenta" => Some(CssValue::Color(255, 0, 255, 255)),
            "gray" | "grey" => Some(CssValue::Color(128, 128, 128, 255)),
            "lightgray" | "lightgrey" => Some(CssValue::Color(211, 211, 211, 255)),
            "darkgray" | "darkgrey" => Some(CssValue::Color(169, 169, 169, 255)),
            "orange" => Some(CssValue::Color(255, 165, 0, 255)),
            "purple" => Some(CssValue::Color(128, 0, 128, 255)),
            "navy" => Some(CssValue::Color(0, 0, 128, 255)),
            "teal" => Some(CssValue::Color(0, 128, 128, 255)),
            "silver" => Some(CssValue::Color(192, 192, 192, 255)),
            "transparent" => Some(CssValue::Color(0, 0, 0, 0)),
            _ => None,
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn consume_char(&mut self) -> Option<char> {
        self.chars.next()
    }

    fn consume_ident(&mut self) -> String {
        let mut ident = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                ident.push(self.chars.next().unwrap());
            } else {
                break;
            }
        }
        ident
    }

    fn consume_whitespace_and_comments(&mut self) {
        loop {
            while let Some(&c) = self.chars.peek() {
                if c.is_whitespace() {
                    self.chars.next();
                } else {
                    break;
                }
            }

            if self.chars.peek() == Some(&'/') {
                let mut lookahead = self.chars.clone();
                lookahead.next();
                if lookahead.peek() == Some(&'*') {
                    self.chars.next();
                    self.chars.next();
                    while let Some(c) = self.chars.next() {
                        if c == '*' && self.chars.peek() == Some(&'/') {
                            self.chars.next();
                            break;
                        }
                    }
                    continue;
                }
            }
            break;
        }
    }
}
