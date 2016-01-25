use std::io;
use std::io::Read;
use byteorder;
use byteorder::{WriteBytesExt, ReadBytesExt, BigEndian};
use serde::{Serialize, Deserialize};
use serde_json::{to_vec, from_slice, Serializer, Deserializer, Error, ErrorCode};

use super::serd::Streamer;

pub trait JsonStreamer {
    type Packet : Serialize + Deserialize;
}

impl<P, T> Streamer for T
    where P : Serialize + Deserialize,
          T : JsonStreamer<Packet=P> + Sized,
{
    type Packet = P;
    type Error = Error;
    fn write_len(len : usize, writer : &mut io::Write) -> Result<(), Self::Error> {
        writer.write_u32::<BigEndian>(len as u32).map_err(error_from_byteorder)
    }
    fn read_len(reader : &mut &[u8]) -> Result<Option<usize>, Self::Error> {
        if reader.len() < 4 {
            Ok(None)
        } else {
            reader.read_u32::<BigEndian>().map(|x| Some(x as usize)).map_err(error_from_byteorder)
        }
    }
    fn write_to_vec(packet : &Self::Packet) -> Result<Vec<u8>, Self::Error> {
        to_vec(packet)
    }
    fn read_from_slice(reader : &[u8]) -> Result<Self::Packet, Self::Error> {
        from_slice(reader)
    }
    fn error_from_io(e : io::Error) -> Self::Error {
        Error::IoError(e)
    }
}
fn error_from_byteorder(e : byteorder::Error) -> Error {
    match e {
        byteorder::Error::UnexpectedEOF => {
            assert!(false);
            Error::IoError(io::Error::new(io::ErrorKind::UnexpectedEof, "UnexpectedEOF returned by byteorder."));
            unreachable!();
        }
        byteorder::Error::Io(e) => {
            Error::IoError(e)
        }
    }
}

