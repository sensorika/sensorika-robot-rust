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
        let token = Builder::new().name(format!("thread-{}",addr))
            .spawn(move ||{
                let mut context = Context::new();
                let mut socket: Socket = context.socket(SocketType::SUB).unwrap();
                socket.connect(&addr).unwrap();
                while their_stop.load(Ordering::Relaxed){
                    if socket.poll(zmq::POLLIN, 1000).unwrap()>=0 {
                        let mut v = socket.recv_json(0).unwrap();
                        {f(&mut v);}
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
        self.stop_event.store(true, Ordering::Relaxed);
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
    use serde_json::{Value, Map};
    use serde_json::builder::ObjectBuilder;
    use zmq;
    use zmq::SocketType::{PUB};
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_stop(){
        let mut c = zmq::Context::new();
        let mut s: zmq::Socket = c.socket(PUB).unwrap();
        s.bind("tcp://127.0.0.1:5655").unwrap();
        let ca1 = ConnectorAsync::new("127.0.0.1".into(),5655, |v: &mut Value|{
            assert!(v.is_array());
            let d : &Vec<Value> = v.as_array().unwrap();
            assert!(d.len() == 2)

        });
        let mut val = Vec::<Value>::new();
        val.push(Value::F64(now()));
        val.push(Value::Null);
        s.send_json(&Value::Array(val), 0).unwrap();
        let mut val = Vec::<Value>::new();
        val.push(Value::F64(now()));
        val.push(Value::Null);
        s.send_json(&Value::Array(val), 0).unwrap();
    }
}