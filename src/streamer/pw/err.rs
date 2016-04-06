use std::io;
use std::error;
use std::fmt;
use serde;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    SerdeError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ErroR")
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "ErroR"
    }
}

impl From<io::Error> for Error {
    fn from(e : io::Error) -> Self {
        Error::IoError(e)
    }
}

impl serde::de::Error for Error {
    //TODO
    /// Raised when there is general error when deserializing a type.
    fn custom<T: Into<String>>(msg: T) -> Self {
        Error::SerdeError
    }
    //TODO
    /// Raised when a `Deserialize` type unexpectedly hit the end of the stream.
    fn end_of_stream() -> Self {
        Error::SerdeError
    }
}

impl serde::ser::Error for Error {
    //TODO
    /// Raised when there is general error when deserializing a type.
    fn custom<T: Into<String>>(msg: T) -> Self {
        Error::SerdeError
    }
}
