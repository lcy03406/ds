#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

#[macro_use]
extern crate ds;
#[macro_use]
extern crate log;
extern crate serde;

use std::io::Write;
use std::str::FromStr;
use std::num::ParseIntError;

use serde::{Serializer, Deserializer};

use ds::service::{Token, ServiceHandler, ServiceRef, ServiceConfig, init, run_loop};
use ds::streamer::memcached::MemcachedStreamer;
use ds::streamer::memcached;

struct DbService;

service_define!(DB_SERVICE : DbService);

impl ServiceHandler for DbService {
    type Packet = memcached::protocol::Packet;
    type Streamer = MemcachedStreamer;
    fn connected(&self, _token : Token) {
    }
    fn disconnected(&self, _token : Token) {
    }
    fn incoming(&self, _token : Token, packet : Self::Packet) {
        trace!("incoming {:?}", packet);
        match packet.header.opcode {
            memcached::protocol::PROTOCOL_BINARY_CMD_GET => {
                assert_eq!(packet.header.opaque, 1);
                assert_eq!(packet.header.status.0, 0);
                assert_eq!(packet.value, b"123a123");
                service_exit!(DB_SERVICE);
            }
            memcached::protocol::PROTOCOL_BINARY_CMD_SET => {
                assert_eq!(packet.header.opaque, 0);
                assert_eq!(packet.header.status.0, 0);
                service_broadcast!(DB_SERVICE, &memcached::protocol::Packet::new_request_get(1, "@@aaa.123".to_string()));
            }
            _ => {
            }
        }
    }
    fn outgoing(&self, _token : Token, _packet : &Self::Packet) {
    }
}

#[test]
fn service_memcached() {
    init();
    let db_service = DbService;
    service_start!(DB_SERVICE, db_service, ServiceConfig::client("db_service", "0.0.0.0:11211"));
    service_broadcast!(DB_SERVICE, &memcached::protocol::Packet::new_request_set(0, "@@aaa.123".to_string(), b"123a123".to_vec()));
    run_loop();
}

