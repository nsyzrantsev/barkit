use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("UTF-8 error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),
    #[error("UMI capture group not found in your pattern")]
    UMIPatternNotFound,
    #[error("Both reads did not match")]
    BothReadsNotMatch,
    #[error("Failed to read a file: {0}")]
    FileReadError(#[from] std::io::Error),
    #[error("No match")]
    PatternNotMatched
}

impl Clone for Error {
    fn clone(&self) -> Self {
        match self {
            Error::Utf8Error(err) => Error::Utf8Error(*err),
            Error::FromUtf8Error(err) => Error::FromUtf8Error(err.clone()),
            Error::RegexError(err) => Error::RegexError(err.clone()),
            Error::UMIPatternNotFound => Error::UMIPatternNotFound,
            Error::BothReadsNotMatch => Error::BothReadsNotMatch,
            Error::FileReadError(err) => Error::FileReadError(err.kind().clone().into()),
            Error::PatternNotMatched => Error::PatternNotMatched
        }
    }
}