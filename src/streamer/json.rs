use std::marker::PhantomData;
use std::io;
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};
use serde_json::{to_writer, to_vec, from_slice, Deserializer, Error};

use ::super::headbody::{HeadStreamer, BodyStreamer, ErrorMapper, StreamerImpl};

pub struct JsonHeadStreamer;

impl HeadStreamer for JsonHeadStreamer {
    type Error = Error;
    fn write_len(len: usize, mut writer: &mut Write) -> Result<(), Self::Error> {
        to_writer(&mut writer, &([len]))
    }
    fn read_len(reader: &[u8]) -> Result<Option<(usize, usize)>, Self::Error> {
        let len1 = reader.len();
        trace!("json deserialize len {}", len1);
        let r = &mut &*reader;
        match <([usize;1]) as Deserialize>::deserialize(&mut Deserializer::new(r.bytes())) {
            Ok(len) => {
                let len2 = reader.len();
                trace!("json deserialize len {} {}", len1, len2);
                Ok(Some((len1 - len2, len[0])))
            }
            Err(e) => {
                trace!("json deserialize error {:?} when {:?}", e, reader);
                Ok(None)
            }
        }
    }
}

pub struct JsonBodyStreamer<P>
    where P : Serialize + Deserialize,
{
    p : PhantomData<*const P>,
}

impl<P> BodyStreamer for JsonBodyStreamer<P>
    where P : Serialize + Deserialize,
{
    type Packet = P;
    type Error = Error;
    fn write_to_vec(packet : &Self::Packet) -> Result<Vec<u8>, Self::Error> {
        to_vec(packet)
    }
    fn read_from_slice(reader : &[u8]) -> Result<Self::Packet, Self::Error> {
        from_slice(reader)
    }
}

pub struct JsonErrorMapper;

impl ErrorMapper for JsonErrorMapper {
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

pub struct JsonStreamer<P>
    where P: Serialize + Deserialize
{
    p : PhantomData<*const P>,
}

impl<P> StreamerImpl for JsonStreamer<P>
    where P: Serialize + Deserialize
{
    type Head = JsonHeadStreamer;
    type Body = JsonBodyStreamer<P>;
    type Error = JsonErrorMapper;
}

