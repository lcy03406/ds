#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

#[macro_use]
extern crate ds;
#[macro_use]
extern crate log;
extern crate serde;
extern crate time;
extern crate toml;
extern crate rustc_serialize;

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::str::FromStr;
use std::num::ParseIntError;
use std::fs::File;
use serde::{Serializer, Deserializer};

use ds::service::{Token, ServiceHandler, ServiceRef, ServiceConfig, init, run_loop};
use ds::streamer::pw::PwStreamer;
use ds::streamer::memcached::MemcachedStreamer;
use ds::streamer::memcached;

use time::PreciseTime;

#[derive(RustcEncodable, RustcDecodable, Debug)]
struct TableConfig {
    prefix : String,
    count : u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Key {
    roleid : u64,
    timestamp : u32,
    passcode : u32,
}

impl Key {
    fn to_string(&self, table: &TableConfig) -> String {
        let part = self.timestamp/86400%table.count;
        format!("@@{}{}.{:016X}|{:08X}|{:08X}", table.prefix, part, self.roleid, self.timestamp, self.passcode)
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
    time : PreciseTime,
}

struct FrontService {
    ongoing : Rc<RefCell<BTreeMap<u32, Ongoing>>>,
    table : TableConfig,
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
                let mut ongoing = self.ongoing.borrow_mut();
                let opaque = match ongoing.keys().next_back() {
                    None => 0,
                    Some(n) => *n + 1,
                };
                let keystr = key.to_string(&self.table);
            	trace!("front_service {:?} {:?} receive request set {:?} {}", token, opaque, key, keystr);
                ongoing.insert(opaque, Ongoing { token : token, roleid : 0, key : key, time : PreciseTime::now() });
                service_broadcast!(DB_SERVICE, &memcached::protocol::Packet::new_request_set(opaque, keystr, value));
            }
            ProtocolFrom7001::Get(roleid, key) => {
                let mut ongoing = self.ongoing.borrow_mut();
                let opaque = match ongoing.keys().next_back() {
                    None => 0,
                    Some(n) => *n + 1,
                };
                let keystr = key.to_string(&self.table);
            	trace!("front_service {:?} {:?} receive request get {:?} {}", token, opaque, key, keystr);
                ongoing.insert(opaque, Ongoing { token : token, roleid : roleid, key : key, time : PreciseTime::now()});
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
        let now = PreciseTime::now();
        match packet.header.opcode {
            memcached::protocol::PROTOCOL_BINARY_CMD_GET => {
                let opaque = packet.header.opaque;
                let result = packet.header.status.0 as i32;
                let ongoing = match self.ongoing.borrow_mut().remove(&packet.header.opaque) {
                    None => {
            	        trace!("db_service {:?} {:?} receive response to unknown get unknown result {:?}", intoken, opaque, result);
                        return
                    }
                    Some(g) => g
                };
                let token = ongoing.token;
                let key = ongoing.key;
            	trace!("db_service {:?} {:?} receive response to {:?} get {:?} result {:?} time {:?}", intoken, opaque, token, key, result, ongoing.time.to(now).num_milliseconds());
                let re = ProtocolFrom7001::GetRe(ongoing.roleid, key, result, packet.value);
                service_write!(FRONT_SERVICE, token, &re);
            }
            memcached::protocol::PROTOCOL_BINARY_CMD_SET => {
                let opaque = packet.header.opaque;
                let result = packet.header.status.0 as i32;
                let ongoing = match self.ongoing.borrow_mut().remove(&packet.header.opaque) {
                    None => {
            	        trace!("db_service {:?} {:?} receive response to unknown set unknown result {:?}", intoken, opaque, result);
                        return
                    },
                    Some(g) => g
                };
                let token = ongoing.token;
                let key = ongoing.key;
            	trace!("db_service {:?} {:?} receive response to {:?} set {:?} result {:?} time {:?}", intoken, opaque, token, key, result, ongoing.time.to(now).num_milliseconds());
                let re = ProtocolFrom7001::SetRe(key, result);
                service_write!(FRONT_SERVICE, token, &re);
            }
            _ => {
            	trace!("db_service {:?} receive unknown response", intoken);
            }
        }
    }
    fn outgoing(&self, _token : Token, _packet : &Self::Packet) {
    }
}

fn main() {
    let mut file = File::open("config.toml").unwrap();
    let mut st = String::new();
    file.read_to_string(&mut st).unwrap();
    let mut map = toml::Parser::new(&st).parse().unwrap();
    let value = map.remove("table").unwrap();
    let table = toml::decode(value).unwrap();
    init();
    let ongoing = Rc::new(RefCell::new(BTreeMap::new()));
    let front_service = FrontService { ongoing : ongoing.clone(), table : table };
    let db_service = DbService { ongoing : ongoing.clone() };
    let front_config = map.remove("front_service").unwrap();
    let db_config = map.remove("db_service").unwrap();
    service_start!(FRONT_SERVICE, front_service, ServiceConfig::from_toml(front_config));
    service_start!(DB_SERVICE, db_service, ServiceConfig::from_toml(db_config));
    run_loop();
}

