use std::io::{Read, Write, BufRead};
use std::io;
use ds::io::{BufWrite, ServiceStreamer, StreamerResult};

pub trait Serializable {
    fn serialize(&self, writer : &mut BufWrite) ->StreamerResult<()>;
    fn unmarshal(reader : &mut BufRead) -> StreamerResult<Self>;
}

impl Serializable for u8 {
    fn serialize(&self, writer : &mut BufWrite) ->StreamerResult<()> {
        write_raw_byte(writer, self);
        Ok(())
    }
    fn unmarshal(reader : &mut BufRead) -> StreamerResult<Self> {

    }
}

