use std::num::ParseIntError;

#[derive(Debug)]
pub enum ParseError {
    IOError(String),
    InvalidFormat(String),
}

impl From<std::io::Error> for ParseError {
    fn from(value: std::io::Error) -> Self {
        ParseError::IOError(value.to_string())
    }
}

impl From<ParseIntError> for ParseError {
    fn from(err: ParseIntError) -> Self {
        ParseError::InvalidFormat(err.to_string())
    }
}

#[derive(Debug)]
pub enum DumpError {
    InternalError,
    OutputError,
}

impl From<std::io::Error> for DumpError {
    fn from(_: std::io::Error) -> Self {
        DumpError::OutputError
    }
}
