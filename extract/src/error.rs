use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("UTF-8 error: {0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    #[error("{0} capture group not found in your pattern")]
    BarcodeCaptureGroupNotFound(String),
    #[error("Failed to read a file: {0}")]
    FileRead(#[from] std::io::Error),
    #[error("No match")]
    PatternNotMatched,
}

impl Clone for Error {
    fn clone(&self) -> Self {
        match self {
            Error::Utf8(err) => Error::Utf8(*err),
            Error::FromUtf8(err) => Error::FromUtf8(err.clone()),
            Error::Regex(err) => Error::Regex(err.clone()),
            Error::BarcodeCaptureGroupNotFound(barcode_type) => {
                Error::BarcodeCaptureGroupNotFound(barcode_type.clone())
            }
            Error::FileRead(err) => Error::FileRead(err.kind().into()),
            Error::PatternNotMatched => Error::PatternNotMatched,
        }
    }
}
