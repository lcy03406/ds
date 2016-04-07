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

struct Ongoing {
    token : Token,
    roleid : u64,
    key : Key,
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
    fn connected(&self, token : Token) {
        trace!("front_service {:?} connected", token);
    }
    fn disconnected(&self, token : Token) {
        trace!("front_service {:?} disconnected", token);
    }
    fn incoming(&self, token : Token, packet : Self::Packet) {
        match packet {
            ProtocolFrom7001::Set(key, value) => {
                let opaque = match self.ongoing.borrow().keys().next_back() {
                    None => 0,
                    Some(n) => *n,
                };
                let keystr = key.to_string();
            	trace!("front_service {:?} receive request set {:?}", token, key);
                self.ongoing.borrow_mut().insert(opaque, Ongoing { token : token, roleid : 0, key : key });
                service_broadcast!(DB_SERVICE, &memcached::protocol::Packet::new_request_set(opaque, keystr, value));
            }
            ProtocolFrom7001::Get(roleid, key) => {
                let opaque = match self.ongoing.borrow().keys().next_back() {
                    None => 0,
                    Some(n) => *n,
                };
                let keystr = key.to_string();
            	trace!("front_service {:?} receive request get {:?}", token, key);
                self.ongoing.borrow_mut().insert(opaque, Ongoing { token : token, roleid : roleid, key : key });
                service_broadcast!(DB_SERVICE, &memcached::protocol::Packet::new_request_get(opaque, keystr));
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
    fn connected(&self, token : Token) {
        trace!("db_service {:?} connected to db", token);
    }
    fn disconnected(&self, token : Token) {
        trace!("db_service {:?} dosconnected to db", token);
    }
    fn incoming(&self, intoken : Token, packet : Self::Packet) {
        match packet.header.opcode {
            memcached::protocol::PROTOCOL_BINARY_CMD_GET => {
                let ongoing = match self.ongoing.borrow_mut().remove(&packet.header.opaque) {
                    None => return,
                    Some(g) => g
                };
                let token = ongoing.token;
                let key = ongoing.key;
                let result = packet.header.status.0 as i32;
            	trace!("db_service {:?} receive response to {:?} get {:?} result {:?}", intoken, token, key, result);
                let re = ProtocolFrom7001::GetRe(ongoing.roleid, key, result, packet.value);
                service_write!(FRONT_SERVICE, token, &re);
            }
            memcached::protocol::PROTOCOL_BINARY_CMD_SET => {
                let ongoing = match self.ongoing.borrow_mut().remove(&packet.header.opaque) {
                    None => return,
                    Some(g) => g
                };
                let token = ongoing.token;
                let key = ongoing.key;
                let result = packet.header.status.0 as i32;
            	trace!("db_service {:?} receive response to {:?} set {:?} result {:?}", intoken, token, key, result);
                let re = ProtocolFrom7001::SetRe(key, result);
                service_write!(FRONT_SERVICE, token, &re);
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
    service_start!(FRONT_SERVICE, front_service, ServiceConfig::from_file("config.toml", "front_service"));
    service_start!(DB_SERVICE, db_service, ServiceConfig::from_file("config.toml", "db_service"));
    run_loop();
}

