use std::net::SocketAddr;
use mio::{Token, Evented, EventSet};
use mio::tcp::TcpListener;

use super::looper::{Eventer, LOOPER};

pub struct Listen {
    token : Token,
    registered : EventSet,
    interest : EventSet,
    pub addr : SocketAddr,
    pub listener : TcpListener,
}

impl Listen {
    pub fn new(token : Token, addr : SocketAddr) -> Self {
        trace!("listen bind {:?} {}", token, addr);
        Listen {
            token : token,
            registered : EventSet::none(),
            interest : EventSet::all(),
            addr : addr,
            listener : TcpListener::bind(&addr).unwrap(),
        }
    }
    pub fn shutdown(&mut self) {
        if self.interest == EventSet::none() {
            return;
        }
        trace!("listen shutdown");
        self.interest = EventSet::none();
        LOOPER.with(|looper| {
            looper.borrow_mut().as_mut().unwrap().reregister(self.token);
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
