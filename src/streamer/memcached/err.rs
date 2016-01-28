use std::io;
use byteorder;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    WrongLen,
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
