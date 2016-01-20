use std::fmt::Debug;
use std::io;
use std::io::{Write, BufRead};
use byteorder;
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
use serde::{Serialize, Deserialize, Error};

use ::service::{BufWrite, ServiceStreamer};

pub trait Streamer {
    type Packet : Serialize + Deserialize;
    type Error : Error + Debug;
    fn to_vec(packet : &Self::Packet) -> Result<Vec<u8>, Self::Error>;
    fn from_slice(reader : &[u8]) -> Result<Self::Packet, Self::Error>;
    fn error_from_io(e : io::Error) -> Self::Error where Self:Sized;
    fn error_from_byteorder(e : byteorder::Error) -> Self::Error where Self:Sized;
}

impl<P, E, T> ServiceStreamer for T
    where P : Serialize + Deserialize,
          E : Error + Debug,
          T : Streamer<Packet=P,Error=E> + Sized
{
    type Packet = P;
    type Error = E;
    fn write_packet(packet : &Self::Packet, writer : &mut BufWrite) ->Result<(), Self::Error>
    {
        match Self::to_vec(packet) {
            Ok(v) => {
                try!(writer.write_u32::<LittleEndian>(v.len() as u32).map_err(Self::error_from_byteorder));
                try!(writer.write(&v).map_err(Self::error_from_io));
                Ok(())
            }
            Err(e) => {
                Err(e)
            }
        }
    }
    fn read_packet(reader : &mut BufRead) -> Result<Option<Self::Packet>, Self::Error>
    {
        let len : usize;
        let p : Self::Packet;
        match reader.fill_buf() {
            Ok(mut buf) => {
                if buf.len() < 4 {
                    return Ok(None);
                }
                len = try!(buf.read_u32::<LittleEndian>().map_err(Self::error_from_byteorder)) as usize;
                if buf.len() < len {
                    return Ok(None);
                }
                p = try!(Self::from_slice(&buf));
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    return Ok(None);
                } else {
                    return Err(Self::error_from_io(e));
                }
            }
        }
        reader.consume(4 + len);
        Ok(Some(p))
    }
}
