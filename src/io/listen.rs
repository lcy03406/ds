use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::cell::{RefCell};
use std::io::{Result, ErrorKind, Write, Read};
use std::net::SocketAddr;
use std::str::FromStr;
use mio::{Token, Evented, EventSet};
use mio::tcp::{TcpStream, TcpListener, Shutdown};

use super::looper::{Eventer, Looper, LooperAndToken};
use super::stream::{Stream, Streamer};

pub trait Listener<'a, > {
    type Streamer : Streamer<'a> + Sized + 'a;
    fn on_accept(&self) -> Self::Streamer;
    fn on_close(&self, c : &mut Listen<'a>);
}

pub struct Listen<'a> {
    lt : LooperAndToken<'a>,
    interest : EventSet,
    got : EventSet,
    pub listener : TcpListener,
    pub addr : SocketAddr,
}

impl<'a> Listen<'a> {
    pub fn close(&mut self) {
        if self.interest == EventSet::none() {
            return;
        }
        self.got = EventSet::hup();
        trace!("listen shutdown");
        self.interest = EventSet::none();
        self.lt.reregister();
    }
    fn on_ready<S : Streamer<'a> + Sized + 'a>(&mut self, listener : &mut (Listener<'a, Streamer=S> + 'a), es : EventSet) {
        let got = self.got;
        self.got = es;
        if es.is_error() || es.is_hup() {
            trace!("listen error?");
            self.close();
            return;
        }
        let looper;
        match Weak::upgrade(&self.lt.looper) {
            Some(l) => {
                looper = l;
            }
            None => {
                trace!("listen weak looper");
                self.close();
                return;
            }
        }
        if es.is_writable() {
            trace!("listen writable?");
        }
        if es.is_readable() {
            trace!("listen read");
            match self.listener.accept() {
                Ok(Some((stream, addr))) => {
                    Stream::accepted(&looper, listener.on_accept(), stream, addr);
                }
                Ok(None) => {
                    trace!("listen accept none");
                }
                Err(e) => {
                    trace!("listen accept err {:?}", e);
                }
            }
        }
    }
}

struct ListenAndListener<'a, T : Listener<'a> + 'a> {
    listen : Listen<'a>,
    listener : T,
}

impl<'a, T : Listener<'a> + 'a> Eventer<'a> for ListenAndListener<'a, T> {
    fn looper_and_token(&mut self) -> &mut LooperAndToken<'a> {
        &mut self.listen.lt
    }
    fn interest(&self) -> EventSet {
        self.listen.interest
    }
    fn evented(&self) -> &Evented {
        &self.listen.listener
    }
    fn on_ready(&mut self, es : EventSet) {
        self.listen.on_ready(&mut self.listener, es);
    }
    fn on_close(&mut self) {
        self.listener.on_close(&mut self.listen);
    }
}
