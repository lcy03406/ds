use std::collections::HashMap;
use std::rc::{Rc};
use std::cell::{RefCell};
use std::io::{Result, ErrorKind, Write, Read};
use std::net::SocketAddr;
use std::str::FromStr;
use mio::{Token, Evented, EventSet};
use mio::tcp::{TcpStream, TcpListener, Shutdown};

use super::buffer::Buffer;
use super::looper::{Eventer, Looper, LooperAndToken};
use super::service::Service;

pub trait Streamer<'a> {
    fn on_connect(&self, c : &mut StreamBody<'a>);
    fn on_close(&self, c : &mut StreamBody<'a>);
    fn on_read(&self, c : &mut StreamBody<'a>);
}

pub struct StreamBody<'a> {
    lt : LooperAndToken<'a>,
    interest : EventSet,
    got : EventSet,
    pub is_client : bool,
    pub peer_addr : SocketAddr,
    pub stream : TcpStream,
    wbuf : Buffer,
    rbuf : Buffer,
}

impl<'a> StreamBody<'a> {
    pub fn close(&mut self) {
        if self.interest == EventSet::none() {
            return;
        }
        self.got = EventSet::hup();
        self.stream.shutdown(Shutdown::Both);
        trace!("stream shutdown");
        self.interest = EventSet::none();
        self.lt.reregister();
    }
    fn want_writable(&mut self) {
        self.got.remove(EventSet::writable());
        //self.interest.insert(EventSet::writable());
        //self.reregister();
    }
    fn on_ready(&mut self, streamer : &mut (Streamer<'a> + 'a), es : EventSet) {
        let got = self.got;
        self.got = es;
        if es.is_error() || es.is_hup() {
            self.close();
            return;
        }
        if es.is_writable() {
            if got == EventSet::none() {
                trace!("stream connect");
                streamer.on_connect(self);
            }
            //self.interest.remove(EventSet::writable());
            self.flush();
        }
        if es.is_readable() {
            trace!("stream read");
            streamer.on_read(self);
        }
    }
}

impl<'a> Write for StreamBody<'a> {
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
                        self.close();
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
                        self.close();
                    }
                    Err(e)
                }
            }
        }
    }
}

pub struct Stream<'a, T : Streamer<'a> + 'a> {
    stream : StreamBody<'a>,
    streamer : T,
}

impl<'a, T : Streamer<'a> + 'a> Stream<'a, T> {
    pub fn connect(looper : &Rc<RefCell<Looper<'a>>>, streamer : T, to : SocketAddr) -> Result<Rc<RefCell<Stream<'a, T>>>> {
        let ter = Rc::new(RefCell::new(Stream {
            stream : StreamBody {
                lt : LooperAndToken {
                    looper : Rc::downgrade(looper),
                    token : Token(0),
                    registered : EventSet::none(),
                },
                interest : EventSet::all(),
                got : EventSet::none(),
                is_client : true,
                peer_addr : to,
                stream : try!(TcpStream::connect(&to)),
                wbuf : Buffer::with_capacity(1024),
                rbuf : Buffer::with_capacity(1024),
            },
            streamer : streamer
        }));
        let a : Rc<RefCell<Eventer<'a>>> = ter.clone();
        Looper::register(looper, &a);
        Ok(ter)
    }
    pub fn accepted(looper : &Rc<RefCell<Looper<'a>>>, streamer : T, stream : TcpStream, to : SocketAddr) -> Rc<RefCell<Stream<'a, T>>> {
        let ter = Rc::new(RefCell::new(Stream {
            stream : StreamBody {
                lt : LooperAndToken {
                    looper : Rc::downgrade(looper),
                    token : Token(0),
                    registered : EventSet::none(),
                },
                interest : EventSet::all(),
                got : EventSet::none(),
                is_client : false,
                peer_addr : to,
                stream : stream,
                wbuf : Buffer::with_capacity(1024),
                rbuf : Buffer::with_capacity(1024),
            },
            streamer : streamer
        }));
        let a : Rc<RefCell<Eventer<'a>>> = ter.clone();
        Looper::register(looper, &a);
        ter
    }
}

impl<'a, T : Streamer<'a> + 'a> Eventer<'a> for Stream<'a, T> {
    fn looper_and_token(&mut self) -> &mut LooperAndToken<'a> {
        &mut self.stream.lt
    }
    fn interest(&self) -> EventSet {
        self.stream.interest
    }
    fn evented(&self) -> &Evented {
        &self.stream.stream
    }
    fn on_ready(&mut self, es : EventSet) {
        self.stream.on_ready(&mut self.streamer, es);
    }
    fn on_close(&mut self) {
        self.streamer.on_close(&mut self.stream);
    }
}
