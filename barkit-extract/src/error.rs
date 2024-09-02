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
    #[error("Provided unexpected barcode capture group {0}")]
    UnexpectedCaptureGroupName(String),
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),
    #[error("No match")]
    PatternNotMatched,
    #[error("Fancy regex error: {0}")]
    FancyRegex(#[from] fancy_regex::Error),
    #[error("Failed to choose permutation mask")]
    PermutationMaskSize,
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
            Error::UnexpectedCaptureGroupName(capture_group) => {
                Error::UnexpectedCaptureGroupName(capture_group.clone())
            }
            Error::IO(err) => Error::IO(err.kind().into()),
            Error::PatternNotMatched => Error::PatternNotMatched,
            Error::FancyRegex(err) => Error::FancyRegex(err.clone()),
            Error::PermutationMaskSize => Error::PermutationMaskSize,
        }
    }
}
