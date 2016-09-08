#![feature(custom_derive)]
#![feature(plugin)]
#![plugin(serde_macros)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]

extern crate serde_json;
extern crate zmq;
extern crate chrono;

pub mod connector;
mod util;

pub use self::connector::Connector;