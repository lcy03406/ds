use std::collections::HashMap;
use std::cell::{RefCell, Ref, RefMut};
use std::io::{Result, ErrorKind, Write, Read};
use std::net::SocketAddr;
use std::str::FromStr;
use mio::{Handler, EventLoop, Token, EventSet, PollOpt, Evented};
use mio::tcp::{TcpStream, TcpListener, Shutdown};

use buffer::Buffer;


trait Eventer {
    fn set_token(&mut self, token : Token);
    fn get_token(&self) -> Token;
    fn get_interest(&self) -> EventSet;
    fn get_evented(&mut self) -> &Evented;
    fn on_ready(&mut self, es : EventSet);
}

struct LoopHandler<'a> {
    eventers : HashMap<Token, Box<Eventer + 'a>>,
    token_counter: usize
}

impl<'a> Handler for LoopHandler<'a> {
    type Timeout = Token;
    type Message = ();

    fn ready(&mut self, e : &mut EventLoop<LoopHandler<'a>>, token : Token, es : EventSet) {
        match self.eventers.get_mut(&token) {
            None => {
            }
            Some(ter) => {
                ter.on_ready(es);
            }
        }
    }
}

struct Looper<'a> {
    event_loop : EventLoop<LoopHandler<'a>>,
    handler : LoopHandler<'a>,
}

impl<'a> Looper<'a> {
    pub fn new() -> Looper<'a> {
        Looper {
            event_loop : EventLoop::new().unwrap(),
            handler : LoopHandler {
                eventers : HashMap::new(),
                token_counter : 0,
            }
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let e = &mut self.event_loop;
        e.run(&mut self.handler)
    }

    pub fn register(&mut self, mut ter: Box<Eventer>) {
        //let token = self.new_token();
        //ter.set_token(token);
        //let es = ter.get_interest();
        //self.event_loop.register(ter.get_evented(), token, es, PollOpt::level());
        //self.handler.eventers.insert(token, ter);
    }

    pub fn deregister(&mut self, token : Token) {
        match self.handler.eventers.remove(&token) {
            Some(mut ter) => {
                self.event_loop.deregister(ter.get_evented());
            },
            None => {
                assert!(false);
            }
        }
    }

    fn new_token(&mut self) -> Token {
        while self.handler.eventers.contains_key(&Token(self.handler.token_counter)) {
            self.handler.token_counter += 1;
            if self.handler.token_counter == usize::max_value() {
                self.handler.token_counter = 0;
            }
        }
        return Token(self.handler.token_counter);
    }
}

trait ConnectionEventer {
    fn on_accept(&mut self);
    fn on_connect(&mut self);
    fn on_close(&mut self);
    fn on_read(&mut self);
}

struct Connection<'a, 'b : 'a, C : ConnectionEventer> {
    stream : TcpStream,
    token : Token,
    looper : RefMut<'a, Looper<'b>>,
    es : EventSet,
    wbuf : Buffer,
    conn : C,
}

impl<'a, 'b, C : ConnectionEventer> Connection<'a, 'b, C> {
    fn connect(mut looper : RefMut<'a, Looper<'b>>, conn : C, to : &str) -> Result<Connection<'a, 'b, C>> {
        Ok(Connection {
            stream : try!(TcpStream::connect(&SocketAddr::from_str(to).unwrap())),
            token : looper.new_token(),
            looper : looper,
            es : EventSet::none(),
            wbuf : Buffer::with_capacity(1024),
            conn : conn,
        })
    }
    pub fn close(&mut self) {
        self.stream.shutdown(Shutdown::Both);
        self.conn.on_close();
        self.looper.deregister(self.token); //will drop self
    }
    fn want_writable(&mut self) {
        self.es.insert(EventSet::writable());
    }
}

impl<'a, 'b, C : ConnectionEventer> Eventer for Connection<'a, 'b, C> {
    fn set_token(&mut self, token : Token) {
        self.token = token;
    }
    fn get_token(&self) -> Token {
        self.token
    }
    fn get_interest(&self) -> EventSet {
        self.es
    }
    fn get_evented(&mut self) -> &Evented {
        &self.stream
    }
    fn on_ready(&mut self, es : EventSet) {
        if es.is_error() || es.is_hup() {
            self.close();
        }
        if es.is_writable() {
            self.flush();
        }
        if es.is_readable() {
            self.conn.on_read();
        }
    }
}

impl<'a, 'b, C : ConnectionEventer> Write for Connection<'a, 'b, C> {
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
        }
    }
    let looper = RefCell::new(Looper::new());
    looper.borrow_mut();//.register(conn);
    let conn = Box::new(Connection::connect(looper.borrow_mut(), Conn{state:0}, "127.0.0.1:12306").unwrap());
    //looper.borrow_mut();//.register(conn);
}


