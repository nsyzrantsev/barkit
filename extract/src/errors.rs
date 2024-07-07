use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("UTF-8 error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("Output FASTQ file not provided")]
    OutputFastqFileNotProvided,
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),
    #[error("Capture group assignment index is out of range: {0}")]
    CaptureGroupIndexError(usize),
    #[error("Unexpected barcode capture group name: {0}")]
    UnexpectedCaptureGroupName(String),
    #[error("Both reads did not match")]
    BothReadsNotMatch,
    #[error("Tre regex error: {0}")]
    TreRegexError(#[from] fuzzy_regex::errors::TreRegexError),
    #[error("Failed to read a file: {0}")]
    FileReadError(#[from] std::io::Error),
}

impl Clone for Error {
    fn clone(&self) -> Self {
        match self {
            Error::Utf8Error(err) => Error::Utf8Error(*err),
            Error::FromUtf8Error(err) => Error::FromUtf8Error(err.clone()),
            Error::OutputFastqFileNotProvided => Error::OutputFastqFileNotProvided,
            Error::RegexError(err) => Error::RegexError(err.clone()),
            Error::CaptureGroupIndexError(idx) => Error::CaptureGroupIndexError(*idx),
            Error::UnexpectedCaptureGroupName(name) => Error::UnexpectedCaptureGroupName(name.clone()),
            Error::BothReadsNotMatch => Error::BothReadsNotMatch,
            Error::TreRegexError(err) => Error::TreRegexError(err.to_owned().clone()),
            Error::FileReadError(err) => Error::FileReadError(err.kind().clone().into()),
        }
    }
}