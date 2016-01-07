use std::rc::{Rc};
use std::cell::{RefCell};
use std::io::{Read, Write};

use super::looper::*;
use super::stream::*;

#[test]
fn test_reg() {
    
    use env_logger;
    env_logger::init().unwrap();

    struct Conn {
        state : i32
    }
    
    impl<'a> Streamer<'a> for Conn {
        fn on_accept(&self, c : &mut Stream<'a>) {
        }
        fn on_connect(&self, c : &mut Stream<'a>) {
            c.write("hello server".as_bytes());
        }
        fn on_close(&self, c : &mut Stream<'a>) {
        }
        fn on_read(&self, c : &mut Stream<'a>) {
        }
    }
    let mut handler = LoopHandler::new();
    let conn = Rc::new(RefCell::new(Conn{state:0}));
    let weak = Rc::downgrade(&conn);
    Stream::connect(&handler.looper, weak, "127.0.0.1:12306").unwrap();
    trace!("run");
    handler.run();
}
