use std::collections::HashMap;
use std::cell::{RefCell};
use std::io::{Result, ErrorKind, Write, Read};
use std::net::SocketAddr;
use std::str::FromStr;
use mio::{Token, Evented, EventSet};
use mio::tcp::{TcpStream, TcpListener, Shutdown};

use super::buffer::Buffer;
use super::looper::{Eventer, LOOPER};

pub struct Stream {
    token : Token,
    registered : EventSet,
    interest : EventSet,
    pub got : EventSet,
    pub is_client : bool,
    pub peer_addr : SocketAddr,
    pub stream : TcpStream,
    wbuf : Buffer,
    rbuf : Buffer,
}

impl Stream {
    pub fn new(token : Token, stream : TcpStream, is_client : bool, peer_addr : SocketAddr) -> Self {
        Stream {
            token : token,
            registered : EventSet::none(),
            interest : EventSet::all(),
            got : EventSet::none(),
            is_client : is_client,
            peer_addr : peer_addr,
            stream : stream,
            wbuf : Buffer::with_capacity(1024),
            rbuf : Buffer::with_capacity(1024),
        }
    }
    pub fn shutdown(&mut self) {
        if self.interest == EventSet::none() {
            return;
        }
        self.got = EventSet::hup();
        self.stream.shutdown(Shutdown::Both);
        trace!("stream shutdown");
        self.interest = EventSet::none();
        LOOPER.with(|looper| {
            looper.borrow_mut().reregister(self.token);
        });
    }
    fn want_writable(&mut self) {
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
        let len = buf.len();
        if self.wbuf.is_empty() && self.got.is_writable() {
            match self.stream.write(buf) {
                Ok(part) => {
                    if part < len {
                        let a = self.wbuf.write(&buf[part..]);
                        debug_assert!(match a { Ok(r) => r == len-part, _ => false});
                        self.want_writable();
                    }
                    Ok(len)
                },
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        self.want_writable();
                        self.wbuf.write(buf)
                    } else {
                        self.shutdown();
                        Err(e)
                    }
                }
            }
        } else {
            //would block, push all into buffer
            self.wbuf.write(buf)
        }
    }
    fn flush(&mut self) -> Result<()> {
        if self.wbuf.is_empty() {
            Ok(())
        } else {
            match self.stream.write(self.wbuf.as_slice()) {
                Ok(part) => {
                    self.wbuf.skip(part);
                    if !self.wbuf.is_empty() {
                        self.want_writable();
                    }
                    Ok(())
                },
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        self.want_writable();
                    } else {
                        self.shutdown();
                    }
                    Err(e)
                }
            }
        }
    }
}

