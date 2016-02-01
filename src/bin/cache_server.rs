#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

#[macro_use]
extern crate ds;
#[macro_use]
extern crate log;
extern crate serde;

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

#[derive(Serialize, Deserialize, Debug)]
struct Key {
    roleid : u64,
    timestamp : u32,
    passcode : u32,
}

impl Key {
    fn to_string(&self) -> String {
        format!("@@cache.{:016X}/{:08X}/{:08X}", self.roleid, self.timestamp, self.passcode)
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

struct Ongoing {
    token : Token,
    roleid : u64,
}

struct FrontService {
    ongoing : Rc<RefCell<BTreeMap<u32, Ongoing>>>,
}
struct DbService {
    ongoing : Rc<RefCell<BTreeMap<u32, Ongoing>>>,
}

service_define!(FRONT_SERVICE : FrontService);
service_define!(DB_SERVICE : DbService);

impl ServiceHandler for FrontService {
    type Packet = ProtocolFrom7001;
    type Streamer = PwStreamer<Self::Packet>;
    fn connected(&self, _token : Token) {
    }
    fn disconnected(&self, _token : Token) {
    }
    fn incoming(&self, token : Token, packet : Self::Packet) {
        match packet {
            ProtocolFrom7001::Set(key, value) => {
                let opaque = match self.ongoing.borrow().keys().next_back() {
                    None => 0,
                    Some(n) => *n,
                };
                self.ongoing.borrow_mut().insert(opaque, Ongoing { token : token, roleid : 0 });
                service_broadcast!(DB_SERVICE, &memcached::protocol::Packet::new_request_set(opaque, key.to_string(), value));
            }
            ProtocolFrom7001::Get(roleid, key) => {
                let opaque = match self.ongoing.borrow().keys().next_back() {
                    None => 0,
                    Some(n) => *n,
                };
                self.ongoing.borrow_mut().insert(opaque, Ongoing { token : token, roleid : roleid });
                service_broadcast!(DB_SERVICE, &memcached::protocol::Packet::new_request_get(opaque, key.to_string()));
            }
            _ => {
                service_shutdown!(FRONT_SERVICE, token);
            }
        }
    }
    fn outgoing(&self, _token : Token, _packet : &Self::Packet) {
    }
}

impl ServiceHandler for DbService {
    type Packet = memcached::protocol::Packet;
    type Streamer = MemcachedStreamer;
    fn connected(&self, _token : Token) {
    }
    fn disconnected(&self, _token : Token) {
        self.ongoing.borrow_mut().clear();
    }
    fn incoming(&self, _token : Token, packet : Self::Packet) {
        match packet.header.opcode {
            memcached::protocol::PROTOCOL_BINARY_CMD_GET => {
                let ongoing = match self.ongoing.borrow_mut().remove(&packet.header.opaque) {
                    None => return,
                    Some(g) => g
                };
                let key = Key::from_str(&packet.key).unwrap();
                let result = packet.header.status.0 as i32;
                let packet = ProtocolFrom7001::GetRe(ongoing.roleid, key, result, packet.value);
                service_write!(FRONT_SERVICE, ongoing.token, &packet);
            }
            memcached::protocol::PROTOCOL_BINARY_CMD_SET => {
                let ongoing = match self.ongoing.borrow_mut().remove(&packet.header.opaque) {
                    None => return,
                    Some(g) => g
                };
                let key = Key::from_str(&packet.key).unwrap();
                let result = packet.header.status.0 as i32;
                let packet = ProtocolFrom7001::SetRe(key, result);
                service_write!(FRONT_SERVICE, ongoing.token, &packet);
            }
            _ => {
            }
        }
    }
    fn outgoing(&self, _token : Token, _packet : &Self::Packet) {
    }
}

fn main() {
    init();
    let ongoing = Rc::new(RefCell::new(BTreeMap::new()));
    let front_service = FrontService { ongoing : ongoing.clone() };
    let db_service = DbService { ongoing : ongoing.clone() };
    service_start!(FRONT_SERVICE, front_service, ServiceConfig::server("front_service", "0.0.0.0:44944"));
    service_start!(DB_SERVICE, db_service, ServiceConfig::client("db_service", "0.0.0.0:11211"));
    run_loop();
}

