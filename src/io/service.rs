use std::rc::{Rc};
use std::cell::{RefCell};
use std::str::FromStr;
use std::collections::HashMap;
use std::io::{Result, ErrorKind, Write, Read};
use std::net::SocketAddr;
use mio::{Token, EventSet};
use mio::tcp::TcpStream;

use super::looper::{Looper, EventHandler, Eventer, LOOPER, TimerToken, TimeHandler};
use super::stream::Stream;
use super::listen::Listen;

pub trait ServiceBroker {

}

pub struct ServiceConfig {
    pub name : String,
    pub listen : Vec<String>,
    pub connect : Vec<String>,
}

pub struct ServiceBody {
    config : ServiceConfig,
    listens : HashMap<Token, Rc<RefCell<Listen>>>,
    streams : HashMap<Token, Rc<RefCell<Stream>>>,
    connecting : HashMap<TimerToken, SocketAddr>,
}

impl ServiceBody {
    fn new(config : ServiceConfig) -> Self {
        ServiceBody {
            config : config,
            listens : HashMap::new(),
            streams : HashMap::new(),
            connecting : HashMap::new(),
        }
    }
}

pub struct Service;
impl Service {
    pub fn new(config : ServiceConfig) -> ServiceRef {
        ServiceRef::new(Rc::new(RefCell::new(ServiceBody::new(config))))
    }
}

#[derive(Clone)]
pub struct ServiceRef {
    service : Rc<RefCell<ServiceBody>>,
}

impl ServiceRef {
    fn new(service : Rc<RefCell<ServiceBody>>) -> Self {
        ServiceRef {
            service : service
        }
    }
    pub fn start(&self) {
        let on_addrs : Vec<SocketAddr> = self.service.borrow_mut().config.connect.iter().map(|on| {
            SocketAddr::from_str(on).unwrap()
        }).collect();
        for addr in on_addrs {
            self.listen(addr);
        };
        let to_addrs : Vec<SocketAddr> = self.service.borrow_mut().config.connect.iter().map(|to| {
            SocketAddr::from_str(to).unwrap()
        }).collect();
        for addr in to_addrs {
            self.connect(addr);
        };
    }
    pub fn streams_count(&self) -> usize {
        self.service.borrow().streams.len()
    }
    fn listen(&self, on : SocketAddr) {
        let token = LOOPER.with(|looper| {
            looper.borrow_mut().register(Rc::new(RefCell::new(self.clone())))
        });
        self.service.borrow_mut().listens.insert(token, Rc::new(RefCell::new(Listen::new(token, on))));
    }
    fn connect(&self, to : SocketAddr) {
        let token = LOOPER.with(|looper| {
            looper.borrow_mut().register(Rc::new(RefCell::new(self.clone())))
        });
        self.service.borrow_mut().streams.insert(token, Rc::new(RefCell::new(Stream::new(token, TcpStream::connect(&to).unwrap(), true, to))));
    }
    fn timer_connect(&self, to : SocketAddr) {
        let token = LOOPER.with(|looper| {
            looper.borrow_mut().register_timer(Rc::new(RefCell::new(self.clone())), 5_000)
        });
        self.service.borrow_mut().connecting.insert(token, to);
    }
    fn on_ready_stream(&self, token : Token, es : EventSet) -> bool {
        let service = self.service.borrow();
        match service.streams.get(&token) {
            None => {
                false
            }
            Some(s) => {
                let mut stream = s.borrow_mut();
                let got = stream.got;
                stream.got = es;
                if es.is_error() || es.is_hup() {
                    stream.shutdown();
                } else {
                    if es.is_writable() {
                        if got == EventSet::none() {
                            trace!("stream connect");
                            info!("Service {} connected to {}", service.config.name, stream.peer_addr);
                        }
                        stream.flush();
                    }
                    if es.is_readable() {
                        trace!("stream read");
                        //TODO
                        //self.on_read(stream);
                    }
                }
                true
            }
        }
    }
    fn on_ready_listen(&self, token : Token, es : EventSet) -> bool {
        let mut service = &mut *self.service.borrow_mut();
        let listens = &service.listens;
        let mut streams = &mut service.streams;
        match listens.get(&token) {
            None => {
                false
            }
            Some(s) => {
                let mut listen = s.borrow_mut();
                let got = listen.got;
                listen.got = es;
                if es.is_error() || es.is_hup() {
                    trace!("listen error?");
                    listen.shutdown();
                } else {
                    if es.is_writable() {
                        trace!("listen writable?");
                    }
                    if es.is_readable() {
                        trace!("listen read");
                        match listen.listener.accept() {
                            Ok(Some((stream, peer))) => {
                                let token = LOOPER.with(|looper| {
                                    looper.borrow_mut().register(Rc::new(RefCell::new(self.clone())))
                                });
                                streams.insert(token,
                                    Rc::new(RefCell::new(Stream::new(token, stream, false, peer))));
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
                true
            }
        }
    }
    fn on_close_stream(&self, token : Token) -> bool {
        let mut r = self.service.borrow_mut().streams.remove(&token);
        match r {
            None => false,
            Some(s) => {
                let mut stream = s.borrow_mut();
                if stream.is_client {
                    let addr = stream.peer_addr;
                    self.timer_connect(addr);
                } else {
                    //TODO logout?
                }
                true
            }
        }
    }
    fn on_close_listen(&self, token : Token) -> bool {
        let mut r = self.service.borrow_mut().listens.remove(&token);
        match r {
            None => false,
            Some(s) => {
                let mut listen = s.borrow_mut();
                    //TODO re-listen?
                true
            }
        }
    }
}

impl EventHandler for ServiceRef {
    fn get_eventer(&mut self, token : Token) -> Option<Rc<RefCell<Eventer>>> {
        let service = self.service.borrow();
        match service.streams.get(&token) {
            Some(stream) => {
                Some(stream.clone())
            }
            None => {
                match service.listens.get(&token) {
                    Some(listen) => {
                        Some(listen.clone())
                    }
                    None => {
                        None
                    }
                }
            }
        }
    }
    fn on_ready(&mut self, token : Token, es : EventSet) {
        let ok = self.on_ready_stream(token, es) || self.on_ready_listen(token, es);
        if !ok {
            LOOPER.with(|looper| {
                looper.borrow_mut().reregister(token);
            });
        }
    }
    fn on_close(&mut self, token : Token) {
        self.on_close_stream(token) || self.on_close_listen(token);
    }
}

impl TimeHandler for ServiceRef {
    fn on_timer(&mut self, token : TimerToken) {
        let mut r = self.service.borrow_mut().connecting.remove(&token);
        match r {
            None => {
            }
            Some(addr) => {
                self.connect(addr);
            }
        }
    }
}

