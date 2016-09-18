
extern crate sensorika;
extern crate serde_json;
extern crate zmq;

use sensorika::ConnectorAsync;
use sensorika::worker::Worker;
use sensorika::Connector;
use serde_json::builder::ObjectBuilder;
use sensorika::message::Msg;
use std::thread;
use sensorika::util::sendrecvjson::SendRecvJson;
use zmq::SocketType;
use serde_json::Value;
use std::fmt::Debug;

static IP: &'static str = "127.0.0.1";
const PORT: u32 = 15701;

fn main() {
    let port = PORT + 2;
    let mut w: Worker<i64> = Worker::<i64>::new("test", IP, port).unwrap();
    let mut c = zmq::Context::new();
    let mut s: zmq::Socket = c.socket(SocketType::REQ).unwrap();
    s.connect(format!("tcp://{}:{}", IP, PORT).as_str()).unwrap();

    let v = serde_json::to_value(&Msg::set(Value::I64(99)));
    println!("value client REQ: {:?}", &v);
    s.send_json(&v, 0).unwrap();
    thread::sleep_ms(200);


    let v: Value = s.recv_json(0).unwrap();

    let status = v.pointer("/status").unwrap().as_str();

    assert_eq!(status, Some("ok"));
    let data0 = w.get(10)[0].data;
    assert_eq!(data0, 99);
}

fn d<T: Debug>(any: T){
    println!("d: {:?}", any);
}