use std::io;
use std::io::{Write, BufRead, Read};
use byteorder::{ByteOrder, BigEndian, ReadBytesExt, WriteBytesExt};


use service::ServiceStreamer;

use ::super::protocol::*;
use ::super::err::Error;

pub struct MemcachedStreamer;

impl ServiceStreamer for MemcachedStreamer {
    type Packet = Packet;
    type Error = Error;
    fn write_packet(packet: &Self::Packet, writer: &mut Write) -> Result<(), Self::Error> {
        try!(writer.write_u8(packet.header.magic.0));
        try!(writer.write_u8(packet.header.opcode.0));
        try!(writer.write_u16::<BigEndian>(packet.key.len() as u16));
        try!(writer.write_u8(packet.extras.len() as u8));
        try!(writer.write_u8(packet.header.datatype.0));
        try!(writer.write_u16::<BigEndian>(packet.header.status.0));
        let bodylen = packet.key.len() + packet.extras.len() + packet.value.len();
        try!(writer.write_u32::<BigEndian>(bodylen as u32));
        try!(writer.write_u32::<BigEndian>(packet.header.opaque));
        try!(writer.write_u64::<BigEndian>(packet.header.cas));
        try!(writer.write_all(&packet.extras[..]));
        try!(writer.write_all(&packet.key.as_bytes()));
        try!(writer.write_all(&packet.value[..]));
        Ok(())
    }
    fn read_packet(reader: &mut BufRead) -> Result<Option<Self::Packet>, Self::Error> {
        let len: usize;
        let p: Packet;
        match reader.fill_buf() {
            Ok(buf) => {
                let buflen = buf.len();
                if buflen < HEADER_SIZE {
                    trace!("buflen {}", buflen);
                    return Ok(None);
                }
                let bodylen = BigEndian::read_u32(&buf[8..12]);
                let totallen = HEADER_SIZE + bodylen as usize;
                if buflen < totallen {
                    trace!("buflen {} totallen {}", buflen, totallen);
                    return Ok(None);
                }
                let magic = buf[0];
                let opcode = buf[1];
                let keylen = BigEndian::read_u16(&buf[2..4]);
                let extlen = buf[4];
                let datatype = buf[5];
                let status = BigEndian::read_u16(&buf[6..8]);
                //let _bodylen = BigEndian::read_u32(&buf[8..12]);
                let opaque = BigEndian::read_u32(&buf[12..16]);
                let cas = BigEndian::read_u64(&buf[16..HEADER_SIZE]);
                let extend = 24 + extlen as usize;
                let keyend = extend + keylen as usize;
                let valueend = 24 + bodylen as usize;
                if keyend > valueend {
                    trace!("keyend {} valueend {}", keyend, valueend);
                    return Err(Error::WrongLen);
                }
                let mut ext = Vec::new();
                try!((&buf[24..extend]).read_to_end(&mut ext));
                let key = String::from_utf8_lossy(&buf[extend..keyend]).to_string();
                let mut value = Vec::new();
                try!((&buf[keyend..valueend]).read_to_end(&mut value));
                len = totallen;
                p = Packet {
                    header : Header {
                        magic : Magic(magic),
                        opcode : Opcode(opcode),
                        keylen : keylen,
                        extlen : extlen,
                        datatype : DataType(datatype),
                        status : ResponseStatus(status),
                        bodylen : bodylen,
                        opaque : opaque,
                        cas : cas,
                    },
                    extras : ext,
                    key : key,
                    value : value,
                };
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    trace!("wouldblock");
                    return Ok(None);
                } else {
                    return Err(From::from(e));
                }
            }
        }
        reader.consume(len);
        Ok(Some(p))
    }
}

