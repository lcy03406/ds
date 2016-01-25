use std::fmt::Debug;
use std::io;
use std::io::{Write, BufRead};
use serde::{Serialize, Deserialize, Error};

use ::service::ServiceStreamer;

pub trait Streamer {
    type Packet : Serialize + Deserialize;
    type Error : Error + Debug;
    fn write_len(len : usize, writer : &mut Write) -> Result<(), Self::Error>;
    fn read_len(reader : &[u8]) -> Result<Option<(usize, usize)>, Self::Error>;
    fn write_to_vec(packet : &Self::Packet) -> Result<Vec<u8>, Self::Error>;
    fn read_from_slice(reader : &[u8]) -> Result<Self::Packet, Self::Error>;
    fn error_from_io(e : io::Error) -> Self::Error; // where Self:Sized;
}

impl<P, E, T> ServiceStreamer for T
    where P : Serialize + Deserialize,
          E : Error + Debug,
          T : Streamer<Packet=P,Error=E> + Sized
{
    type Packet = P;
    type Error = E;
    fn write_packet(packet : &Self::Packet, writer : &mut Write) ->Result<(), Self::Error>
    {
        match Self::write_to_vec(packet) {
            Ok(v) => {
                try!(Self::write_len(v.len(), writer as &mut Write));
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
            Ok(buf) => {
                let len1 = buf.len();
                match Self::read_len(buf) {
                    Ok(Some((header_len, packet_len))) => {
                        let len2 = buf.len();
                        len = header_len + packet_len;
                        if buf.len() < len {
                            return Ok(None);
                        }
                        p = try!(Self::read_from_slice(&buf[header_len..len]));
                    }
                    Ok(None) => {
                        return Ok(None);
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    return Ok(None);
                } else {
                    return Err(Self::error_from_io(e));
                }
            }
        }
        reader.consume(len);
        Ok(Some(p))
    }
}
