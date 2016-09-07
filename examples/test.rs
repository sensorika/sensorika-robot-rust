
extern crate zmq;

fn main() {
    println!("ALL OK");
    let mut ctx = zmq::Context::new();

    let mut socket = match ctx.socket(zmq::REQ) {
        Ok(socket) => { socket },
        Err(e) => { panic!(e) }
    };

    match socket.connect("tcp://127.0.0.1:1234") {
        Ok(()) => (),
        Err(e) => panic!(e)
    }

    match socket.send_str("hello world!", 0) {
        Ok(()) => (),
        Err(e) => panic!(e)
    }
}