use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("Enexpected value: {}", _0)]
    UnexpectedValue(i32),
    #[error("Unknown nucleotide letter: {}", _0)]
    UnknownNucleotideLetter(char),
    #[error("Unimplemented error")]
    UnimplementedError(),
}