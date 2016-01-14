use std::rc::{Rc};
use std::cell::{RefCell};
use std::io::{Read, Write};

use super::looper::*;
use super::stream::*;
use super::service::*;

#[test]
fn test_service() {
    
    use env_logger;
    env_logger::init().unwrap();

    let mut handler = LoopHandler;
    let conf = ServiceConfig {
        name : "test service".to_string(),
        listen : vec!["0.0.0.0:12306"].iter().map(|s| s.to_string()).collect(),
        connect : vec!["127.0.0.1:12306"].iter().map(|s| s.to_string()).collect(),
    };
    let s = Service::new(conf);
    s.start();
    trace!("run");
    handler.run();
    assert!(s.streams_count() == 0);
}
