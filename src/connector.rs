use std::error::Error;
use std::result::*;
use zmq::{Context, Socket, SocketType};
use serde_json::{Value, Map};
use serde_json::to_string;
use serde_json::builder::ObjectBuilder;
use std::fmt::format;
use util::time::now;
use util::sendrecvjson::SendRecvJson;

pub struct Connector {
    context: Context,
    socket: Socket,
    addr: String,
    t0: f64,
    data_ready: bool,
    cache: Value
}

impl Connector {
    fn new(ip: String, port: i32) -> Result<Connector, Box<Error>> {
        let mut context = Context::new();
        let mut socket: Socket = try!(context.socket(SocketType::REQ));
        let addr: String = format!("tcp://{}:{}", ip, port);
        try!(socket.connect(&addr));
        Ok(Connector{
            context:context,
            socket:socket,
            addr: addr,
            t0:now(), data_ready:false,
            cache: Value::F64(0.0)})
    }

    fn from_default_port(ip: String) -> Result<Connector, Box<Error>> {
        Connector::new(ip, 15701)
    }

    fn get_with_dt(&mut self, dt: f64) -> Result<&Value, Box<Error>> {
        if (self.t0 + dt > now()) && self.data_ready {
            return Ok(&self.cache)
        }else{
            let req = ObjectBuilder::new()
                .insert("action", "get")
                .insert("count", 2)
                .build();
            try!(self.socket.send_json(&req));
            self.cache = try!(self.socket.recv_json());
            self.t0 = now();
            self.data_ready = true;
            return Ok(&self.cache)
        }
    }

    fn get(&mut self) -> Result<&Value, Box<Error>> {
        self.get_with_dt(0.05)
    }

    fn set(&self, data: Value) -> Result<Value, Box<Error>> {
        Ok(Value::I64(0))
    }
}