use zmq;
use std::str;
use std::error::Error;
use std::result::*;
use rand;
use rand::distributions::{IndependentSample, Range};

pub trait BindRandomPort{
    fn bind_to_random_port(&mut self, &str) -> Result<u32, zmq::Error>;
    fn bind_to_random_port_options(&mut self, &str, u32, u32, u32) -> Result<u32, zmq::Error>;
}

impl BindRandomPort for zmq::Socket {
    fn bind_to_random_port(&mut self, addr:&str) -> Result<u32, zmq::Error> {
        self.bind_to_random_port_options(addr, 49152, 65536, 100)
    }
    fn bind_to_random_port_options(
        &mut self, addr: &str,
        min_port: u32, max_port: u32,
        max_tries: u32) -> Result<u32, zmq::Error> {

        let ports = Range::new(min_port, max_port);
        let mut rng = rand::thread_rng();
        for it in 0 .. max_tries {
            let port = ports.ind_sample(&mut rng);
            match self.bind(&format!("{}:{}", addr, port)) {
                Ok(())=>{
                    return Ok(port)
                },
                Err(err)=>{
                    if err == zmq::Error::EADDRINUSE {
                        continue;
                    }else{
                        return Err(err)
                    }
                }
            }
        }
        Err(zmq::Error::EFAULT)
    }
}

#[cfg(test)]
mod tests {
    use super::BindRandomPort;
    use zmq;
    use zmq::SocketType::PUB;
    use std::error::Error;
    use std::result::*;

    #[test]
    fn test_bind_random_port() {
        let mut ctx = zmq::Context::new();
        let mut sock = ctx.socket(PUB).unwrap();
        let r = sock.bind_to_random_port("tcp:*");
        assert!(r.is_err() && r.err() == Some(zmq::Error::EINVAL));
        let r = sock.bind_to_random_port("rand://*");
        assert!(r.is_err()) ;
        assert_eq!(r.err(), Some(zmq::Error::EPROTONOSUPPORT));
    }


}