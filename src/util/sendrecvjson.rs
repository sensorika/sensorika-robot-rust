use zmq;
use serde_json;
use serde_json::Value;
use std::str;
use std::error::Error;
use std::result::*;

#[derive(Copy, Clone)]
enum SendRecvMode{
    Block = 0,
    NonBlock = 1,
}

pub trait  SendRecvJson{
    fn recv_json(&mut self, i32) -> Result<Value, Box<Error>>;
    fn send_json(&mut self, &Value, i32) -> Result<(), Box<Error>>;
}

impl SendRecvJson for zmq::Socket {
    fn recv_json(&mut self, flags:i32) -> Result<Value, Box<Error>> {
        let req = try!(self.recv_bytes(flags));
        let req_slice = req.as_slice();
        let str_req: &str =  try!(str::from_utf8(&req_slice));
        let res: Value = try!(serde_json::from_str(&str_req));
        Ok(res)
    }

    fn send_json(&mut self, req: &Value, flags: i32) -> Result<(), Box<Error>> {
        Ok(try!(self.send(try!(serde_json::to_string(&req)).as_ref(), flags)))
    }

}

#[cfg(test)]
mod tests{
    use super::SendRecvJson;
    use std::thread::Builder;
    use serde_json::{Value, Map};
    use serde_json::builder::ObjectBuilder;
    use zmq;
    use zmq::SocketType::{REQ, REP};

    #[test]
    fn test_req_rep(){
        let recieving = Builder::new().name("recv".into()).spawn(move||{
            let mut ctx = zmq::Context::new();
            let mut sock: zmq::Socket = ctx.socket(REP).unwrap();
            sock.bind("tcp://*:5556").unwrap();
            for i in 1..10 {
                let req: Value = sock.recv_json(0).unwrap();
                assert!(req.is_object());
                let req: &Map<String, Value> = req.as_object().unwrap();
                assert!(req.contains_key("action")&& req.contains_key("count"));
                let act: &Value = req.get("action").unwrap();
                assert!(act.is_string());
                assert!(act.as_str().unwrap() == "set");
                let cnt: &Value = req.get("count").unwrap();
                assert!(cnt.is_number());
                assert!(cnt.as_i64().unwrap()==1);
                let o = ObjectBuilder::new().insert("status", "ok").build();
                sock.send_json(&o, 0).unwrap();
            }
        }).unwrap();
        let sending = Builder::new().name("send".into()).spawn(move||{
            let mut ctx = zmq::Context::new();
            let mut sock: zmq::Socket = ctx.socket(REQ).unwrap();
            sock.connect("tcp://127.0.0.1:5556").unwrap();
            for i in 1..10 {
                let o = ObjectBuilder::new()
                    .insert("action", "set")
                    .insert("count", 1)
                    .build();
                sock.send_json(&o,0).unwrap();
                let rep: Value = sock.recv_json(0).unwrap();
                assert!(rep.is_object());
                let rep: &Map<String, Value> = rep.as_object().unwrap();
                assert!(rep.contains_key("status"));
                let st: &Value = rep.get("status").unwrap();
                assert!(st.is_string());
                assert!(st.as_str().unwrap() == "ok");
            }
        }).unwrap();
        sending.join().unwrap();
        recieving.join().unwrap();
    }

    #[test]
    fn test_async(){
        let mut ctx = zmq::Context::new();
        let mut sock_req: zmq::Socket = ctx.socket(REQ).unwrap();
        let mut sock_rep: zmq::Socket = ctx.socket(REP).unwrap();
        sock_rep.bind("tcp://127.0.0.1:5557").unwrap();
        sock_req.connect("tcp://127.0.0.1:5557").unwrap();
        sock_req.send_json(&Value::F64(0.0), zmq::DONTWAIT).unwrap();
        match sock_req.recv_json(zmq::DONTWAIT){
            Ok(val) => assert!(false),
            Err(e) => {

            }
        }
        sock_rep.recv_json(0).unwrap();
        sock_rep.send_json(&Value::Bool(true), 0).unwrap();
        sock_req.recv_json(0).unwrap();
    }
}