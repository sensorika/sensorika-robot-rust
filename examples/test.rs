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
    let mut ctx = zmq::Context::new();
    let mut sock_req: zmq::Socket = ctx.socket(REQ).unwrap();
    let mut sock_rep: zmq::Socket = ctx.socket(REP).unwrap();
    sock_rep.bind("tcp://127.0.0.1:5557").unwrap();
    sock_req.connect("tcp://127.0.0.1:5557").unwrap();
    sock_req.send_json(&Value::F64(0.0), zmq::DONTWAIT).unwrap();
    match sock_req.recv_json(zmq::DONTWAIT){
        Ok(val) => assert!(false),
        Err(e) => {
            println!("{:?}", e);
        }
    }
    sock_rep.recv_json(0).unwrap();
    sock_rep.send_json(&Value::Bool(true), 0).unwrap();
    sock_req.recv_json(0).unwrap();
}
