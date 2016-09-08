
use std::str;
use std::error::Error;
use std::result::*;
use std::time::SystemTime;
use util::buffered_queue::BufferedQueue;
use util::sendrecvjson::SendRecvJson;
use serde_json::{Value, Map};
use serde_json;
use std::str::FromStr;

use zmq::SocketType;
use zmq::Socket;
use zmq::Context;
use zmq::DONTWAIT;

pub struct Worker {
    context: Context,
    sync_socket: Socket,
    data: BufferedQueue<String>,
    dt: f32,
}

impl Worker {
    pub fn new(ip: &str, port: u32) -> Result<Worker, Box<Error>> {
        let mut ctx = Context::new();
        let mut sync_socket: Socket = try!(ctx.socket(SocketType::REP));

        //FIXME:
        unimplemented!();
    }

    /// Добавляет новое значение
    pub fn add(&mut self, v: &Value){
        if let Ok(str) = serde_json::to_string(&v){
            self.data.push(str);
        }
    }

    /// Возвращает N последних присланных сообщений
    pub fn get(&self, n: u32) -> Vec<Value> {
        let mut result: Vec<Value> = Vec::new();
        let json_strings: Vec<String> = self.data.take(n);
        for str in json_strings {
            if let Ok(v) = Value::from_str(str.as_ref()){
                result.push(v);
            }
        }
        result
    }

    pub fn populate(&self){
        //FIXME:
        unimplemented!();
    }

    pub fn run(&self){
        //FIXME:
        unimplemented!();
    }
}

