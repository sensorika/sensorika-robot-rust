use zmq;
use serde_json;
use serde_json::Value;
use std::str;
use std::error::Error;
use std::result::*;

pub trait  SendRecvJson{
    fn recv_json(&mut self) -> Result<Value, Box<Error>>;
    fn send_json(&mut self, &Value) -> Result<(), Box<Error>>;
}

impl SendRecvJson for zmq::Socket {
    fn recv_json(&mut self) -> Result<Value, Box<Error>> {
        let req = try!(self.recv_bytes(0));
        let req_slice = req.as_slice();
        let str_req: &str =  try!(str::from_utf8(&req_slice));
        let res: Value = try!(serde_json::from_str(&str_req));
        Ok(res)
    }

    fn send_json(&mut self, req: &Value) -> Result<(), Box<Error>> {
        Ok(try!(self.send(try!(serde_json::to_string(&req)).as_ref(), 0)))
    }
}

#[cfg(test)]
mod tests{
    use super::SendRecvJson;
    use std;
    use std::thread::Builder;
    use std::sync::Arc;
    use serde_json::{Value, Map};
    use serde_json::builder::ObjectBuilder;
    use zmq;
    use zmq::SocketType::{REQ, REP};

    fn handle(r: std::thread::Result<()>){
        match r {
            Ok(r) => println!("All is well! {:?}", r),
            Err(e) => {
                if let Some(e) = e.downcast_ref::<&'static str>() {
                    panic!("Thread panicked: {}", e);
                } else {
                    panic!("Thread returned error: {:?}", e);
                }
            }
        }
    }

    #[test]
    fn test_req_rep(){
        let recieving = Builder::new().name("recv".into()).spawn(move||{
            let mut ctx = zmq::Context::new();
            let mut sock: zmq::Socket = ctx.socket(REP).unwrap();
            sock.bind("tcp://*:5555").unwrap();
            let req :Value = sock.recv_json().unwrap();
            assert!(req.is_object());
            let req : &Map<String, Value> = req.as_object().unwrap();
            assert!(req.contains_key("action")&& req.contains_key("count"));
            let act: &Value = req.get("action").unwrap();
            assert!(act.is_string());
            assert!(act.as_str().unwrap() == "set");
            let cnt: &Value = req.get("count").unwrap();
            assert!(cnt.is_number());
            assert!(cnt.as_i64().unwrap()==1);
            let o = ObjectBuilder::new().insert("status","ok").build();
            sock.send_json(&o).unwrap();
        }).unwrap();
        let sending = Builder::new().name("send".into()).spawn(move||{
            let mut ctx = zmq::Context::new();
            let mut sock: zmq::Socket = ctx.socket(REQ).unwrap();
            let o = ObjectBuilder::new()
                .insert("action", "set")
                .insert("count",1)
                .build();
            sock.connect("tcp://127.0.0.1:5555").unwrap();
            sock.send_json(&o).unwrap();
            let rep:Value = sock.recv_json().unwrap();
            assert!(rep.is_object());
            let rep : &Map<String, Value> = rep.as_object().unwrap();
            assert!(rep.contains_key("status"));
            let st: &Value = rep.get("status").unwrap();
            assert!(st.is_string());
            assert!(st.as_str().unwrap() == "ok");
        }).unwrap();
        sending.join().unwrap();
        recieving.join().unwrap();
    }
}