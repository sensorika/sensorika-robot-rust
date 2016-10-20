
use std::thread;
use std::ops::FnOnce;

struct Timer<F> {
    sec: u32,
    handler: F,
}

impl<F> Timer<F>
where F: FnOnce() + Send + Sync + 'static
{
    fn new(sec: u32, handler: F) -> Self {
        Timer {sec: sec, handler: handler}
    }

    fn start(self) {
        let _ = thread::spawn(move || {
            thread::sleep_ms(self.sec * 1000);
            (self.handler)();
        });
    }
}

#[cfg(test)]
mod tests{
    use super::Timer;
    use std::thread;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    fn nop(){}

    fn sleep(msec: u64){
        thread::sleep(Duration::from_millis(msec));
    }

    fn inc(x: &mut i32){
        *x += 1;
    }

    #[test]
    fn test_timer_count() {
        let r = thread::Builder::new().name("test timer count".into()).spawn(move ||{
            let counter = Arc::new(Mutex::new(0));
            let c_r = counter.clone();
            let t: Timer<_> = Timer::new(1, move ||{
                let mut number = c_r.lock().unwrap();
                *number += 1;
            });
            t.start();
            let c_r1 = counter.clone();
            sleep(3001);
            let n = c_r1.lock().unwrap();
            assert_eq!(*n, 1);
        }).unwrap();
        r.join().unwrap();
    }
}