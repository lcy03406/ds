use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::mem::swap;
use mio::{Handler, EventLoop, Token, EventSet, PollOpt, Evented};
use env_logger;

thread_local!(pub static LOOPER: RefCell<Option<Looper>> = RefCell::new(None));

pub trait Eventer {
    fn registered(&self) -> EventSet;
    fn set_registered(&mut self, es : EventSet);
    fn interest(&self) -> EventSet;
    fn evented(&self) -> &Evented;
}

/*
pub struct EventHandle {
    token : Token,
    registered : EventSet,
    interest : EventSet,
}
*/

pub trait EventHandler {
    fn get_eventer(&mut self, token : Token) -> Option<Rc<RefCell<Eventer>>> ;
    fn on_ready(&mut self, token : Token, es : EventSet);
    fn on_close(&mut self, token : Token);
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]                                                                                   
pub struct TimerToken(pub usize);

pub trait TimeHandler {
    fn on_timer(&mut self, tt : TimerToken);
}

pub struct Looper {
    eventers : HashMap<Token, Rc<RefCell<EventHandler+'static>>>,
    token_counter: usize,
    to_reg : Vec<Token>,
    pending : Vec<Token>,
    timers : HashMap<TimerToken, (Rc<RefCell<TimeHandler+'static>>, u64)>,
    timer_counter: usize,
    timer_to_reg : Vec<TimerToken>,
}

impl Looper {
    pub fn new() -> Self {
        Looper {
            eventers : HashMap::new(),
            token_counter : 0,
            to_reg : Vec::new(),
            pending : Vec::new(),
            timers : HashMap::new(),
            timer_counter : 0,
            timer_to_reg : Vec::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.eventers.is_empty() && self.timers.is_empty()
    }

    fn has_pending(&self) -> bool {
        !(self.pending.is_empty() && self.to_reg.is_empty() && self.timer_to_reg.is_empty())
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

    fn get_handler(&mut self, token : Token) -> Option<Rc<RefCell<EventHandler>>> {
        match self.eventers.get(&token) {
            None => {
                None
            }
            Some(h) => {
                Some(h.clone())
            }
        }
    }

    fn get_eventer(&mut self, token : Token) -> Option<Rc<RefCell<Eventer>>> {
        match self.eventers.get(&token) {
            None => {
                None
            }
            Some(h) => {
                h.borrow_mut().get_eventer(token)
            }
        }
    }

    pub fn register(&mut self, h : Rc<RefCell<EventHandler>>) -> Token {
        let token = self.new_token();
        self.eventers.insert(token, h);
        self.to_reg.push(token);
        trace!("looper register {:?}", token);
        token
    }

    pub fn reregister(&mut self, token : Token) {
        debug_assert!(self.eventers.contains_key(&token));
        match self.pending.binary_search(&token) {
            Ok(_) => {
                trace!("looper reregister already {:?}", token);
            }
            Err(idx) => {
                trace!("looper reregister pending {:?}", token);
                self.pending.insert(idx, token);
            }
        }
    }

    fn new_timer(&mut self) -> TimerToken {
        loop {
            self.timer_counter += 1;
            if self.timer_counter == usize::max_value() {
                self.timer_counter = 1;
            }
            let t = TimerToken(self.timer_counter);
            if !self.timers.contains_key(&t) {
                return t;
            }
        }
    }

    pub fn register_timer(&mut self, h : Rc<RefCell<TimeHandler>>, delay : u64) -> TimerToken {
        let token = self.new_timer();
        self.timers.insert(token, (h, delay));
        self.timer_to_reg.push(token);
        trace!("looper register timer {:?}", token);
        token
    }
}

pub struct LoopHandler;

impl LoopHandler {
    pub fn run(&mut self) {
        let mut el = EventLoop::new().unwrap();
        self.tick(&mut el);
        el.run(self).unwrap();
    }
    fn loop_register(&mut self, el : &mut EventLoop<Self>, lp : &RefCell<Option<Looper>>) {
        let mut borrow = lp.borrow_mut();
        let mut looper = borrow.as_mut().unwrap();
        if looper.to_reg.is_empty() {
            return;
        }
        let mut to_reg = Vec::new();
        swap(&mut to_reg, &mut looper.to_reg);
        for token in to_reg {
            match looper.get_eventer(token) {
                None => {
                    trace!("loop_register none? {:?}", token);
                    looper.reregister(token);
                    continue;
                }
                Some(tt) => {
                    let t = tt.borrow();
                    let es = t.interest();
                    el.register(t.evented(), token, es, PollOpt::edge()).unwrap();
                    trace!("event_loop register {:?}", token);
                }
            }
        }
    }
    fn loop_reregister(&mut self, el : &mut EventLoop<Self>, lp : &RefCell<Option<Looper>>) {
        let mut closed = HashMap::new();
        {
            let mut borrow = lp.borrow_mut();
            let mut looper = borrow.as_mut().unwrap();
            if looper.pending.is_empty() {
                return;
            }
            let mut pending = Vec::new();
            swap(&mut pending, &mut looper.pending);
            for token in pending {
                match looper.get_eventer(token) {
                    None => {
                        trace!("loop_reregister none? {:?}", token);
                        match looper.eventers.remove(&token) {
                            None => {
                            }
                            Some(h) => {
                                closed.insert(token, h);
                            }
                        }
                    }
                    Some(tt) => {
                        let mut t = tt.borrow_mut();
                        let es = t.interest();
                        if es == EventSet::none() {
                            el.deregister(t.evented()).unwrap();
                            trace!("event_loop deregister {:?}", token);
                            match looper.eventers.remove(&token) {
                                None => {
                                }
                                Some(h) => {
                                    closed.insert(token, h);
                                }
                            }
                        } else if t.registered() != es {
                            el.reregister(t.evented(), token, es, PollOpt::edge()).unwrap();
                            trace!("event_loop reregister {:?} {:?}", token, es);
                            t.set_registered(es);
                        } else {
                            trace!("event_loop reregister same? {:?} {:?}", token, es);
                        }
                    }
                }
            }
        }
        for (token, h) in closed {
            trace!("handler on_close {:?}", token);
            h.borrow_mut().on_close(token);
            trace!("handler on_close done {:?}", token);
        }
    }
    fn loop_register_timer(&mut self, el : &mut EventLoop<Self>, lp : &RefCell<Option<Looper>>) {
        let mut borrow = lp.borrow_mut();
        let mut looper = borrow.as_mut().unwrap();
        if looper.timer_to_reg.is_empty() {
            return;
        }
        let mut timer_to_reg = Vec::new();
        swap(&mut timer_to_reg, &mut looper.timer_to_reg);
        for token in timer_to_reg {
            match looper.timers.get(&token) {
                None => {
                    trace!("loop_register_timer none? {:?}", token);
                    continue;
                }
                Some(&(_, delay)) => {
                    el.timeout_ms(token, delay).unwrap();
                    trace!("event_loop register timer {:?} {}", token, delay);
                }
            }
        }
    }
}

impl Handler for LoopHandler {
    type Timeout = TimerToken;
    type Message = ();

    fn ready(&mut self, _ : &mut EventLoop<Self>, token : Token, es : EventSet) {
        match LOOPER.with(|looper| {
            looper.borrow_mut().as_mut().unwrap().get_handler(token)
        }) {
            None => {
                trace!("handler on_ready none? {:?} {:?}", token, es);
            }
            Some(h) => {
                trace!("handler on_ready {:?} {:?}", token, es);
                h.borrow_mut().on_ready(token, es);
                trace!("handler on_ready done {:?}", token);
            }
        };
    }
    fn timeout(&mut self, _ : &mut EventLoop<Self>, token : TimerToken) {
        match LOOPER.with(|looper| {
            looper.borrow_mut().as_mut().unwrap().timers.remove(&token)
        }) {
            None => {
                trace!("handler on_timer none? {:?}", token);
            }
            Some((h, _)) => {
                trace!("handler on_timer {:?}", token);
                h.borrow_mut().on_timer(token);
                trace!("handler on_ready done {:?}", token);
            }
        };
    }
    fn tick(&mut self, el: &mut EventLoop<Self>) {
        trace!("handler tick");
        LOOPER.with(|looper| {
            while looper.borrow().as_ref().unwrap().has_pending() {
                self.loop_reregister(el, &looper);
                self.loop_register(el, &looper);
                self.loop_register_timer(el, &looper);
            }
            if looper.borrow().as_ref().unwrap().is_empty() {
                trace!("handler shutdown");
                el.shutdown();
            } else {
                trace!("handler eventes {:?}, timers {:?}",
                       looper.borrow().as_ref().unwrap().eventers.len(),
                       looper.borrow().as_ref().unwrap().timers.len());
            }
        });
    }
}

pub fn init() {
    env_logger::init().ok();
    LOOPER.with(|looper| {
        if looper.borrow().is_none() {
            *looper.borrow_mut() = Some(Looper::new());
        }
    })
}

pub fn run_loop() {
    let mut handler = LoopHandler;
    handler.run();
}
