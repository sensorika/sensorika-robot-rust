use std::error::Error;
use std::result::*;
use std::thread::{Builder, JoinHandle};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use zmq;
use zmq::{Context, Socket, SocketType};
use serde_json::{Value, Map};
use serde_json::to_string;
use serde_json::builder::ObjectBuilder;
use std::fmt::format;
use util::time::now;
use util::sendrecvjson::SendRecvJson;

pub struct ConnectorAsync {
    token: JoinHandle<()>,
    cache: Arc<Value>,
    stop_event : Arc<AtomicBool>
}

//noinspection RustSelfConvention
impl ConnectorAsync {
    pub fn new<F>(ip: String, port: i32, mut f: F) -> Result<ConnectorAsync, Box<Error>>
        where for<'r> F: FnMut(&'r mut Value) -> (), F: Send + 'static
    {
        let addr : String = format!("tcp://{}:{}", ip, port);
        let stop = Arc::new(AtomicBool::new(false));
        let their_stop = stop.clone();
        let mut v = Vec::<Value>::new();
        v.push(Value::F64(now()));
        v.push(Value::Null);
        let mut cache = Arc::new(Value::Array(v));
        let mut context = Context::new();
        let mut socket: Socket = context.socket(SocketType::SUB).unwrap();
        socket.set_subscribe("".as_bytes()).unwrap();
        socket.connect(&addr).unwrap();
        let token = Builder::new().name(format!("thread-{}",addr))
            .spawn(move ||{
                let mut context = context;
                let mut socket = socket;
                println!("started into loop!");
                while !their_stop.load(Ordering::SeqCst){
                    if socket.poll(zmq::POLLIN, 1000).unwrap()>0 {
                        println!("Got data, recv!");
                        let mut v: Value = socket.recv_json(0).unwrap();
                        if v.is_array(){
                            let mut a: &mut Vec<Value> = v.as_array_mut().unwrap();
                            if a[0].is_f64(){
                                let t = a[0].as_f64().unwrap();
                                a[0] = Value::Array(vec![Value::F64(now()), Value::F64(t)]);
                            }else if a[0].is_array() {
                                let aa = a[0].as_array_mut().unwrap();
                                aa.insert(0, Value::F64(now()));
                            }
                        }else{
                            let mut v1 = Value::Array(vec![Value::F64(now())]);
                            v = Value::Array(vec![v1, v]);
                        }
                        {f(&mut v);}
                    }else{
                        println!("No data, loop!");
                    }
                }
            }).unwrap();

        Ok(ConnectorAsync {
            token : token,
            cache: cache,
            stop_event : stop
        })
    }

    fn stop(&mut self){
        self.stop_event.store(true, Ordering::SeqCst);
    }

    pub fn to_locator<F>(ip: String, mut f:F) -> Result<ConnectorAsync, Box<Error>>
        where for<'r> F: FnMut(&'r mut Value) -> (), F: Send + 'static
    {
        ConnectorAsync::new(ip, 15701, f)
    }

}

impl Drop for ConnectorAsync{
    fn drop(&mut self){
        self.stop();
    }
}

mod tests{
    use super::ConnectorAsync;
    use util::sendrecvjson::SendRecvJson;
    use util::time::now;
    use std::thread::Builder;
    use std::panic::catch_unwind;
    use std::sync::mpsc::{channel, Sender, Receiver};
    use serde_json::{Value, Map};
    use serde_json::builder::ObjectBuilder;
    use zmq;
    use zmq::SocketType::{PUB};
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_stop() {
        let mut c = zmq::Context::new();
        let mut s: zmq::Socket = c.socket(PUB).unwrap();
        let (mut send, mut recv) = channel::<(bool, i32)>();
        s.bind("tcp://127.0.0.1:5655").unwrap();
        let mut i = 0;
        let ca1 = ConnectorAsync::new("127.0.0.1".into(), 5655, move |v: &mut Value| {
            let catcher = catch_unwind(|| {
                assert!(v.is_array());
                let d: &Vec<Value> = v.as_array().unwrap();
                assert!(d.len() == 2);
                let d1: &Vec<Value> = d[0].as_array().unwrap();
                match i {
                    0 => { assert!(d1.len() == 2) },
                    1 => { assert!(d1.len() == 3)},
                    3 => { assert!(d1.len() == 1)},
                    _ => {panic!("too much data!")}
                }
            });
            i += 1;
            send.send((catcher.is_ok(), i)).unwrap();
        });
        sleep(Duration::new(1,0));
        let mut val = Vec::<Value>::new();
        val.push(Value::F64(now()));
        val.push(Value::Null);
        println!("Sending!");
        s.send_json(&Value::Array(val), 0).unwrap();
        let mut val = Vec::<Value>::new();
        val.push(Value::Array(vec![Value::F64(now()),Value::F64(now())]));
        val.push(Value::Null);
        println!("Sending!");
        s.send_json(&Value::Array(val), 0).unwrap();
        s.send_json(&Value::Null, 0).unwrap();
        println!("Waiting result!");
        loop {
            match recv.recv() {
                Ok(v) => {
                    println!("Recvd {:?}", v);
                    assert!(v.0);
                    if v.1 == 3 {
                        break;
                    }
                },
                Err(e) => {
                    //println!("{:?}", e);
                    panic!("Connector exited too early! {}", e);
                    break;
                }
            }
        }
    }
}