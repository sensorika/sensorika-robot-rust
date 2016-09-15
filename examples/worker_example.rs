
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


fn main() {
    let mut w: Worker<i32> = Worker::<i32>::new("test", "127.0.0.1", 15701).unwrap();
    for i in 0..10 {
        w.add(i);
    }
    println!("{:?}", w.get(3));

    let mut c = zmq::Context::new();
    let mut s: zmq::Socket = c.socket(SocketType::REQ).unwrap();
    s.connect("tcp://127.0.0.1:15701").unwrap();

    let mut v = serde_json::to_value(&Msg::get(3));
    println!("value client REQ: {:?}", &v);
    s.send_json(&v, 0).unwrap();
    thread::sleep_ms(1000);
    let new_v = s.recv_json(0).unwrap();
    println!("value client RES: {:?}", &new_v);

}