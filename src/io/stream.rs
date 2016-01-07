use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::cell::{RefCell};
use std::mem::swap;
use std::io::{Result, ErrorKind, Write, Read};
use std::net::SocketAddr;
use std::str::FromStr;
use mio::{Token, Evented, EventSet};
use mio::tcp::{TcpStream, TcpListener, Shutdown};
use super::buffer::Buffer;
use super::looper::{Eventer, Looper, LooperAndToken};

pub trait Streamer<'a> {
    fn on_accept(&self, c : &mut Stream<'a>);
    fn on_connect(&self, c : &mut Stream<'a>);
    fn on_close(&self, c : &mut Stream<'a>);
    fn on_read(&self, c : &mut Stream<'a>);
}

pub struct Stream<'a> {
    lt : LooperAndToken<'a>,
    interest : EventSet,
    got : EventSet,
    stream : TcpStream,
    wbuf : Buffer,
    conn : Weak<RefCell<Streamer<'a>>>,
}

impl<'a> Stream<'a> {
    pub fn connect(looper : &Rc<RefCell<Looper<'a>>>, conn : Weak<RefCell<Streamer<'a>>>, to : &str) -> Result<()> {
        let ter = Rc::new(RefCell::new(Stream {
            lt : LooperAndToken {
                looper : Rc::downgrade(looper),
                token : Token(0),
                registered : EventSet::none(),
            },
            interest : EventSet::all(),
            got : EventSet::none(),
            stream : try!(TcpStream::connect(&SocketAddr::from_str(to).unwrap())),
            wbuf : Buffer::with_capacity(1024),
            conn : conn,
        }));
        Looper::register(looper, ter);
        Ok(())
    }
    pub fn close(&mut self) {
        if self.interest == EventSet::none() {
            return;
        }
        self.got = EventSet::hup();
        self.stream.shutdown(Shutdown::Both);
        match Weak::upgrade(&self.conn) {
            Some(rc) => {
                trace!("conn close");
                rc.borrow().on_close(self);
            }
            None => {
                trace!("conn close orphan");
            }
        }
        self.interest = EventSet::none();
        self.reregister();
    }
    fn want_writable(&mut self) {
        self.got.remove(EventSet::writable());
        //self.interest.insert(EventSet::writable());
        //self.reregister();
    }
}

impl<'a> Eventer<'a> for Stream<'a> {
    fn looper_and_token(&mut self) -> &mut LooperAndToken<'a> {
        &mut self.lt
    }
    fn interest(&self) -> EventSet {
        self.interest
    }
    fn evented(&self) -> &Evented {
        &self.stream
    }
    fn on_ready(&mut self, es : EventSet) {
        let got = self.got;
        self.got = es;
        if es.is_error() || es.is_hup() {
            self.close();
            return;
        }
        if es.is_writable() {
            if got == EventSet::none() {
                match Weak::upgrade(&self.conn) {
                    Some(rc) => {
                        trace!("conn connect");
                        rc.borrow().on_connect(self);
                    }
                    None => {
                        trace!("conn orphan");
                        self.close();
                        return;
                    }
                }
            }
            //self.interest.remove(EventSet::writable());
            self.flush();
        }
        if es.is_readable() {
            match Weak::upgrade(&self.conn) {
                Some(rc) => {
                    trace!("conn read");
                    rc.borrow().on_read(self);
                }
                None => {
                    trace!("conn orphan");
                    self.close();
                    return;
                }
            }
        }
    }
}

impl<'a> Write for Stream<'a> {
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
