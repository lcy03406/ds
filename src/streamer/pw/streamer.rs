use std::marker::PhantomData;
use std::io;
use std::io::Read;
use byteorder;
use byteorder::{WriteBytesExt, ReadBytesExt, BigEndian};
use serde::{Serialize, Deserialize};

use ::super::super::serd::Streamer;
use ::super::ser::Serializer;
use ::super::de::Deserializer;
use ::super::err::Error;

pub struct PwStreamer<P>
    where P : Serialize + Deserialize
{
    phantom : PhantomData<*const P>
}

impl<P> Streamer for PwStreamer<P>
    where P : Serialize + Deserialize,
{
    type Packet = P;
    type Error = Error;
    fn write_len(len : usize, writer : &mut io::Write) -> Result<(), Self::Error> {
        Ok(Serializer::new(writer).compact_u32(len as u32).unwrap())
    }
    fn read_len(reader : &[u8]) -> Result<Option<(usize, usize)>, Self::Error> {
        let len1 = reader.len();
        let r = &mut &*reader;
        match Deserializer::new(r).uncompact_u32() {
            Ok(len) => {
                let len2 = reader.len();
                Ok(Some((len1 - len2, len as usize)))
            }
            Err(Error::IoError(ref e)) if e.kind() == io::ErrorKind::UnexpectedEof => {
                let len2 = reader.len();
                Ok(None)
            }
            Err(e) => {
                assert!(false);
                Ok(None)
            }
        }
    }
    fn write_to_vec(packet : &Self::Packet) -> Result<Vec<u8>, Self::Error> {
        let mut buf = Vec::new();
        try!(packet.serialize(&mut Serializer::new(&mut buf)));
        Ok(buf)
    }
    fn read_from_slice(reader : &[u8]) -> Result<Self::Packet, Self::Error> {
        Deserialize::deserialize(&mut Deserializer::new(reader))
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

