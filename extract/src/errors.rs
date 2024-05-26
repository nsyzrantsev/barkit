use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("Enexpected value: {}", _0)]
    UnexpectedValue(i32),
    #[error("Pattern {} does not match with {}", _0, _1)]
    PatternNotMatched(String, String),
    #[error("Unknown nucleotide letter: {}", _0)]
    UnknownNucleotideLetter(char),
    #[error("Unimplemented error")]
    UnimplementedError(),
    #[error("Invalid pattern: {}", _0)]
    InvalidPattern(String),
}