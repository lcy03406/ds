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
use std::collections::BTreeMap;
use std::io::Write;
use std::str::FromStr;
use std::num::ParseIntError;

use serde::{Serializer, Deserializer};

use ds::service::{Token, ServiceHandler, ServiceRef, ServiceConfig, init, run_loop};
use ds::streamer::pw::PwStreamer;
use ds::streamer::memcached::MemcachedStreamer;
use ds::streamer::memcached;

use time::{Duration, PreciseTime};


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
}

impl Stat {
    fn print(&self) {
        trace!("conn={} sent={} got={} mean_time={} max_time={}", self.conn, self.sent, self.got, self.time_total/self.got, self.time_max);
    }
}

struct ClientService {
    packet5k : Vec<u8>,
    begin : PreciseTime,
    stat : Rc<RefCell<Stat>>,
}

impl ClientService {
    fn new() -> Self {
        ClientService {
            packet5k : vec![0xCF;5555],
            begin : PreciseTime::now(),
            stat : Rc::new(RefCell::new(Stat {
                conn : 0,
                sent : 0,
                got : 0,
                time_total : 0,
                time_max : 0,
            }))
        }
    }
    fn new_key(&self) -> Key {
        Key {
            roleid : 24678,
            timestamp : time::get_time().sec as u32,
            passcode : self.begin.to(PreciseTime::now()).num_milliseconds() as u32,
        }
    }
}

service_define!(CLIENT_SERVICE : ClientService);

impl ServiceHandler for ClientService {
    type Packet = ProtocolFrom7001;
    type Streamer = PwStreamer<Self::Packet>;
    fn connected(&self, token : Token) {
        trace!("client_service {:?} connected", token);
        let mut stat = self.stat.borrow_mut();
        stat.conn += 1;
        stat.sent += 1;
        let key = self.new_key();
        let req = ProtocolFrom7001::Set(key, self.packet5k.clone());
        service_write!(CLIENT_SERVICE, token, &req);
    }
    fn disconnected(&self, token : Token) {
        trace!("client_service {:?} disconnected", token);
    }
    fn incoming(&self, token : Token, packet : Self::Packet) {
        trace!("client_service {:?} incoming", token);
        match packet {
            ProtocolFrom7001::SetRe(key, ret) => {
                let now = self.begin.to(PreciseTime::now()).num_milliseconds() as u32;
                let mut stat = self.stat.borrow_mut();
                stat.got += 1;
                let t = now - key.passcode;
                stat.time_total += t;
                if stat.time_max < t {
                    stat.time_max = t;
                }
                if stat.got == 1 {
                    stat.print();
                    service_exit!(CLIENT_SERVICE);
                }

                stat.sent += 1;
                let nextkey = self.new_key();
                let nextreq = ProtocolFrom7001::Set(nextkey, self.packet5k.clone());
                service_write!(CLIENT_SERVICE, token, &nextreq);
            }
            ProtocolFrom7001::GetRe(roleid, key, ret, data) => {
            }
            _ => {
                service_shutdown!(CLIENT_SERVICE, token);
                service_exit!(CLIENT_SERVICE);
            }
        }
    }
    fn outgoing(&self, _token : Token, _packet : &Self::Packet) {
    }
}

fn main() {
    init();
    let client_service = ClientService::new();
    service_start!(CLIENT_SERVICE, client_service, ServiceConfig::from_file("config.toml", "client_service"));
    run_loop();
}

