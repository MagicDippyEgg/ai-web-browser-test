#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Doctype {
        name: String,
        public_id: Option<String>,
        system_id: Option<String>,
    },
    StartTag {
        name: String,
        attributes: Vec<(String, String)>,
        self_closing: bool,
    },
    EndTag {
        name: String,
    },
    Comment(String),
    Character(char),
    EndOfFile,
}

pub struct HtmlTokenizer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> HtmlTokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }
}
