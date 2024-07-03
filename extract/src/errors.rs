use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("UTF-8 error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("RegexError")]
    RegexError(#[from] tre_regex::RegexError),
}