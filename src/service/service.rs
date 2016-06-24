use std::rc::{Rc};
use std::cell::{RefCell};
use std::str::FromStr;
use std::collections::HashMap;
use std::io::{Write, BufRead};
use std::net::SocketAddr;
use std::fmt::Debug;
use mio::{Token, EventSet};
use mio::tcp::TcpStream;

use super::looper::{LOOPER, EventHandler, Eventer, TimerToken, TimeHandler};
use super::stream::Stream;
use super::listen::Listen;
use super::config::ServiceConfig;

pub trait ServiceStreamer {
    type Packet;
    type Error : Debug;
    fn write_packet(packet : &Self::Packet, writer : &mut Write) ->Result<(), Self::Error>;
    fn read_packet(reader : &mut BufRead) -> Result<Option<Self::Packet>, Self::Error>;
}

pub trait ServiceHandler {
    type Packet;
    type Streamer : ServiceStreamer<Packet=Self::Packet>;
    fn connected(&self, token : Token);
    fn disconnected(&self, token : Token);
    fn incoming(&self, token : Token, packet : Self::Packet);
    fn outgoing(&self, token : Token, packet : &Self::Packet);
}

pub struct ServiceBody {
    name : String,
    listens : HashMap<Token, Rc<RefCell<Listen>>>,
    streams : HashMap<Token, Rc<RefCell<Stream>>>,
    connecting : HashMap<TimerToken, SocketAddr>,
}

impl ServiceBody {
    fn new() -> Self {
        ServiceBody {
            name : String::new(),
            listens : HashMap::new(),
            streams : HashMap::new(),
            connecting : HashMap::new(),
        }
    }
}

pub struct ServiceRef<H : ServiceHandler + 'static> {
    service : Rc<RefCell<ServiceBody>>,
    handler : Rc<RefCell<H>>,
}

impl<H: ServiceHandler + 'static> Clone for ServiceRef<H> {
    fn clone(&self) -> Self {
        ServiceRef {
            service : self.service.clone(),
            handler : self.handler.clone(),
        }
    }
}

impl<H: ServiceHandler + 'static > ServiceRef<H> {
    pub fn new(h : H) -> ServiceRef<H> {
        ServiceRef {
            service : Rc::new(RefCell::new(ServiceBody::new())),
            handler : Rc::new(RefCell::new(h)),
        }
    }
    pub fn start(&self, config : ServiceConfig) {
        self.service.borrow_mut().name = config.name;
        let on_addrs : Vec<SocketAddr> = config.listen.iter().map(|on| {
            SocketAddr::from_str(on).unwrap()
        }).collect();
        for addr in on_addrs {
            self.listen(addr);
        };
        let to_addrs : Vec<SocketAddr> = config.connect.iter().map(|to| {
            SocketAddr::from_str(to).unwrap()
        }).collect();
        for addr in to_addrs {
            self.connect(addr, true);
        };
    }
    pub fn exit(&self) {
        let mut service = self.service.borrow_mut();
        for stream in service.streams.values() {
            stream.borrow_mut().reconnect = false;
            stream.borrow_mut().shutdown();
        }
        //service.streams.clear();
        for listen in service.listens.values() {
            listen.borrow_mut().shutdown();
        }
        //service.listens.clear();
        for connect in service.connecting.keys() {
            LOOPER.with(|looper| {
                looper.borrow_mut().as_mut().unwrap().deregister_timer(*connect)
            });
        }
        service.connecting.clear();
    }
    pub fn write(&self, token : Token, packet : &H::Packet) {
        let stream = match self.service.borrow_mut().streams.get_mut(&token) {
            None => {
                trace!("service write none {:?}", token);
                return;
            }
            Some(s) => {
                s.clone()
            }
        };
        trace!("service handler outgoing begin {:?}", token);
        self.handler.borrow().outgoing(token, packet);
        trace!("service handler outgoing end {:?}", token);
        let r = H::Streamer::write_packet(packet, &mut *stream.borrow_mut());
        match r {
            Ok(_) => {
                trace!("service write ok {:?}", token);
                stream.borrow_mut().flush().ok();
            }
            Err(e) => {
                trace!("service write err {:?} {:?}", token, e);
            }
        }
    }
    pub fn broadcast(&self, packet : &H::Packet) {
        let streams = self.service.borrow_mut().streams.clone();
        for (token, stream) in streams {
            trace!("service handler outgoing begin {:?}", token);
            self.handler.borrow().outgoing(token, packet);
            trace!("service handler outgoing end {:?}", token);
            let r = H::Streamer::write_packet(packet, &mut *stream.borrow_mut());
            match r {
                Ok(_) => {
                    trace!("service write ok {:?}", token);
                    stream.borrow_mut().flush().ok();
                }
                Err(e) => {
                    trace!("service write err {:?} {:?}", token, e);
                }
            }
        }
    }
    pub fn shutdown(&self, token : Token) {
        match self.service.borrow_mut().streams.get_mut(&token) {
            None => {
                trace!("service shutdown none {:?}", token);
                return;
            }
            Some(s) => {
                trace!("service shutdown {:?}", token);
                s.borrow_mut().reconnect = false;
                s.borrow_mut().shutdown();
            }
        };
    }
    pub fn streams_count(&self) -> usize {
        self.service.borrow().streams.len()
    }
    fn listen(&self, on : SocketAddr) {
        let c : ServiceRef<H> = self.clone();
        let token = LOOPER.with(|looper| {
            looper.borrow_mut().as_mut().unwrap().register(Rc::new(RefCell::new(c)))
        });
        self.service.borrow_mut().listens.insert(token, Rc::new(RefCell::new(Listen::new(token, on))));
    }
    fn connect(&self, to : SocketAddr, reconnect : bool) {
        let token = LOOPER.with(|looper| {
            looper.borrow_mut().as_mut().unwrap().register(Rc::new(RefCell::new(self.clone())))
        });
        let stream = Stream::new(token, TcpStream::connect(&to).unwrap(), true, reconnect, to);
        self.service.borrow_mut().streams.insert(token, Rc::new(RefCell::new(stream)));
    }
    fn timer_connect(&self, to : SocketAddr) {
        let token = LOOPER.with(|looper| {
            looper.borrow_mut().as_mut().unwrap().register_timer(Rc::new(RefCell::new(self.clone())), 5_000)
        });
        self.service.borrow_mut().connecting.insert(token, to);
    }
    fn on_ready_stream(&self, token : Token, es : EventSet) -> bool {
        let mut  new_connected = false;
        let mut packets = Vec::new();
        {
            let service = self.service.borrow();
            match service.streams.get(&token) {
                None => {
                    return false;
                }
                Some(s) => {
                    let mut stream = s.borrow_mut();
                    let got = stream.got;
                    stream.got = es;
                    if es.is_error() || es.is_hup() {
                        stream.shutdown();
                    } else {
                        if es.is_writable() {
                            if stream.connecting {
                                info!("Service {} connected to {:?} {}", service.name, token, stream.peer_addr);
                                new_connected = true;
                                stream.connecting = false;
                            }
                            stream.flush().ok();
                        }
                        if es.is_readable() {
                            trace!("stream read");
                            loop {
                                match H::Streamer::read_packet(&mut *stream) {
                                    Ok(Some(p)) => {
                                        packets.push(p);
                                    }
                                    Ok(None) => {
                                        break;
                                    }
                                    Err(e) => {
                                        trace!("service read err {:?} {:?}", token, e);
                                        stream.shutdown();
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if new_connected {
            trace!("service handler connected begin {:?}", token);
            self.handler.borrow().connected(token);
            trace!("service handler connected end {:?}", token);
        }
        for packet in packets {
            trace!("service handler incoming begin {:?}", token);
            self.handler.borrow().incoming(token, packet);
            trace!("service handler incoming end {:?}", token);
        }
        true
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
                if es.is_error() || es.is_hup() {
                    trace!("listen error?");
                    listen.shutdown();
                } else {
                    if es.is_writable() {
                        trace!("listen writable?");
                    }
                    if es.is_readable() {
                        trace!("listen read");
                        loop {
                            match listen.listener.accept() {
                                Ok(Some((stream, peer))) => {
                                    let token = LOOPER.with(|looper| {
                                        looper.borrow_mut().as_mut().unwrap().register(Rc::new(RefCell::new(self.clone())))
                                    });
                                    streams.insert(token,
                                        Rc::new(RefCell::new(Stream::new(token, stream, false, false, peer))));
                                }
                                Ok(None) => {
                                    trace!("listen accept none");
                                    break;
                                }
                                Err(e) => {
                                    trace!("listen accept err {:?}", e);
                                }
                            }
                        }
                    }
                }
                true
            }
        }
    }
    fn on_close_stream(&self, token : Token) -> bool {
        let addr = {
            let mut service = self.service.borrow_mut();
            let r = service.streams.remove(&token);
            match r {
                None => {
                    return false;
                }
                Some(s) => {
                    let stream = s.borrow();
                    if stream.is_client && stream.reconnect {
                        info!("Service {} disconnected from {:?} {}", service.name, token, stream.peer_addr);
                        Some(stream.peer_addr)
                    } else {
                        None
                    }
                }
            }
        };
        match addr {
            Some(addr) => {
                self.timer_connect(addr);
            }
            None => {
            }
        }
        trace!("service handler disconnected begin {:?}", token);
        self.handler.borrow().disconnected(token);
        trace!("service handler disconnected end {:?}", token);
        true
    }
    fn on_close_listen(&self, token : Token) -> bool {
        let r = self.service.borrow_mut().listens.remove(&token);
        match r {
            None => false,
            Some(_) => {
                trace!("service close listen {:?}", token);
                    //TODO re-listen?
                true
            }
        }
    }
}

impl<H: ServiceHandler + 'static> EventHandler for ServiceRef<H> {
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
                looper.borrow_mut().as_mut().unwrap().reregister(token);
            });
        }
    }
    fn on_close(&mut self, token : Token) {
        self.on_close_stream(token) || self.on_close_listen(token);
    }
}

impl<H: ServiceHandler + 'static> TimeHandler for ServiceRef<H> {
    fn on_timer(&mut self, token : TimerToken) {
        let r = self.service.borrow_mut().connecting.remove(&token);
        match r {
            None => {
            }
            Some(addr) => {
                self.connect(addr, true);
            }
        }
    }
}

#[macro_export]
macro_rules! service_define {
    ($n:ident : $t:ty) => {
        thread_local!(static $n : ::std::cell::RefCell<Option<ServiceRef<$t>>> = ::std::cell::RefCell::new(None));
    };
    (pub $n:ident : $t:ty) => {
        thread_local!(pub static $n : ::std::cell::RefCell<Option<ServiceRef<$t>>> = ::std::cell::RefCell::new(None));
    };
}
#[macro_export]
macro_rules! service_start {
    ($n:ident, $h:expr, $c:expr) => {
        $n.with(move |s| {
            assert!(s.borrow().is_none());
            *s.borrow_mut() = Some(ServiceRef::new($h));
            s.borrow_mut().as_mut().unwrap().start($c)
        })
    }
}
#[macro_export]
macro_rules! service_exit {
    ($n:ident) => {
        $n.with(|s| s.borrow_mut().as_mut().unwrap().exit())
    }
}
#[macro_export]
macro_rules! service_write {
    ($n:ident , $t:expr, $p:expr) => {
        $n.with(|s| s.borrow_mut().as_mut().unwrap().write($t, $p))
    }
}
#[macro_export]
macro_rules! service_broadcast {
    ($n:ident , $p:expr) => {
        $n.with(|s| s.borrow_mut().as_mut().unwrap().broadcast($p))
    }
}
#[macro_export]
macro_rules! service_shutdown {
    ($n:ident , $t:expr) => {
        $n.with(|s| s.borrow_mut().as_mut().unwrap().shutdown($t))
    }
}

