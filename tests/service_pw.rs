#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

#[macro_use]
extern crate ds;
#[macro_use]
extern crate log;
extern crate serde;

use std::cell::RefCell;
use std::io::Write;
use serde::{Serializer, Deserializer};

use ds::service::{Token, ServiceHandler, ServiceRef, ServiceConfig, init, run_loop};
use ds::streamer::pw::PwStreamer;

#[derive(Serialize, Deserialize, Debug)]
struct Packet {
    x : i32,
    y : i32,
    zzz : Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
enum ProtocolFrom1 {
    Proto1(Packet),
}


struct Stat {
    conn : i32,
    disc : i32,
    send : i32,
    recv : i32,
}

impl Stat {
    fn new() -> Self {
        Stat {
            conn : 0,
            disc : 0,
            send : 0,
            recv : 0,
        }
    }
}

impl Drop for Stat {
    fn drop(&mut self) {
        assert_eq!(self.conn, 2);
        assert_eq!(self.disc, 2);
        assert_eq!(self.send, 10);
        assert_eq!(self.recv, 10);
    }
}

struct TestService {
    stat : RefCell<Stat>,
}
service_define!(TEST_SERVICE : TestService);

impl TestService {
    fn new() -> Self {
        TestService {
            stat : RefCell::new(Stat::new())
        }
    }
}

impl ServiceHandler for TestService {
    type Packet = ProtocolFrom1;
    type Streamer = PwStreamer<Self::Packet>;
    fn connected(&self, token : Token) {
        self.stat.borrow_mut().conn += 1;
        if self.stat.borrow().send == 0 {
            service_write!(TEST_SERVICE, token, &ProtocolFrom1::Proto1(Packet{x:1,y:1,zzz:vec![0x21;256]}));
        }
    }
    fn disconnected(&self, token : Token) {
        self.stat.borrow_mut().disc += 1;
        service_exit!(TEST_SERVICE);
    }
    fn incoming(&self, token : Token, packett : Self::Packet) {
        self.stat.borrow_mut().recv += 1;
        let ProtocolFrom1::Proto1(packet) = packett;
        assert!(packet.x == 1);
        if packet.y < 10 {
            service_write!(TEST_SERVICE, token, &ProtocolFrom1::Proto1(Packet{x:1,y:packet.y+1,zzz:vec![0x22;256]}));
        } else {
            service_shutdown!(TEST_SERVICE, token);
        }
    }
    fn outgoing(&self, token : Token, packet : &Self::Packet) {
        self.stat.borrow_mut().send += 1;
    }
}

#[test]
fn service_pw() {
    init();
    let conf = ServiceConfig {
        name : "service_pw".to_string(),
        listen : vec!["0.0.0.0:44944"].iter().map(|s| s.to_string()).collect(),
        connect : vec!["127.0.0.1:44944"].iter().map(|s| s.to_string()).collect(),
    };
    service_start!(TEST_SERVICE, TestService::new(), conf);
    trace!("loop begin");
    run_loop();
    trace!("loop exit");
}
