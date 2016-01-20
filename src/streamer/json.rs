use std::io;
use byteorder;
use serde::{Serialize, Deserialize};
use serde_json::{to_vec, from_slice};

use super::serd::Streamer;

pub use serde_json::Error as Error;

pub trait JsonStreamer {
    type Packet : Serialize + Deserialize;
}

impl<P, T> Streamer for T
    where P : Serialize + Deserialize,
          T : JsonStreamer<Packet=P> + Sized,
{
    type Packet = P;
    type Error = Error;
    fn to_vec(packet : &Self::Packet) -> Result<Vec<u8>, Self::Error>
    {
        to_vec(packet)
    }
    fn from_slice(reader : &[u8]) -> Result<Self::Packet, Self::Error>
    {
        from_slice(reader)
    }
    fn error_from_io(e : io::Error) -> Self::Error {
        Error::IoError(e)
    }
    fn error_from_byteorder(e : byteorder::Error) -> Self::Error {
        match e {
            byteorder::Error::UnexpectedEOF => {
                assert!(false);
                Self::error_from_io(io::Error::new(io::ErrorKind::UnexpectedEof, "UnexpectedEOF returned by byteorder."));
                unreachable!();
            }
            byteorder::Error::Io(e) => {
                Self::error_from_io(e)
            }
        }
    }
}
