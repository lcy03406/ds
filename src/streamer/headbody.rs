use std::fmt::Debug;
use std::io;
use std::io::{Write, BufRead};

use service::ServiceStreamer;

pub trait HeadStreamer {
    type Error : Debug;
    fn write_len(len: usize, writer: &mut Write) -> Result<(), Self::Error>;
    fn read_len(reader: &[u8]) -> Result<Option<(usize, usize)>, Self::Error>;
}

pub trait BodyStreamer {
    type Packet;
    type Error : Debug;
    fn write_to_vec(packet: &Self::Packet) -> Result<Vec<u8>, Self::Error>;
    fn read_from_slice(reader: &[u8]) -> Result<Self::Packet, Self::Error>;
}

pub trait ErrorMapper {
    type HE;
    type BE;
    type Error : Debug;
    fn error_from_head(e: Self::HE) -> Self::Error;
    fn error_from_body(e: Self::BE) -> Self::Error;
    fn error_from_io(e: io::Error) -> Self::Error;
}

pub trait StreamerImpl {
    type Head : HeadStreamer;
    type Body : BodyStreamer;
    type Error : ErrorMapper;
}
//<HE=<Self as StreamerImpl>::Head::Error, BE=<Self as StreamerImpl>::Body::Error>;

impl<T,H,B,E> ServiceStreamer for T
    where T: StreamerImpl<Head=H, Body=B, Error=E>,
          H: HeadStreamer,
          B: BodyStreamer,
          E: ErrorMapper<HE=H::Error, BE=B::Error>
{
    type Packet = B::Packet;
    type Error = E::Error;
    fn write_packet(packet: &Self::Packet, writer: &mut Write) -> Result<(), Self::Error> {
        match B::write_to_vec(packet) {
            Ok(v) => {
                try!(H::write_len(v.len(), writer as &mut Write).map_err(E::error_from_head));
                try!(writer.write(&v).map_err(E::error_from_io));
                Ok(())
            }
            Err(e) => Err(E::error_from_body(e)),
        }
    }
    fn read_packet(reader: &mut BufRead) -> Result<Option<Self::Packet>, Self::Error> {
        let len: usize;
        let p: Self::Packet;
        match reader.fill_buf() {
            Ok(buf) => {
                match H::read_len(buf) {
                    Ok(Some((header_len, packet_len))) => {
                        len = header_len + packet_len;
                        if buf.len() < len {
                            return Ok(None);
                        }
                        p = try!(B::read_from_slice(&buf[header_len..len]).map_err(E::error_from_body));
                    }
                    Ok(None) => {
                        return Ok(None);
                    }
                    Err(e) => {
                        return Err(E::error_from_head(e));
                    }
                }
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    return Ok(None);
                } else {
                    return Err(E::error_from_io(e));
                }
            }
        }
        reader.consume(len);
        Ok(Some(p))
    }
}

/*
pub struct Streamer<H,B,E>
    where H: HeadStreamer,
          B: BodyStreamer,
          E: ErrorMapper<HE=H::Error, BE=B::Error>
{
    h : PhantomData<*const H>,
    b : PhantomData<*const B>,
    e : PhantomData<*const E>,
}

impl<H, B, E> StreamerImpl for Streamer<H,B,E>
    where H: HeadStreamer,
          B: BodyStreamer,
          E: ErrorMapper<HE=H::Error, BE=B::Error>
{
    type Head = H;
    type Body = B;
    type Error = E;
}
*/
