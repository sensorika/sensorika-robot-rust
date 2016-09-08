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
extern crate chrono;
extern crate sensorika;
use sensorika::Connector;
use sensorika::util::sendrecvjson::SendRecvJson;
use std::thread::Builder;
use serde_json::{Value, Map};
use serde_json::builder::ObjectBuilder;
use zmq::SocketType::{REQ, REP};
use std::thread::sleep;
use std::time::Duration;

fn main(){

    println!("Ok");
}
