use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("UTF-8 error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("TreRegexError")]
    TreRegexError(#[from] fuzzy_regex::errors::TreRegexError),
    #[error("Output FASTQ file not provided")]
    OutputFastqFileNotProvided,
    #[error("RegexError")]
    RegexError(#[from] regex::Error),
    #[error("{0} capture group index does not exist.")]
    CaptureGroupIndexError(usize)
}