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