use std::marker::PhantomData;
use std::io;
use serde::{Serialize, Deserialize};

use ::super::super::headbody::{HeadStreamer, BodyStreamer, ErrorMapper, StreamerImpl};
use ::super::ser::Serializer;
use ::super::de::Deserializer;
use ::super::err::Error;

pub struct PwHeadStreamer;

impl HeadStreamer for PwHeadStreamer {
    type Error = Error;
    fn write_len(len : usize, writer : &mut io::Write) -> Result<(), Self::Error> {
        Ok(())
    }
    fn read_len(reader : &[u8]) -> Result<Option<(usize, usize)>, Self::Error> {
        let len1 = reader.len();
        let r = &mut &*reader;
        let mut de = Deserializer::new(r);
        let _ty = match de.uncompact_u32() {
            Ok(t) => {
                t
            }
            Err(Error::IoError(ref e)) if e.kind() == io::ErrorKind::UnexpectedEof => {
                return Ok(None)
            }
            Err(_) => {
                assert!(false);
                return Ok(None)
            }
        };
        let len = match de.uncompact_u32() {
            Ok(l) => {
                l
            }
            Err(Error::IoError(ref e)) if e.kind() == io::ErrorKind::UnexpectedEof => {
                return Ok(None)
            }
            Err(_) => {
                assert!(false);
                return Ok(None)
            }
        };
        let len2 = reader.len();
        Ok(Some((0, len1 - len2 + len as usize)))
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

