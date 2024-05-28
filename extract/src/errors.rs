use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("Enexpected value: {}", _0)]
    UnexpectedValue(i32),
    #[error("Pattern does not match with {}", _0)]
    PatternNotMatched(String),
    #[error("Unknown nucleotide letter: {}", _0)]
    UnknownNucleotideLetter(char),
    #[error("Unimplemented error")]
    UnimplementedError(),
    #[error("Invalid pattern: {}", _0)]
    InvalidPattern(String),
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("UTF-8 error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
}