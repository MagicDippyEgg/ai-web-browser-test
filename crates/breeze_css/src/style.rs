use crate::parser::CssParser;
use crate::selector::{Combinator, ComplexSelector, CompoundSelector, SimpleSelector, Specificity};
use crate::stylesheet::{CssValue, Declaration, Stylesheet, Unit};
use breeze_html::dom::{NodeData, NodeRef};
use std::rc::Rc;

pub const DEFAULT_USER_AGENT_CSS: &str = r#"
head, script, style, title, meta, link {
    display: none;
}
html, body {
    margin: 8px;
    padding: 0px;
    font-family: sans-serif;
    font-size: 16px;
    color: #000000;
    background-color: #ffffff;
    display: block;
}
div, section, article, nav, header, footer, main, form {
    display: block;
    margin: 0px;
    padding: 0px;
}
p {
    display: block;
    margin-top: 16px;
    margin-bottom: 16px;
}
h1 {
    display: block;
    font-size: 32px;
    font-weight: bold;
    margin-top: 21px;
    margin-bottom: 21px;
}
h2 {
    display: block;
    font-size: 24px;
    font-weight: bold;
    margin-top: 19px;
    margin-bottom: 19px;
}
h3 {
    display: block;
    font-size: 18px;
    font-weight: bold;
    margin-top: 18px;
    margin-bottom: 18px;
}
h4 {
    display: block;
    font-size: 16px;
    font-weight: bold;
    margin-top: 21px;
    margin-bottom: 21px;
}
ul, ol {
    display: block;
    margin-top: 16px;
    margin-bottom: 16px;
    padding-left: 40px;
}
li {
    display: block;
    margin-top: 4px;
    margin-bottom: 4px;
}
a {
    color: #0000ee;
    text-decoration: underline;
    display: inline;
}
span, b, strong, i, em, u {
    display: inline;
}
b, strong {
    font-weight: bold;
}
i, em {
    font-style: italic;
}
button, input, textarea, select {
    display: inline-block;
    font-family: inherit;
    font-size: 14px;
    margin: 2px;
    border: 1px solid #767676;
}
button {
    background-color: #f0f0f0;
    padding: 4px 8px;
    cursor: pointer;
}
input[type=text], textarea {
    background-color: #ffffff;
    padding: 4px;
}
img {
    display: inline-block;
}
table {
    display: block;
    margin: 0px;
}
tr {
    display: flex;
}
td, th {
    display: block;
    padding: 4px 8px;
    border: 1px solid #cccccc;
}
th {
    font-weight: bold;
    background-color: #f2f2f2;
}
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Display {
    Block,
    Inline,
    InlineBlock,
    Flex,
    Grid,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    Static,
    Relative,
    Absolute,
    Fixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    Column,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignItems {
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxSizing {
    ContentBox,
    BorderBox,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComputedStyle {
    pub display: Display,
    pub position: Position,
    pub box_sizing: BoxSizing,
    pub color: [u8; 4],
    pub background_color: [u8; 4],
    pub font_size: f32,
    pub font_weight: String,
    pub font_style: String,
    pub font_family: String,
    pub text_align: String,
    pub text_decoration: String,

    pub width: Option<f32>,
    pub height: Option<f32>,
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,

    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub left: Option<f32>,
    pub z_index: i32,

    pub margin_top: f32,
    pub margin_right: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,

    pub padding_top: f32,
    pub padding_right: f32,
    pub padding_bottom: f32,
    pub padding_left: f32,

    pub border_top_width: f32,
    pub border_right_width: f32,
    pub border_bottom_width: f32,
    pub border_left_width: f32,
    pub border_color: [u8; 4],
    pub border_radius: f32,

    pub flex_direction: FlexDirection,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub gap: f32,

    pub grid_template_columns: Vec<f32>,
    pub grid_gap: f32,

    pub opacity: f32,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            display: Display::Inline,
            position: Position::Static,
            box_sizing: BoxSizing::ContentBox,
            color: [0, 0, 0, 255],
            background_color: [0, 0, 0, 0],
            font_size: 16.0,
            font_weight: "normal".to_string(),
            font_style: "normal".to_string(),
            font_family: "sans-serif".to_string(),
            text_align: "left".to_string(),
            text_decoration: "none".to_string(),

            width: None,
            height: None,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,

            top: None,
            right: None,
            bottom: None,
            left: None,
            z_index: 0,

            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,

            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,

            border_top_width: 0.0,
            border_right_width: 0.0,
            border_bottom_width: 0.0,
            border_left_width: 0.0,
            border_color: [0, 0, 0, 255],
            border_radius: 0.0,

            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Stretch,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            gap: 0.0,

            grid_template_columns: Vec::new(),
            grid_gap: 0.0,

            opacity: 1.0,
        }
    }
}

pub struct StyleEngine {
    stylesheets: Vec<Stylesheet>,
    viewport_width: f32,
    viewport_height: f32,
}

impl StyleEngine {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        let ua_sheet = CssParser::parse(DEFAULT_USER_AGENT_CSS);
        Self {
            stylesheets: vec![ua_sheet],
            viewport_width,
            viewport_height,
        }
    }

    pub fn add_stylesheet(&mut self, sheet: Stylesheet) {
        self.stylesheets.push(sheet);
    }

    pub fn compute_style(&self, node: &NodeRef, parent_style: Option<&ComputedStyle>) -> ComputedStyle {
        let mut style = if let Some(parent) = parent_style {
            Self::inherit_properties(parent)
        } else {
            ComputedStyle::default()
        };

        let node_borrow = node.borrow();
        let (_elem_tag, inline_style_str) = match &node_borrow.data {
            NodeData::Element(el) => (Some(&el.tag_name), el.get_attribute("style")),
            NodeData::Text { .. } => {
                return style;
            }
            _ => return style,
        };

        let mut matched_decls: Vec<(Specificity, usize, Declaration)> = Vec::new();
        let mut rule_index = 0;

        for sheet in &self.stylesheets {
            for rule in &sheet.rules {
                rule_index += 1;
                for sel in &rule.selectors {
                    if Self::matches_complex_selector(node, sel) {
                        let spec = sel.specificity();
                        for decl in &rule.declarations {
                            matched_decls.push((spec, rule_index, decl.clone()));
                        }
                    }
                }
            }

            for mrule in &sheet.media_rules {
                if self.matches_media_query(&mrule.query) {
                    for rule in &mrule.rules {
                        rule_index += 1;
                        for sel in &rule.selectors {
                            if Self::matches_complex_selector(node, sel) {
                                let spec = sel.specificity();
                                for decl in &rule.declarations {
                                    matched_decls.push((spec, rule_index, decl.clone()));
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(inline_str) = inline_style_str {
            let mut parser = CssParser::new(inline_str);
            let inline_decls = parser.parse_declarations();
            let inline_spec = Specificity::new(1, 0, 0, 0);
            for decl in inline_decls {
                rule_index += 1;
                matched_decls.push((inline_spec, rule_index, decl));
            }
        }

        matched_decls.sort_by(|(spec_a, idx_a, decl_a), (spec_b, idx_b, decl_b)| {
            if decl_a.important != decl_b.important {
                return decl_a.important.cmp(&decl_b.important);
            }
            if spec_a != spec_b {
                return spec_a.cmp(spec_b);
            }
            idx_a.cmp(idx_b)
        });

        for (_, _, decl) in matched_decls {
            Self::apply_declaration(&mut style, &decl, self.viewport_width, self.viewport_height);
        }

        style
    }

    fn matches_media_query(&self, q: &crate::stylesheet::MediaQuery) -> bool {
        if let Some(max_w) = q.max_width {
            if self.viewport_width > max_w {
                return false;
            }
        }
        if let Some(min_w) = q.min_width {
            if self.viewport_width < min_w {
                return false;
            }
        }
        true
    }

    fn inherit_properties(parent: &ComputedStyle) -> ComputedStyle {
        let mut child = ComputedStyle::default();
        child.color = parent.color;
        child.font_size = parent.font_size;
        child.font_family = parent.font_family.clone();
        child.font_weight = parent.font_weight.clone();
        child.font_style = parent.font_style.clone();
        child.text_align = parent.text_align.clone();
        child
    }

    fn matches_complex_selector(node: &NodeRef, sel: &ComplexSelector) -> bool {
        if !Self::matches_compound(node, &sel.compound) {
            return false;
        }

        if let Some((combinator, next_sel)) = &sel.next {
            match combinator {
                Combinator::Child => {
                    if let Some(parent_weak) = &node.borrow().parent {
                        if let Some(parent) = parent_weak.upgrade() {
                            return Self::matches_complex_selector(&parent, next_sel);
                        }
                    }
                    false
                }
                Combinator::Descendant => {
                    let mut curr_parent = node.borrow().parent.as_ref().and_then(|p| p.upgrade());
                    while let Some(parent) = curr_parent {
                        if Self::matches_complex_selector(&parent, next_sel) {
                            return true;
                        }
                        curr_parent = parent.borrow().parent.as_ref().and_then(|p| p.upgrade());
                    }
                    false
                }
            }
        } else {
            true
        }
    }

    fn matches_compound(node: &NodeRef, compound: &CompoundSelector) -> bool {
        let node_borrow = node.borrow();
        let elem = match &node_borrow.data {
            NodeData::Element(el) => el,
            _ => return false,
        };

        for simple in &compound.simple_selectors {
            match simple {
                SimpleSelector::Universal => {}
                SimpleSelector::Tag(tag) => {
                    if elem.tag_name != *tag {
                        return false;
                    }
                }
                SimpleSelector::Id(id) => {
                    if elem.id.as_deref() != Some(id) {
                        return false;
                    }
                }
                SimpleSelector::Class(cls) => {
                    if !elem.has_class(cls) {
                        return false;
                    }
                }
                SimpleSelector::Attribute { name, value } => {
                    match elem.get_attribute(name) {
                        None => return false,
                        Some(val) => {
                            if let Some(expected) = value {
                                if val != expected {
                                    return false;
                                }
                            }
                        }
                    }
                }
                SimpleSelector::PseudoClass(pseudo) => {
                    if pseudo == "first-child" {
                        if let Some(p_weak) = &node_borrow.parent {
                            if let Some(parent) = p_weak.upgrade() {
                                if let Some(first) = parent.borrow().children.first() {
                                    if !Rc::ptr_eq(node, first) {
                                        return false;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        true
    }

    fn apply_declaration(style: &mut ComputedStyle, decl: &Declaration, vp_w: f32, vp_h: f32) {
        let resolve_len = |val: &CssValue, current_font_size: f32| -> Option<f32> {
            match val {
                CssValue::Length(num, unit) => match unit {
                    Unit::Px | Unit::Pt => Some(*num),
                    Unit::Em | Unit::Rem => Some(*num * current_font_size),
                    Unit::Percent => Some(*num * 0.01 * vp_w),
                    Unit::Vw => Some(*num * 0.01 * vp_w),
                    Unit::Vh => Some(*num * 0.01 * vp_h),
                },
                CssValue::Number(num) => Some(*num),
                _ => None,
            }
        };

        let resolve_color = |val: &CssValue| -> Option<[u8; 4]> {
            match val {
                CssValue::Color(r, g, b, a) => Some([*r, *g, *b, *a]),
                _ => None,
            }
        };

        let val = &decl.value;
        match decl.property.as_str() {
            "display" => {
                if let CssValue::Keyword(k) = val {
                    style.display = match k.as_str() {
                        "block" => Display::Block,
                        "inline" => Display::Inline,
                        "inline-block" => Display::InlineBlock,
                        "flex" => Display::Flex,
                        "grid" => Display::Grid,
                        "none" => Display::None,
                        _ => style.display,
                    };
                }
            }
            "position" => {
                if let CssValue::Keyword(k) = val {
                    style.position = match k.as_str() {
                        "relative" => Position::Relative,
                        "absolute" => Position::Absolute,
                        "fixed" => Position::Fixed,
                        _ => Position::Static,
                    };
                }
            }
            "box-sizing" => {
                if let CssValue::Keyword(k) = val {
                    style.box_sizing = match k.as_str() {
                        "border-box" => BoxSizing::BorderBox,
                        _ => BoxSizing::ContentBox,
                    };
                }
            }
            "color" => {
                if let Some(col) = resolve_color(val) {
                    style.color = col;
                }
            }
            "background-color" | "background" => {
                if let Some(col) = resolve_color(val) {
                    style.background_color = col;
                }
            }
            "font-size" => {
                if let CssValue::Length(num, unit) = val {
                    style.font_size = match unit {
                        Unit::Px | Unit::Pt => *num,
                        Unit::Em | Unit::Rem => *num * style.font_size,
                        _ => *num,
                    };
                }
            }
            "font-weight" => {
                if let CssValue::Keyword(k) = val {
                    style.font_weight = k.clone();
                } else if let CssValue::Number(n) = val {
                    style.font_weight = if *n >= 600.0 { "bold".to_string() } else { "normal".to_string() };
                }
            }
            "font-style" => {
                if let CssValue::Keyword(k) = val {
                    style.font_style = k.clone();
                }
            }
            "font-family" => {
                if let CssValue::Keyword(k) = val {
                    style.font_family = k.clone();
                } else if let CssValue::String(s) = val {
                    style.font_family = s.clone();
                }
            }
            "text-align" => {
                if let CssValue::Keyword(k) = val {
                    style.text_align = k.clone();
                }
            }
            "text-decoration" => {
                if let CssValue::Keyword(k) = val {
                    style.text_decoration = k.clone();
                }
            }
            "width" => {
                if let CssValue::Length(num, unit) = val {
                    style.width = match unit {
                        Unit::Px | Unit::Pt => Some(*num),
                        Unit::Percent => Some(*num * 0.01 * vp_w),
                        Unit::Vw => Some(*num * 0.01 * vp_w),
                        _ => Some(*num),
                    };
                }
            }
            "height" => {
                if let CssValue::Length(num, unit) = val {
                    style.height = match unit {
                        Unit::Px | Unit::Pt => Some(*num),
                        Unit::Percent => Some(*num * 0.01 * vp_h),
                        Unit::Vh => Some(*num * 0.01 * vp_h),
                        _ => Some(*num),
                    };
                }
            }
            "top" => {
                if let CssValue::Length(num, _) = val { style.top = Some(*num); }
            }
            "right" => {
                if let CssValue::Length(num, _) = val { style.right = Some(*num); }
            }
            "bottom" => {
                if let CssValue::Length(num, _) = val { style.bottom = Some(*num); }
            }
            "left" => {
                if let CssValue::Length(num, _) = val { style.left = Some(*num); }
            }
            "z-index" => {
                if let CssValue::Number(num) = val {
                    style.z_index = *num as i32;
                }
            }
            "margin" => {
                if let Some(n) = resolve_len(val, style.font_size) {
                    style.margin_top = n;
                    style.margin_right = n;
                    style.margin_bottom = n;
                    style.margin_left = n;
                }
            }
            "margin-top" => {
                if let Some(n) = resolve_len(val, style.font_size) { style.margin_top = n; }
            }
            "margin-right" => {
                if let Some(n) = resolve_len(val, style.font_size) { style.margin_right = n; }
            }
            "margin-bottom" => {
                if let Some(n) = resolve_len(val, style.font_size) { style.margin_bottom = n; }
            }
            "margin-left" => {
                if let Some(n) = resolve_len(val, style.font_size) { style.margin_left = n; }
            }
            "padding" => {
                if let Some(n) = resolve_len(val, style.font_size) {
                    style.padding_top = n;
                    style.padding_right = n;
                    style.padding_bottom = n;
                    style.padding_left = n;
                }
            }
            "padding-top" => {
                if let Some(n) = resolve_len(val, style.font_size) { style.padding_top = n; }
            }
            "padding-right" => {
                if let Some(n) = resolve_len(val, style.font_size) { style.padding_right = n; }
            }
            "padding-bottom" => {
                if let Some(n) = resolve_len(val, style.font_size) { style.padding_bottom = n; }
            }
            "padding-left" => {
                if let Some(n) = resolve_len(val, style.font_size) { style.padding_left = n; }
            }
            "border" | "border-width" => {
                if let Some(n) = resolve_len(val, style.font_size) {
                    style.border_top_width = n;
                    style.border_right_width = n;
                    style.border_bottom_width = n;
                    style.border_left_width = n;
                }
            }
            "border-color" => {
                if let Some(col) = resolve_color(val) {
                    style.border_color = col;
                }
            }
            "border-radius" => {
                if let Some(n) = resolve_len(val, style.font_size) {
                    style.border_radius = n;
                }
            }
            "flex-direction" => {
                if let CssValue::Keyword(k) = val {
                    style.flex_direction = match k.as_str() {
                        "column" => FlexDirection::Column,
                        _ => FlexDirection::Row,
                    };
                }
            }
            "justify-content" => {
                if let CssValue::Keyword(k) = val {
                    style.justify_content = match k.as_str() {
                        "center" => JustifyContent::Center,
                        "flex-end" => JustifyContent::FlexEnd,
                        "space-between" => JustifyContent::SpaceBetween,
                        "space-around" => JustifyContent::SpaceAround,
                        _ => JustifyContent::FlexStart,
                    };
                }
            }
            "align-items" => {
                if let CssValue::Keyword(k) = val {
                    style.align_items = match k.as_str() {
                        "flex-start" => AlignItems::FlexStart,
                        "flex-end" => AlignItems::FlexEnd,
                        "center" => AlignItems::Center,
                        _ => AlignItems::Stretch,
                    };
                }
            }
            "flex-grow" => {
                if let CssValue::Number(num) = val {
                    style.flex_grow = *num;
                }
            }
            "flex-shrink" => {
                if let CssValue::Number(num) = val {
                    style.flex_shrink = *num;
                }
            }
            "gap" => {
                if let CssValue::Length(num, _) = val {
                    style.gap = *num;
                    style.grid_gap = *num;
                }
            }
            "grid-gap" => {
                if let CssValue::Length(num, _) = val {
                    style.grid_gap = *num;
                }
            }
            _ => {}
        }
    }
}
