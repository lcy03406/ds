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

struct FrontService {
}

service_define!(FRONT_SERVICE : FrontService);

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
            	trace!("front_service {:?} receive request set {:?}", token, key);
                let re = ProtocolFrom7001::SetRe(key, 0);
                service_write!(FRONT_SERVICE, token, &re);
            }
            ProtocolFrom7001::Get(roleid, key) => {
            	trace!("front_service {:?} receive request get {:?}", token, key);
                let re = ProtocolFrom7001::GetRe(roleid, key, 0, vec![57;5555]);
                service_write!(FRONT_SERVICE, token, &re);
            }
            _ => {
                service_shutdown!(FRONT_SERVICE, token);
            }
        }
    }
    fn outgoing(&self, _token : Token, _packet : &Self::Packet) {
    }
}

fn main() {
    init();
    let front_service = FrontService {};
    service_start!(FRONT_SERVICE, front_service, ServiceConfig::from_file("config.toml", "front_service"));
    run_loop();
}

