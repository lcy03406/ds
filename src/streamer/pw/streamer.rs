use std::marker::PhantomData;
use std::io;
use serde::{Serialize, Deserialize};

use ::super::super::serd::{HeadStreamer, BodyStreamer, ErrorMapper, StreamerImpl};
use ::super::ser::Serializer;
use ::super::de::Deserializer;
use ::super::err::Error;

pub struct PwHeadStreamer;

impl HeadStreamer for PwHeadStreamer {
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
                Ok(None)
            }
            Err(e) => {
                assert!(false);
                Ok(None)
            }
        }
    }
}

pub struct PwBodyStreamer<P>
    where P : Serialize + Deserialize,
{
    p : PhantomData<*const P>,
}

impl<P> BodyStreamer for PwBodyStreamer<P>
    where P : Serialize + Deserialize,
{
    type Packet = P;
    type Error = Error;
    fn write_to_vec(packet : &Self::Packet) -> Result<Vec<u8>, Self::Error> {
        let mut buf = Vec::new();
        try!(packet.serialize(&mut Serializer::new(&mut buf)));
        Ok(buf)
    }
    fn read_from_slice(reader : &[u8]) -> Result<Self::Packet, Self::Error> {
        Deserialize::deserialize(&mut Deserializer::new(reader))
    }
}

pub struct PwErrorMapper;

impl ErrorMapper for PwErrorMapper {
    type HE = Error;
    type BE = Error;
    type Error = Error;
    fn error_from_head(e: Self::HE) -> Self::Error {
        e
    }
    fn error_from_body(e: Self::BE) -> Self::Error {
        e
    }
    fn error_from_io(e: io::Error) -> Self::Error {
        Error::IoError(e)
    }
}

pub struct PwStreamer<P>
    where P : Serialize + Deserialize
{
    phantom : PhantomData<*const P>
}

impl<P> StreamerImpl for PwStreamer<P>
    where P: Serialize + Deserialize
{
    type Head = PwHeadStreamer;
    type Body = PwBodyStreamer<P>;
    type Error = PwErrorMapper;
}

