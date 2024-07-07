use thiserror::Error;

#[derive(Error, Debug)]
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
    #[error("CaptureGroupIndexError: capture group assignment index is out of range.")]
    CaptureGroupIndexError(usize),
    #[error("Failed to read a file")]
    FileReadError(#[from] std::io::Error)
}