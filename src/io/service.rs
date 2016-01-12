use std::rc::{Rc, Weak};
use std::cell::{RefCell};
use std::str::FromStr;
use std::collections::HashMap;
use std::io::{Result, ErrorKind, Write, Read};
use std::net::SocketAddr;
use mio::{Token};

use super::looper::{Looper, LoopHandler};
use super::stream::{Stream, Streamer, StreamBody};
use super::listen::{Listen, Listener};

pub trait ServiceHandler<'a> {
    fn on_connect(&self, c : &mut StreamBody<'a>);
    fn on_close(&self, c : &mut StreamBody<'a>);
    fn on_read(&self, c : &mut StreamBody<'a>);
}

pub struct ServiceConfig {
    listen : Vec<String>,
    connect : Vec<String>,
}

pub struct ServiceBody<'a> {
    looper : Rc<RefCell<Looper<'a>>>,
    config : ServiceConfig,
    streams : HashMap<Token, Rc<RefCell<Stream<'a, ServiceRef<'a>>>>>,
}

impl<'a> ServiceBody<'a> {
    fn new(lh : &LoopHandler<'a>, config : ServiceConfig) -> Self {
        ServiceBody::<'a> {
            looper : lh.looper.clone(),
            config : config,
            streams : HashMap::new(),
        }
    }

}

pub struct Service<'a> {
    service : Rc<RefCell<ServiceBody<'a>>>,
}

impl<'a> Service<'a> {
    fn start(&self) {
        let myself = self.service.borrow_mut();
        myself.config.connect.iter().map(|to| {
            let addr = SocketAddr::from_str(to).unwrap();
            Stream::connect(&myself.looper, ServiceRef::new(Rc::downgrade(&self.service)), addr);
        });
    }
 //   fn get_stream(&self, token : Token) -> Option<&mut Stream<'a, ServiceRef<'a>>> {
  //  }
}

struct ServiceRef<'a> {
    service : Weak<RefCell<ServiceBody<'a>>>,
}

impl<'a> ServiceRef<'a> {
    fn new(service : Weak<RefCell<ServiceBody<'a>>>) -> Self {
        ServiceRef {
            service : service
        }
    }
}

impl<'a> Streamer<'a> for ServiceRef<'a> {
    fn on_connect(&self, c : &mut StreamBody<'a>) {
    }
    fn on_close(&self, c : &mut StreamBody<'a>) {
        let service;
        match Weak::upgrade(&self.service) {
            Some(ser) => {
                service = ser;
            }
            None => {
                c.close();
                return;
            }
        }
        let looper = service.borrow_mut().looper.clone();
        if c.is_client {
            let addr = c.peer_addr;
            Stream::connect(&looper, ServiceRef::new(Rc::downgrade(&service)), addr);
        }
    }
    fn on_read(&self, c : &mut StreamBody<'a>) {
    }
}
