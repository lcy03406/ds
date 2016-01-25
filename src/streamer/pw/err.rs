use std::io;
use byteorder;
use serde;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    SerdeError,
}

impl From<io::Error> for Error {
    fn from(e : io::Error) -> Self {
        Error::IoError(e)
    }
}

impl From<byteorder::Error> for Error {
    fn from(e : byteorder::Error) -> Self {
        match e {
            byteorder::Error::Io(e) => {
                Error::IoError(e)
            }
            byteorder::Error::UnexpectedEOF => {
                Error::IoError(io::Error::new(io::ErrorKind::UnexpectedEof, "UnexpectedEOF from byteorder"))
            }
        }
    }
}

impl serde::Error for Error {
    //TODO
    /// Raised when there is general error when deserializing a type.
    fn syntax(msg: &str) -> Self {
        Error::SerdeError
    }

    /// Raised when a `Deserialize` type unexpectedly hit the end of the stream.
    fn end_of_stream() -> Self {
        Error::SerdeError
    }

    /// Raised when a `Deserialize` struct type received an unexpected struct field.
    fn unknown_field(field: &str) -> Self {
        Error::SerdeError
    }

    /// Raised when a `Deserialize` struct type did not receive a field.
    fn missing_field(field: &'static str) -> Self {
        Error::SerdeError
    }
}
