use std::io;

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

