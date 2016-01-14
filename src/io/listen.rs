use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::cell::{RefCell};
use std::io::{Result, ErrorKind, Write, Read};
use std::net::SocketAddr;
use std::str::FromStr;
use mio::{Token, Evented, EventSet};
use mio::tcp::{TcpStream, TcpListener, Shutdown};

use super::looper::{Eventer, LOOPER};

pub struct Listen {
    token : Token,
    registered : EventSet,
    interest : EventSet,
    pub got : EventSet,
    pub addr : SocketAddr,
    pub listener : TcpListener,
}

impl Listen {
    pub fn new(token : Token, addr : SocketAddr) -> Self {
        Listen {
            token : token,
            registered : EventSet::none(),
            interest : EventSet::all(),
            got : EventSet::none(),
            addr : addr,
            listener : TcpListener::bind(&addr).unwrap(),
        }
    }
    pub fn shutdown(&mut self) {
        if self.interest == EventSet::none() {
            return;
        }
        self.got = EventSet::hup();
        trace!("listen shutdown");
        self.interest = EventSet::none();
        LOOPER.with(|looper| {
            looper.borrow_mut().reregister(self.token);
        });
    }
}

impl Eventer for Listen {
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
        &self.listener
    }
}
