use thiserror::Error;

#[derive(Error, Debug)]
pub enum BreezeError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("HTML parse error: {0}")]
    HtmlParse(String),
    #[error("CSS parse error: {0}")]
    CssParse(String),
    #[error("JS execution error: {0}")]
    JsError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Generic error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, BreezeError>;
