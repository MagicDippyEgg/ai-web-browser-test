use crate::selector::ComplexSelector;

#[derive(Debug, Clone, PartialEq)]
pub enum CssValue {
    Keyword(String),
    Color(u8, u8, u8, u8),
    Length(f32, Unit),
    Number(f32),
    String(String),
    Url(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Px,
    Em,
    Rem,
    Percent,
    Vw,
    Vh,
    Pt,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub property: String,
    pub value: CssValue,
    pub important: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub selectors: Vec<ComplexSelector>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MediaQuery {
    pub max_width: Option<f32>,
    pub min_width: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MediaRule {
    pub query: MediaQuery,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Default)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
    pub media_rules: Vec<MediaRule>,
}
