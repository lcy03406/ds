#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

#[macro_use]
extern crate ds;
#[macro_use]
extern crate log;
extern crate serde;
extern crate time;

use std::rc::Rc;
use std::cell::RefCell;
use std::io::Write;
use std::str::FromStr;
use std::num::ParseIntError;
use std::env;

use serde::{Serializer, Deserializer};

use ds::service::{Token, ServiceHandler, ServiceRef, ServiceConfig, init, run_loop};
use ds::streamer::pw::PwStreamer;

use time::PreciseTime;

#[derive(Serialize, Deserialize, Debug)]
struct Key {
    roleid : u64,
    timestamp : u32,
    passcode : u32,
}

impl Key {
    fn to_string(&self) -> String {
        format!("@@cache.{:016X}|{:08X}|{:08X}", self.roleid, self.timestamp, self.passcode)
    }
}

impl FromStr for Key {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const P : usize = 8;
        let roleid = try!(u64::from_str_radix(&s[P..P+16], 16));
        let timestamp = try!(u32::from_str_radix(&s[P+17..P+17+8], 16));
        let passcode = try!(u32::from_str_radix(&s[P+17+9..P+17+9+8], 16));
        Ok(Key { roleid : roleid, timestamp : timestamp, passcode : passcode })
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum ProtocolFrom7001 {
    Set(Key, Vec<u8>),
    SetRe(Key, i32),
    Get(u64, Key),
    GetRe(u64, Key, i32, Vec<u8>),
}

struct Stat {
    conn : u32,
    sent : u32,
    got : u32,
    time_total : u32,
    time_max : u32,
    now : u32,
}

impl Stat {
    fn new() ->Self {
        Stat {
            conn : 0,
            sent : 0,
            got : 0,
            time_total : 0,
            time_max : 0,
            now : 0,
        }
    }
    fn print(&self) {
        info!("now={} conn={} sent={} got={} mean_time={} max_time={}", self.now, self.conn, self.sent, self.got, self.time_total/self.got, self.time_max);
    }
}

struct ClientService {
    begin : PreciseTime,
    concur : u32,
    total : u32,
    stat_set : Rc<RefCell<Stat>>,
    stat_get : Rc<RefCell<Stat>>,
}

impl ClientService {
    fn new(concur : u32, total : u32) -> Self {
        ClientService {
            begin : PreciseTime::now(),
            concur : concur,
            total : total,
            stat_set : Rc::new(RefCell::new(Stat::new())),
            stat_get : Rc::new(RefCell::new(Stat::new())),
        }
    }
    fn new_key(&self) -> Key {
        Key {
            roleid : 24678,
            timestamp : time::get_time().sec as u32,
            passcode : self.begin.to(PreciseTime::now()).num_milliseconds() as u32,
        }
    }
    fn send_set(&self, token : Token) {
        let mut stat = self.stat_set.borrow_mut();
        if stat.sent < self.total {
            let key = self.new_key();
            let req = ProtocolFrom7001::Set(key, vec![0xCF;5555]);
            service_write!(CLIENT_SERVICE, token, &req);
            stat.sent += 1;
        }
    }
    fn send_get(&self, token : Token, key : Key) {
        let mut stat = self.stat_get.borrow_mut();
        let req = ProtocolFrom7001::Get(21476, key);
        service_write!(CLIENT_SERVICE, token, &req);
        stat.sent += 1;
    }
}

service_define!(CLIENT_SERVICE : ClientService);

impl ServiceHandler for ClientService {
    type Packet = ProtocolFrom7001;
    type Streamer = PwStreamer<Self::Packet>;
    fn connected(&self, token : Token) {
        trace!("client_service {:?} connected", token);
        self.stat_set.borrow_mut().conn += 1;
        for _ in 0..self.concur {
            self.send_set(token);
        }
    }
    fn disconnected(&self, token : Token) {
        trace!("client_service {:?} disconnected", token);
    }
    fn incoming(&self, token : Token, packet : Self::Packet) {
        //trace!("client_service {:?} incoming", token);
        match packet {
            ProtocolFrom7001::SetRe(key, ret) => {
                let now = self.begin.to(PreciseTime::now()).num_milliseconds() as u32;
                {
                    let mut stat = self.stat_set.borrow_mut();
                    stat.now = now;
                    stat.got += 1;
                    let t = now - key.passcode;
                    stat.time_total += t;
                    if stat.time_max < t {
                        stat.time_max = t;
                    }
                    if stat.got >= self.total {
                        stat.print();
                    }
                }
                self.send_get(token, key);
            }
            ProtocolFrom7001::GetRe(roleid, key, ret, data) => {
                let now = self.begin.to(PreciseTime::now()).num_milliseconds() as u32;
                {
                    let mut stat = self.stat_get.borrow_mut();
                    stat.now = now;
                    stat.got += 1;
                    let t = now - key.passcode;
                    stat.time_total += t;
                    if stat.time_max < t {
                        stat.time_max = t;
                    }
                    if stat.got >= self.total {
                        stat.print();
                        service_exit!(CLIENT_SERVICE);
                    }
                }
                self.send_set(token);
            }
            _ => {
                trace!("client_service {:?} fail", token);
                service_shutdown!(CLIENT_SERVICE, token);
                service_exit!(CLIENT_SERVICE);
            }
        }
    }
    fn outgoing(&self, _token : Token, _packet : &Self::Packet) {
    }
}

fn main() {
    let args : Vec<_> = env::args().collect();
    let concur : u32 = args[1].parse().unwrap();
    let total : u32 = args[2].parse().unwrap();
    init();
    let client_service = ClientService::new(concur, total);
    service_start!(CLIENT_SERVICE, client_service, ServiceConfig::from_file("config.toml", "client_service"));
    run_loop();
}

