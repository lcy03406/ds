use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::cell::{RefCell, RefMut};
use std::mem::swap;
use std::io::{Result, ErrorKind, Write, Read};
use mio::{Handler, EventLoop, Token, EventSet, PollOpt, Evented};

pub trait Eventer<'a> {
    fn looper_and_token(&mut self) -> &mut LooperAndToken<'a>;
    fn interest(&self) -> EventSet;
    fn evented(&self) -> &Evented;
    fn on_ready(&mut self, es : EventSet);
    fn on_close(&mut self);
}

pub struct LooperAndToken<'a> {
    pub looper : Weak<RefCell<Looper<'a>>>,
    pub token : Token,
    pub registered : EventSet,
}

impl<'a> LooperAndToken<'a> {
    pub fn reregister(&mut self) {
        match Weak::upgrade(&self.looper) {
            Some(ref loo) => {
                loo.borrow_mut().reregister(self.token);
            }
            None => {
            }
        }
    }
}

pub struct Looper<'a> {
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

    pub fn register(looper : &Rc<RefCell<Looper<'a>>>, ter: &Rc<RefCell<Eventer<'a>+'a>>) {
        let mut myself = looper.borrow_mut();
        let token = myself.new_token();
        ter.borrow_mut().looper_and_token().token = token;
        myself.eventers.insert(token, ter.clone());
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

pub struct LoopHandler<'a> {
    pub looper : Rc<RefCell<Looper<'a>>>
}

impl<'a> LoopHandler<'a> {
    pub fn new() -> Self {
        LoopHandler {
            looper : Rc::new(RefCell::new(Looper::new()))
        }
    }
    pub fn run(&mut self) {
        let mut el = EventLoop::new().unwrap();
        self.tick(&mut el);
        el.run(self);
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
                    el.register(t.evented(), *token, es, PollOpt::edge());
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
            self.looper.borrow_mut().eventers.remove(&token);
            trace!("handler on_close {:?}", token);
            self.looper.borrow_mut().current = token;
            t.on_close();
            self.looper.borrow_mut().current = Token(0);
            trace!("handler on_close done {:?}", token);
        } else if t.looper_and_token().registered != es {
            el.reregister(t.evented(), token, es, PollOpt::edge());
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
        trace!("handler on_ready {:?} {:?}", token, es);
        self.looper.borrow_mut().current = token;
        t.on_ready(es);
        self.looper.borrow_mut().current = Token(0);
        trace!("handler on_ready done {:?}", token);
        self.loop_reregister_eventer(el, &mut *t);
        self.loop_register(el);
    }
    fn tick(&mut self, el: &mut EventLoop<Self>) {
        trace!("handler tick");
        self.loop_reregister(el);
        self.loop_register(el);
        if self.looper.borrow().eventers.is_empty() {
            trace!("handler shutdown");
            el.shutdown();
        } else {
            trace!("handler eventes {:}", self.looper.borrow().eventers.len());
        }
    }
}
