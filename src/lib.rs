
//FIXME: delete after prototyping
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(unreachable_code)]
#![feature(plugin, custom_derive)]
#![plugin(serde_macros)]

extern crate serde_json;
extern crate zmq;

pub mod worker;
pub mod util;

