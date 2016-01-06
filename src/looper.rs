use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::cell::{RefCell, RefMut};
use std::mem::swap;
use std::io::{Result, ErrorKind, Write, Read};
use std::net::SocketAddr;
use std::str::FromStr;
use mio::{Handler, EventLoop, Token, EventSet, PollOpt, Evented};
use mio::tcp::{TcpStream, TcpListener, Shutdown};

use buffer::Buffer;

struct Looper<'a> {
    eventers : HashMap<Token, Rc<RefCell<Eventer<'a>+'a>>>,
    token_counter: usize,
    current : Token,
    to_reg : Vec<Token>,
    pending : Vec<Token>,
}

impl<'a> Looper<'a> {
    pub fn new() -> Self {
        Looper {
            eventers : HashMap::new(),
            token_counter : 0,
            current : Token(0),
            to_reg : Vec::new(),
            pending : Vec::new(),
        }
    }

    pub fn register(looper : &Rc<RefCell<Looper<'a>>>, ter: Rc<RefCell<Eventer<'a>+'a>>) {
        let mut myself = looper.borrow_mut();
        let token = myself.new_token();
        ter.borrow_mut().looper_and_token().token = token;
        myself.eventers.insert(token, ter);
        myself.to_reg.push(token);
        trace!("looper register {:?}", token);
    }

    pub fn reregister(&mut self, token : Token) {
        debug_assert!(self.eventers.contains_key(&token));
        if token == self.current {
            trace!("looper reregister current {:?}", token);
            return;
        }
        match self.pending.binary_search(&token) {
            Ok(idx) => {
                trace!("looper reregister already {:?}", token);
            }
            Err(idx) => {
                trace!("looper reregister pending {:?}", token);
                self.pending.insert(idx, token);
            }
        }
    }

    fn new_token(&mut self) -> Token {
        loop {
            self.token_counter += 1;
            if self.token_counter == usize::max_value() {
                self.token_counter = 1;
            }
            if !self.eventers.contains_key(&Token(self.token_counter)) {
                return Token(self.token_counter);
            }
        }
    }
}

struct LoopHandler<'a> {
    looper : Rc<RefCell<Looper<'a>>>
}

impl<'a> LoopHandler<'a> {
    fn new() -> Self {
        LoopHandler {
            looper : Rc::new(RefCell::new(Looper::new()))
        }
    }
    fn loop_register(&mut self, el : &mut EventLoop<Self>) {
        let mut looper = self.looper.borrow_mut();
        for token in &looper.to_reg {
            match looper.eventers.get(&token) {
                None => {
                    assert!(false);
                }
                Some(ter) => {
                    let t = ter.borrow_mut();
                    let es = t.interest();
                    el.register(t.evented(), *token, es, PollOpt::level());
                    trace!("event_loop register {:?}", token);
                }
            }
        }
        looper.to_reg.clear();
    }
    fn loop_reregister(&mut self, el : &mut EventLoop<Self>) {
        let mut pending;
        {
            let swap_pending = &mut self.looper.borrow_mut().pending;
            if swap_pending.is_empty() {
                return;
            }
            pending = Vec::new();
            swap(&mut pending, swap_pending);
        }
        let mut ter;
        for token in pending {
            match self.looper.borrow().eventers.get(&token) {
                None => {
                    assert!(false);
                    continue;
                }
                Some(e) => {
                    ter = e.clone();
                }
            }
            self.loop_reregister_eventer(el, &mut *ter.borrow_mut());
        }
    }
    fn loop_reregister_eventer(&mut self, el : &mut EventLoop<Self>, t : &mut Eventer) {
        let es = t.interest();
        let token : Token = t.looper_and_token().token;
        if es == EventSet::none() {
            el.deregister(t.evented());
            trace!("event_loop deregister {:?}", token);
            let mut looper = self.looper.borrow_mut();
            looper.eventers.remove(&token);
        } else if t.looper_and_token().registered != es {
            el.reregister(t.evented(), token, es, PollOpt::level());
            trace!("event_loop reregister {:?} {:?}", token, es);
            t.looper_and_token().registered = es;
        }
    }
}

impl<'a> Handler for LoopHandler<'a> {
    type Timeout = Token;
    type Message = ();

    fn ready(&mut self, el : &mut EventLoop<Self>, token : Token, es : EventSet) {
        let mut ter;
        match self.looper.borrow().eventers.get(&token) {
            None => {
                assert!(false);
                return;
            }
            Some(e) => {
                ter = e.clone();
            }
        }
        let mut t = ter.borrow_mut();
        trace!("handler ready {:?} {:?}", token, es);
        self.looper.borrow_mut().current = token;
        t.on_ready(es);
        self.looper.borrow_mut().current = Token(0);
        trace!("handler ready done");
        self.loop_reregister_eventer(el, &mut *t);
        self.loop_register(el);
    }
    fn tick(&mut self, el: &mut EventLoop<Self>) {
        trace!("handler tick");
        self.loop_register(el);
        self.loop_reregister(el);
        if self.looper.borrow().eventers.is_empty() {
            trace!("handler shutdown");
            el.shutdown();
        } else {
            trace!("handler eventes {:}", self.looper.borrow().eventers.len());
        }
    }
}

struct LooperAndToken<'a> {
    looper : Weak<RefCell<Looper<'a>>>,
    token : Token,
    registered : EventSet,
}

trait Eventer<'a> {
    fn looper_and_token(&mut self) -> &mut LooperAndToken<'a>;
    fn interest(&self) -> EventSet;
    fn evented(&self) -> &Evented;
    fn on_ready(&mut self, es : EventSet);

    fn reregister(&mut self) {
        let lt = self.looper_and_token();
        match Weak::upgrade(&lt.looper) {
            Some(ref loo) => {
                loo.borrow_mut().reregister(lt.token);
            }
            None => {
            }
        }
    }
}

trait ConnectionEventer {
    fn on_accept(&mut self);
    fn on_connect(&mut self);
    fn on_close(&mut self);
    fn on_read(&mut self);
}

struct Connection<'a, C : ConnectionEventer + 'a> {
    lt : LooperAndToken<'a>,
    interest : EventSet,
    stream : TcpStream,
    wbuf : Buffer,
    conn : C,
}

impl<'a, C : ConnectionEventer + 'a> Connection<'a, C> {
    fn connect(looper : &Rc<RefCell<Looper<'a>>>, conn : C, to : &str) -> Result<()> {
        let ter = Rc::new(RefCell::new(Connection {
            lt : LooperAndToken {
                looper : Rc::downgrade(looper),
                token : Token(0),
                registered : EventSet::none(),
            },
            interest : EventSet::all() ,
            stream : try!(TcpStream::connect(&SocketAddr::from_str(to).unwrap())),
            wbuf : Buffer::with_capacity(1024),
            conn : conn,
        }));
        Looper::register(looper, ter);
        Ok(())
    }
    pub fn close(&mut self) {
        self.stream.shutdown(Shutdown::Both);
        self.conn.on_close();
        self.interest = EventSet::none();
        self.reregister();
    }
    fn want_writable(&mut self) {
        self.interest.insert(EventSet::writable());
        self.reregister();
    }
}

impl<'a, C : ConnectionEventer + 'a> Eventer<'a> for Connection<'a, C> {
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
        if es.is_error() || es.is_hup() {
            self.close();
        }
        if es.is_writable() {
            self.interest.remove(EventSet::writable());
            self.flush();
        }
        if es.is_readable() {
            self.conn.on_read();
        }
    }
}

impl<'a, C : ConnectionEventer + 'a> Write for Connection<'a, C> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let len = buf.len();
        if self.wbuf.is_empty() {
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

#[test]
fn test_reg() {
    
    use env_logger;
    env_logger::init().unwrap();

    struct Conn {
        state : i32
    }
    
    impl ConnectionEventer for Conn {
        fn on_accept(&mut self) {
        }
        fn on_connect(&mut self) {
        }
        fn on_close(&mut self) {
        }
        fn on_read(&mut self) {
            self.write("hello server");
        }
    }
    let mut handler = LoopHandler::new();
    Connection::connect(&handler.looper, Conn{state:0}, "127.0.0.1:12306").unwrap();
    trace!("run");
    let mut el = EventLoop::new().unwrap();
    handler.tick(&mut el);
    el.run(&mut handler);
}


