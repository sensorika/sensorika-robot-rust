
use std::str;
use std::error::Error;
use std::result::*;
use std::time::SystemTime;
use util::buffered_queue::BufferedQueue;

use zmq::SocketType;
use zmq::Socket;
use zmq::Context;

pub struct Worker {
    context: Context,
    sync_socket: Socket,
    data: BufferedQueue<String>,
    dt: f32,
}

impl Worker {
    pub fn new() -> Worker {
        let mut ctx = Context::new();
        let sync_socket = ctx.socket(SocketType::REP);

        unimplemented!();
    }

    pub fn add(v: String){

    }

    pub fn run(self){
        unimplemented!();
    }

    // возвращает N последних добавленных элементов
    // [1,2,3,4,5,6,7,8,9]
    //  ^ ^ ^
    // при N = 3
    pub fn get() -> Vec<String> {
        unimplemented!();
    }
}