#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

#[macro_use]
extern crate log;
extern crate mio;
extern crate byteorder;
extern crate serde;
extern crate serde_json;
extern crate env_logger;
extern crate rustc_serialize;
extern crate toml;

#[macro_use]
pub mod service;

pub mod streamer;

