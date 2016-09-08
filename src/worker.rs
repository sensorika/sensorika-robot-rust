
use std::str;
use std::error::Error;
use std::result::*;
use std::time::SystemTime;
use std::collections::LinkedList;

use zmq::SocketType;
use zmq::Socket;
use zmq::Context;

pub struct Worker {
    context: Context,
    sync_socket: Socket,
    //Массив, который при переполнении удаляет первые элементы
    data: Vec<String>, //FIXME:
    dt: f32,
}

impl Worker {
    pub fn new() -> Worker {
        let mut ctx = Context::new();
        let sync_socket = ctx.socket(SocketType::REP);

        panic!(" ");
    }

    pub fn run(self){

    }

    // возвращает N последних добавленных элементов
    // [1,2,3,4,5,6,7,8,9]
    //  ^ ^ ^
    // при N = 3
    pub fn get() -> Vec<String> {
        panic!(" ");
    }
}