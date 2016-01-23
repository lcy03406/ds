#![cfg(test)]

use std::io::{Write, BufRead};

use super::*;

struct TestService;
service_define!(TEST_SERVICE : TestService);

impl ServiceStreamer for TestService {
    type Packet = u8;
    type Error =();
    fn write_packet(packet : &u8, writer : &mut BufWrite) ->Result<(), Self::Error> {
        writer.write(&vec![*packet][..]).unwrap();
        Ok(())
    }
    fn read_packet(reader : &mut BufRead) -> Result<Option<u8>, Self::Error> {
        match match reader.fill_buf() {
            Err(e) => {
                Ok(None)
            }
            Ok(buf) if buf.len() == 0 => {
                Ok(None)
            }
            Ok(buf) => {
                Ok(Some(buf[0]))
            }
        } {
            Ok(Some(a)) => {
                reader.consume(1);
                Ok(Some(a))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e)
        }
    }
}
impl ServiceHandler for TestService {
    type Packet = u8;
    type Streamer = TestService;
    fn connected(&self, token : Token) {
        service_write!(TEST_SERVICE, token, &1u8);
    }
    fn disconnected(&self, token : Token) {
        service_exit!(TEST_SERVICE);
    }
    fn incoming(&self, token : Token, packet : Self::Packet) {
        if packet == 1u8 {
            service_shutdown!(TEST_SERVICE, token);
        }
    }
    fn outgoing(&self, token : Token, packet : &Self::Packet) {
    }
}

#[test]
fn service_test() {
    init();
    let conf = ServiceConfig {
        name : "test service".to_string(),
        listen : vec!["0.0.0.0:12306"].iter().map(|s| s.to_string()).collect(),
        connect : vec!["127.0.0.1:12306"].iter().map(|s| s.to_string()).collect(),
    };
    service_start!(TEST_SERVICE, TestService, conf);
    trace!("loop begin");
    run_loop();
    trace!("loop exit");
}
