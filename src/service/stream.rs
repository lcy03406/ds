use std::io::{Result, ErrorKind, Write, Read, BufRead};
use std::net::SocketAddr;
use std::cmp::min;
use mio::{Token, Evented, EventSet};
use mio::tcp::{TcpStream, Shutdown};

use super::buffer::Buffer;
use super::looper::{Eventer, LOOPER};
use super::bufwrite::BufWrite;

pub struct Stream {
    token : Token,
    registered : EventSet,
    interest : EventSet,
    pub got : EventSet,
    pub is_client : bool,
    pub reconnect : bool,
    pub peer_addr : SocketAddr,
    pub stream : TcpStream,
    wbuf : Buffer,
    rbuf : Buffer,
}

const INIT_WBUF_SIZE : usize = 4096;
const INIT_RBUF_SIZE : usize = 4096;
const MORE_WBUF_SIZE : usize = 4096;
const MORE_RBUF_SIZE : usize = 4096;

impl Stream {
    pub fn new(token : Token, stream : TcpStream, is_client : bool, reconnect : bool, peer_addr : SocketAddr) -> Self {
        Stream {
            token : token,
            registered : EventSet::none(),
            interest : EventSet::all(),
            got : EventSet::none(),
            is_client : is_client,
            reconnect : reconnect,
            peer_addr : peer_addr,
            stream : stream,
            wbuf : Buffer::with_capacity(INIT_WBUF_SIZE),
            rbuf : Buffer::with_capacity(INIT_RBUF_SIZE),
        }
    }
    pub fn shutdown(&mut self) {
        if self.interest == EventSet::none() {
            return;
        }
        self.got = EventSet::hup();
        self.stream.shutdown(Shutdown::Both).ok();
        trace!("stream shutdown");
        self.interest = EventSet::none();
        LOOPER.with(|looper| {
            looper.borrow_mut().as_mut().unwrap().reregister(self.token);
        });
    }
    fn want_writable(&mut self) {
        self.got.remove(EventSet::writable());
    }
    fn want_readable(&mut self) {
        self.got.remove(EventSet::writable());
    }
}

impl Eventer for Stream {
    fn registered(&self) -> EventSet {
        self.registered
    }
    fn set_registered(&mut self, es : EventSet) {
        self.registered = es;
    }
    fn interest(&self) -> EventSet {
        self.interest
    }
    fn evented(&self) -> &Evented {
        &self.stream
    }
}

impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.wbuf.write(buf)
    }
    fn flush(&mut self) -> Result<()> {
        if self.wbuf.is_empty() {
            trace!("stream flush empty");
            Ok(())
        } else {
            trace!("stream flush");
            match self.stream.write(self.wbuf.fill_buf().unwrap()) {
                Ok(part) => {
                    self.wbuf.consume(part);
                    if !self.wbuf.is_empty() {
                        self.want_writable();
                    }
                    Ok(())
                },
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        self.want_writable();
                    } else {
                        trace!("stream write err {:?}", e);
                        self.shutdown();
                    }
                    Err(e)
                }
            }
        }
    }
}

impl BufWrite for Stream {
    fn reserve_buf(&mut self, min_size : usize) -> &mut [u8] {
        self.wbuf.reserve_buf(min(min_size, MORE_WBUF_SIZE))
    }
    fn buf_filled(&mut self, amt: usize) {
        self.wbuf.buf_filled(amt);
    }
}

impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.rbuf.data_len() < buf.len() {
            trace!("stream read need more {} < {}", self.rbuf.data_len(), buf.len());
            match self.fill_buf() {
                Ok(_) => {
                    self.rbuf.read(buf)
                }
                Err(e) => {
                    Err(e)
                }
            }
        } else {
            self.rbuf.read(buf)
        }
    }
}

impl BufRead for Stream {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        loop {
            match self.stream.read(self.rbuf.reserve_buf(MORE_RBUF_SIZE)) {
                Ok(part) => {
                    self.rbuf.buf_filled(part);
                    if part == 0 {
                        trace!("stream read zero");
                        self.shutdown();
                        break;
                    }
                },
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        self.want_readable();
                    } else {
                        trace!("stream read err {:?}", e);
                        self.shutdown();
                    }
                    if self.rbuf.is_empty() {
                        return Err(e)
                    } else {
                        break;
                    }
                }
            }
        }
        self.rbuf.fill_buf()
    }
    fn consume(&mut self, amt: usize) {
        self.rbuf.consume(amt);
    }
}

