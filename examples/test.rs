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
use sensorika::ConnectorAsync;
use sensorika::util::sendrecvjson::SendRecvJson;
use sensorika::util::time::now;
use std::thread::Builder;
use std::panic::catch_unwind;
use std::sync::mpsc::{channel, Sender, Receiver};
use serde_json::{Value, Map};
use serde_json::builder::ObjectBuilder;
use zmq::SocketType::{PUB};
use std::thread::sleep;
use std::time::Duration;


fn main(){
    let mut c = zmq::Context::new();
    let mut s: zmq::Socket = c.socket(PUB).unwrap();
    let (mut send, mut recv)= channel::<(bool,i32)>();
    s.bind("tcp://127.0.0.1:5655").unwrap();
    let mut i = 0;
    let ca1 = ConnectorAsync::new("127.0.0.1".into(), 5655, move |v: &mut Value| {
        let catcher = catch_unwind(|| {
            assert!(v.is_array());
            let d: &Vec<Value> = v.as_array().unwrap();
            assert!(d.len() == 2);
            let d1: &Vec<Value> = d[0].as_array().unwrap();
            assert!(d1.len() == 2);
        });
        i += 1;
        println!("Sending {:?}", catcher.is_ok());
        send.send((catcher.is_ok(), i)).unwrap();
    });

    let mut val = Vec::<Value>::new();
    val.push(Value::F64(now()));
    val.push(Value::Null);
    println!("Sending!");
    s.send_json(&Value::Array(val), 0).unwrap();
    let mut val = Vec::<Value>::new();
    val.push(Value::F64(now()));
    val.push(Value::Null);
    println!("Sending!");
    s.send_json(&Value::Array(val), 0).unwrap();
    println!("Waiting result!");
    loop{
        match recv.recv() {
            Ok(v) => {
                println!("Recvd {:?}", v);
                assert!(v.0);
                if v.1==1{
                    break;
                }
            },
            Err(e) => {
                //println!("{:?}", e);
                //panic!("{Connector exited too early!}");
                break;
            }
        }
    }
}
