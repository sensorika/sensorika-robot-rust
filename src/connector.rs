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
    addr: String,
    t0: f64,
    data_ready: bool,
    cache: Value,
    socket: Socket,
    context: Context
}

//noinspection RustSelfConvention
impl Connector {
    pub fn new(ip: String, port: i32) -> Result<Connector, Box<Error>> {
        let mut context = Context::new();
        let mut socket: Socket = try!(context.socket(SocketType::REQ));
        let addr: String = format!("tcp://{}:{}", ip, port);
        try!(socket.connect(&addr));
        Ok(Connector{
            addr: addr,
            t0:now(), data_ready:false,
            cache: Value::F64(0.0),
            socket:socket,
            context:context
        })
    }

    pub fn to_locator(ip: String) -> Result<Connector, Box<Error>> {
        Connector::new(ip, 15701)
    }


    pub fn get_count_dt(&mut self, count: i64, dt: f64) -> Result<&Value, Box<Error>> {
        if (self.t0 + dt > now()) && self.data_ready {
            return Ok(&self.cache)
        }else{
            let req = ObjectBuilder::new()
                .insert("action", "get")
                .insert("count", count)
                .build();
            try!(self.socket.send_json(&req, 0));
            self.cache = try!(self.socket.recv_json(0));
            self.t0 = now();
            self.data_ready = true;
            return Ok(&self.cache)
        }
    }

    pub fn get_dt(&mut self, dt: f64) -> Result<&Value, Box<Error>> {
        self.get_count_dt(1, dt)
    }

    pub fn get_count(&mut self, count: i64) -> Result<&Value, Box<Error>> {
        self.get_count_dt(count, 0.05)
    }

    pub fn get(&mut self) -> Result<&Value, Box<Error>> {
        self.get_count_dt(1, 0.05)
    }

    pub fn set(&mut self, data: Value) -> Result<Value, Box<Error>> {
        let to_send: Value = ObjectBuilder::new()
            .insert("action","set").insert("count",1)
            .insert("data", data).build();
        try!(self.socket.send_json(&to_send, 0));
        let rep: Value = try!(self.socket.recv_json(0));
        Ok(rep)
    }
}

impl Drop for Connector{
    fn drop(&mut self) {
        self.socket.set_linger(0).unwrap();
    }
}

#[cfg(test)]
mod tests{
    use super::Connector;
    use util::sendrecvjson::SendRecvJson;
    use std::thread::Builder;
    use serde_json::{Value, Map};
    use serde_json::builder::ObjectBuilder;
    use zmq;
    use zmq::SocketType::{REQ, REP};
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_get(){
        let serv = Builder::new().name("serv".into()).spawn(move ||{
            let mut c = zmq::Context::new();
            let mut s: zmq::Socket = c.socket(REP).unwrap();
            s.bind("tcp://*:5555").unwrap();
            for i in 1..5 {
                let req: Value = s.recv_json(0).unwrap();
                assert!(req.is_object());
                let req: &Map<String, Value> = req.as_object().unwrap();
                assert!(req.contains_key("action")&& req.contains_key("count"));
                let act: &Value = req.get("action").unwrap();
                assert!(act.is_string());
                assert!(act.as_str().unwrap() == "get");
                let cnt: &Value = req.get("count").unwrap();
                assert!(cnt.is_number());
                assert!(cnt.as_i64().unwrap()==i);
                let o = ObjectBuilder::new().insert("status", "ok")
                    .insert("data",i).build();
                s.send_json(&o, 0).unwrap();
            }
        }).unwrap();
        let mut c: Connector = Connector::new("127.0.0.1".into(),5555).unwrap();
        {
            let rep: &Value = c.get().unwrap();
            assert!(rep.is_object());
            let rep: &Map<String, Value> = rep.as_object().unwrap();
            assert!(rep.contains_key("status"));
            let st: &Value = rep.get("status").unwrap();
            assert!(st.is_string());
            assert!(st.as_str().unwrap() == "ok");
        }
        for i in 2..5{
            sleep(Duration::new(1,0));
            let rep:&Value =c.get_count(i).unwrap();
            assert!(rep.is_object());
            let rep : &Map<String, Value> = rep.as_object().unwrap();
            assert!(rep.contains_key("status")&&rep.contains_key("data"));
            let st: &Value = rep.get("status").unwrap();
            assert!(st.is_string());
            assert!(st.as_str().unwrap() == "ok");
            let cnt: &Value = rep.get("data").unwrap();
            assert!(cnt.is_number());
            assert!(cnt.as_i64().unwrap()==i);
        }
        for i in 1..5 {
            let rep: &Value = c.get_count_dt(i, 100.0).unwrap();
            assert!(rep.is_object());
            let rep: &Map<String, Value> = rep.as_object().unwrap();
            assert!(rep.contains_key("status")&&rep.contains_key("data"));
            let st: &Value = rep.get("status").unwrap();
            assert!(st.is_string());
            assert!(st.as_str().unwrap() == "ok");
            let cnt: &Value = rep.get("data").unwrap();
            assert!(cnt.is_number());
            assert!(cnt.as_i64().unwrap()==4);
        }
        for i in 1..5 {
            let rep: &Value = c.get_dt(100.0).unwrap();
            assert!(rep.is_object());
            let rep: &Map<String, Value> = rep.as_object().unwrap();
            assert!(rep.contains_key("status")&&rep.contains_key("data"));
            let st: &Value = rep.get("status").unwrap();
            assert!(st.is_string());
            assert!(st.as_str().unwrap() == "ok");
            let cnt: &Value = rep.get("data").unwrap();
            assert!(cnt.is_number());
            assert!(cnt.as_i64().unwrap()==4);
        }
        serv.join().unwrap();
    }

    #[test]
    fn test_set_to_locator(){
        let serv = Builder::new().name("serv".into()).spawn(move || {
            let mut c = zmq::Context::new();
            let mut s: zmq::Socket = c.socket(REP).unwrap();
            s.bind("tcp://*:15701").unwrap();
            let req: Value = s.recv_json(0).unwrap();
            assert!(req.is_object());
            let req: &Map<String, Value> = req.as_object().unwrap();
            assert!(req.contains_key("action")&& req.contains_key("count"));
            let act: &Value = req.get("action").unwrap();
            assert!(act.is_string());
            assert!(act.as_str().unwrap() == "set");
            let cnt: &Value = req.get("count").unwrap();
            assert!(cnt.is_number());
            assert!(cnt.as_i64().unwrap()==1);
            let cnt: &Value = req.get("data").unwrap();
            assert!(cnt.is_number());
            assert!(cnt.as_i64().unwrap()==100500);
            let o = ObjectBuilder::new().insert("status", "ok").build();
            s.send_json(&o,0).unwrap();

        }).unwrap();
        let conn = Builder::new().name("conn".into()).spawn(move || {
            let mut c = Connector::to_locator("127.0.0.1".into()).unwrap();
            let rep: Value = c.set(Value::I64(100500)).unwrap();
            assert!(rep.is_object());
            let rep: &Map<String, Value> = rep.as_object().unwrap();
            assert!(rep.contains_key("status"));
            let st: &Value = rep.get("status").unwrap();
            assert!(st.is_string());
            assert!(st.as_str().unwrap() == "ok");
        }).unwrap();
        serv.join().unwrap();
        conn.join().unwrap();
    }
}
