use std::num::ParseIntError;

#[derive(Debug)]
pub enum ParseError {
    IOError(String),
    InvalidFormat,
    UnknownError(String),
}

impl From<std::io::Error> for ParseError {
    fn from(value: std::io::Error) -> Self {
        ParseError::IOError(value.to_string())
    }
}

impl From<ParseIntError> for ParseError {
    fn from(_: ParseIntError) -> Self {
        ParseError::InvalidFormat
    }
}

#[derive(Debug)]
pub enum DumpError {
    InternalError,
}
